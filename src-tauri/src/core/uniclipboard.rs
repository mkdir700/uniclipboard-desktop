use anyhow::Result;
use log::{error, info};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::signal::ctrl_c;
use tokio::sync::{mpsc, RwLock};

use crate::config::get_config_dir;
use crate::core::clipboard_content_receiver::ClipboardContentReceiver;
use crate::core::download_decision::DownloadDecisionMaker;
use crate::core::metadata::MetadataGenerator;
use crate::core::transfer::ClipboardTransferMessage;
use crate::infrastructure::connection::connection_manager::ConnectionManager;
use crate::infrastructure::storage::db::pool::DB_POOL;
use crate::infrastructure::storage::file_storage::FileStorageManager;
use crate::infrastructure::storage::record_manager::ClipboardRecordManager;
use crate::infrastructure::web::WebServer;
use crate::interface::local_clipboard_trait::LocalClipboardTrait;
use crate::interface::RemoteSyncManagerTrait;
use crate::message::Payload;

// 本地剪贴板管理器 - 负责监听本地剪贴板变化并推送到远程
pub struct LocalClipboardManager {
    device_id: String,
    clipboard: Arc<dyn LocalClipboardTrait>,
    remote_sync: Arc<dyn RemoteSyncManagerTrait>,
    is_running: Arc<RwLock<bool>>,
    is_paused: Arc<RwLock<bool>>,
    last_payload: Arc<RwLock<Option<Payload>>>,
    record_manager: Arc<ClipboardRecordManager>,
    file_storage: Arc<FileStorageManager>,
    metadata_generator: Arc<MetadataGenerator>,
}

impl LocalClipboardManager {
    pub fn new(
        clipboard: Arc<dyn LocalClipboardTrait>,
        remote_sync: Arc<dyn RemoteSyncManagerTrait>,
        record_manager: Arc<ClipboardRecordManager>,
        device_id: String,
    ) -> Result<Self> {
        let file_storage = Arc::new(FileStorageManager::new()?);
        let metadata_generator = Arc::new(MetadataGenerator::new(device_id.clone()));

        Ok(Self {
            device_id,
            clipboard,
            remote_sync,
            is_running: Arc::new(RwLock::new(false)),
            is_paused: Arc::new(RwLock::new(false)),
            last_payload: Arc::new(RwLock::new(None)),
            record_manager,
            file_storage,
            metadata_generator,
        })
    }

    pub async fn start(&self, clipboard_receiver: mpsc::Receiver<Payload>) -> Result<()> {
        {
            let mut is_running = self.is_running.write().await;
            *is_running = true;
        }
        let result = self.start_local_to_remote_sync(clipboard_receiver).await;
        info!("启动本地到远程同步完成");
        result
    }

    async fn start_local_to_remote_sync(
        &self,
        mut clipboard_receiver: mpsc::Receiver<Payload>,
    ) -> Result<()> {
        let device_id = self.device_id.clone();
        let remote_sync = self.remote_sync.clone();
        let is_running = self.is_running.clone();
        let last_payload = self.last_payload.clone();
        let record_manager = self.record_manager.clone();
        let file_storage = self.file_storage.clone();
        let metadata_generator = self.metadata_generator.clone();
        info!("Local to remote sync is running");

        tokio::spawn(async move {
            while *is_running.read().await {
                if let Some(payload) = clipboard_receiver.recv().await {
                    info!("Handle new clipboard content: {}", payload);
                    let last_content = last_payload.read().await;
                    if let Some(last_payload) = last_content.as_ref() {
                        if last_payload.is_duplicate(&payload) {
                            info!(
                                "Skip push to remote: {}, because it's the same as the last one",
                                payload
                            );
                            continue;
                        }
                    }
                    let tmp = last_content.clone();
                    drop(last_content);

                    {
                        *last_payload.write().await = Some(payload.clone());
                    }

                    // 步骤1: 将 Payload 持久化存储并获取存储路径
                    let storage_path = match file_storage.store(&payload).await {
                        Ok(path) => path,
                        Err(e) => {
                            error!("Failed to store payload: {:?}", e);
                            continue;
                        }
                    };

                    // 步骤2: 使用 payload + 本地存储路径，构建 metadata
                    let metadata = metadata_generator.generate_metadata(&payload, &storage_path);
                    info!("Push to remote: {}", metadata);
                    let result = record_manager.add_record_with_metadata(&metadata).await;

                    match result {
                        Ok(record_id) => {
                            // 步骤3: 将 metadata 发送至远程
                            let sync_message = ClipboardTransferMessage::from_metadata(
                                metadata,
                                device_id.clone(),
                                record_id,
                            );
                            if let Err(e) = remote_sync.push(sync_message).await {
                                // 恢复到之前的值
                                *last_payload.write().await = tmp;
                                // 处理错误，可能需要重试或记录日志
                                error!("Failed to push to remote: {:?}", e);
                            }
                        }
                        Err(e) => {
                            error!("Failed to add record: {:?}", e);
                            continue;
                        }
                    }
                }
            }
        });

        // 确保立即返回，不要阻塞
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        Ok(())
    }

    pub async fn pause(&self) -> Result<()> {
        let mut is_paused = self.is_paused.write().await;
        *is_paused = true;
        self.clipboard.pause().await;
        Ok(())
    }

    pub async fn resume(&self) -> Result<()> {
        let mut is_paused = self.is_paused.write().await;
        *is_paused = false;
        self.clipboard.resume().await;
        Ok(())
    }
}

// 远程剪贴板管理器 - 负责从远程获取内容并写入本地剪贴板
pub struct RemoteClipboardManager {
    clipboard: Arc<dyn LocalClipboardTrait>,
    remote_sync: Arc<dyn RemoteSyncManagerTrait>,
    record_manager: Arc<ClipboardRecordManager>,
    is_running: Arc<RwLock<bool>>,
    is_paused: Arc<RwLock<bool>>,
    last_payload: Arc<RwLock<Option<Payload>>>,
    download_decision_maker: Arc<DownloadDecisionMaker>,
    content_receiver: Arc<ClipboardContentReceiver>,
}

impl RemoteClipboardManager {
    pub fn new(
        clipboard: Arc<dyn LocalClipboardTrait>,
        remote_sync: Arc<dyn RemoteSyncManagerTrait>,
        record_manager: Arc<ClipboardRecordManager>,
        file_storage: Arc<FileStorageManager>,
    ) -> Self {
        Self {
            clipboard,
            remote_sync,
            record_manager: record_manager.clone(),
            is_running: Arc::new(RwLock::new(false)),
            is_paused: Arc::new(RwLock::new(false)),
            last_payload: Arc::new(RwLock::new(None)),
            download_decision_maker: Arc::new(DownloadDecisionMaker::new()),
            content_receiver: Arc::new(ClipboardContentReceiver::new(file_storage, record_manager)),
        }
    }

    pub async fn start(&self) -> Result<()> {
        {
            let mut is_running = self.is_running.write().await;
            *is_running = true;
        }
        info!("开始启动远程同步服务");
        self.remote_sync.start().await?;
        info!("远程同步服务已启动，开始处理远程到本地同步");
        self.start_remote_to_local_sync().await
    }

    async fn start_remote_to_local_sync(&self) -> Result<()> {
        let clipboard = self.clipboard.clone();
        let remote_sync = self.remote_sync.clone();
        let is_running = self.is_running.clone();
        let last_payload = self.last_payload.clone();
        let record_manager = self.record_manager.clone();
        let download_decision_maker = self.download_decision_maker.clone();
        let content_receiver = self.content_receiver.clone();

        tokio::spawn(async move {
            while *is_running.read().await {
                match remote_sync.pull(Some(Duration::from_secs(10))).await {
                    Ok(message) => {
                        // 经过下载决策层，决定是否下载
                        if !download_decision_maker.should_download(&message).await {
                            info!("内容类型不在下载范围内，跳过: {}", message);
                            continue;
                        }

                        // 检查是否超过最大允许大小
                        if download_decision_maker.exceeds_max_size(&message) {
                            info!("内容超过最大允许大小，跳过: {}", message);
                            continue;
                        }

                        // 添加到剪贴板记录
                        if let Err(e) = record_manager
                            .add_record_with_transfer_message(&message)
                            .await
                        {
                            error!("Failed to add clipboard record: {:?}", e);
                        }

                        // 接收内容
                        match content_receiver.receive(message).await {
                            Ok(payload) => {
                                info!("Set local clipboard: {}", payload);
                                let tmp = last_payload.read().await.clone();
                                {
                                    *last_payload.write().await = Some(payload.clone());
                                }

                                // 设置本地剪贴板
                                if let Err(e) = clipboard.set_clipboard_content(payload).await {
                                    error!("Failed to set clipboard content: {:?}", e);
                                    // 恢复到之前的值
                                    *last_payload.write().await = tmp;
                                }
                            }
                            Err(e) => {
                                error!("Failed to receive clipboard content: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to pull from remote: {:?}", e);
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        Ok(())
    }

    pub async fn pause(&self) -> Result<()> {
        let mut is_paused = self.is_paused.write().await;
        *is_paused = true;
        Ok(())
    }

    pub async fn resume(&self) -> Result<()> {
        let mut is_paused = self.is_paused.write().await;
        *is_paused = false;
        Ok(())
    }
}

// 主协调器 - 保持对外接口不变
pub struct UniClipboard {
    local_manager: Arc<LocalClipboardManager>,
    remote_manager: Arc<RemoteClipboardManager>,
    webserver: Arc<WebServer>,
    connection_manager: Arc<ConnectionManager>,
    clipboard_receiver: Option<mpsc::Receiver<Payload>>,
}

impl UniClipboard {
    pub fn new(
        device_id: String,
        webserver: Arc<WebServer>,
        clipboard: Arc<dyn LocalClipboardTrait>,
        remote_sync: Arc<dyn RemoteSyncManagerTrait>,
        connection_manager: Arc<ConnectionManager>,
        clipboard_record_manager: Arc<ClipboardRecordManager>,
        file_storage: Arc<FileStorageManager>,
    ) -> Self {
        // 创建本地剪贴板管理器
        let local_manager = Arc::new(
            LocalClipboardManager::new(
                clipboard.clone(),
                remote_sync.clone(),
                clipboard_record_manager.clone(),
                device_id.clone(),
            )
            .expect("Failed to create LocalClipboardManager"),
        );

        let remote_manager = Arc::new(RemoteClipboardManager::new(
            clipboard.clone(),
            remote_sync.clone(),
            clipboard_record_manager.clone(),
            file_storage.clone(),
        ));

        Self {
            local_manager,
            remote_manager,
            webserver,
            connection_manager,
            clipboard_receiver: None,
        }
    }

    pub fn get_record_manager(&self) -> Arc<ClipboardRecordManager> {
        self.local_manager.record_manager.clone()
    }

    pub fn get_file_storage_manager(&self) -> Arc<FileStorageManager> {
        self.local_manager.file_storage.clone()
    }

    pub async fn start(&self) -> Result<()> {
        // 使用作用域来确保锁被及时释放
        {
            let mut is_running = self.local_manager.is_running.write().await;
            if *is_running {
                anyhow::bail!("Already running");
            }
            *is_running = true;
        } // 锁在这里被释放

        // 初始化数据库
        // 如果环境变量中没有设置 DATABASE_URL，则设置默认的配置目录
        if env::var("DATABASE_URL").is_err() {
            let config_dir = get_config_dir()?;
            env::set_var(
                "DATABASE_URL",
                config_dir.join("uniclipboard.db").to_str().unwrap(),
            );
        }
        DB_POOL.init()?;

        // 与其他设备建立连接
        info!("Starting connection manager");
        self.connection_manager.start().await?;

        // 启动本地剪切板监听
        info!("Starting local clipboard monitoring");
        let clipboard_receiver = self.local_manager.clipboard.start_monitoring().await?;

        // 启动本地到远程的同步任务
        info!("Starting local to remote sync");
        self.local_manager.start(clipboard_receiver).await?;

        // 启动远程到本地的同步任务
        info!("Starting remote to local sync");
        self.remote_manager.start().await?;

        let webserver = self.webserver.clone();
        // 启动 Web 服务器
        tokio::spawn(async move {
            if let Err(e) = webserver.run().await {
                error!("Web server error: {:?}", e);
            }
        });

        info!("UniClipboard 全部启动完成");
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut is_running = self.local_manager.is_running.write().await;
        if !*is_running {
            anyhow::bail!("Not running");
        }
        *is_running = false;

        // 停止连接管理器
        self.connection_manager.stop().await;

        // 停止本地剪切板监听
        self.local_manager.clipboard.stop_monitoring().await?;

        // 停止远程同步
        self.remote_manager.remote_sync.stop().await?;

        // 停止本地管理器
        self.local_manager.stop().await?;

        // 停止远程管理器
        self.remote_manager.stop().await?;

        // 停止 Web 服务器
        self.webserver.shutdown().await?;

        Ok(())
    }

    pub async fn pause(&self) -> Result<()> {
        let mut is_paused = self.local_manager.is_paused.write().await;
        *is_paused = true;

        // 暂停本地剪切板管理器
        self.local_manager.pause().await?;

        // 暂停远程剪切板管理器
        self.remote_manager.pause().await?;

        // 暂停远程同步
        if let Err(e) = self.remote_manager.remote_sync.pause().await {
            error!("Failed to pause remote sync: {:?}", e);
        }

        Ok(())
    }

    pub async fn resume(&self) -> Result<()> {
        let mut is_paused = self.local_manager.is_paused.write().await;
        *is_paused = false;

        // 恢复本地剪切板管理器
        self.local_manager.resume().await?;

        // 恢复远程剪切板管理器
        self.remote_manager.resume().await?;

        // 恢复远程同步
        if let Err(e) = self.remote_manager.remote_sync.resume().await {
            error!("Failed to resume remote sync: {:?}", e);
        }

        Ok(())
    }

    pub async fn wait_for_stop(&self) -> Result<()> {
        loop {
            select! {
                _ = ctrl_c() => {
                    info!("收到 Ctrl+C，正在停止...");
                    break;
                }
            }
        }

        // 当收到 Ctrl+C 信号后，停止同步
        self.stop().await?;
        info!("剪贴板同步已停止");
        Ok(())
    }
}
