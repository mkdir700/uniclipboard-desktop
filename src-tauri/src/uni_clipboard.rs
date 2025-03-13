use anyhow::Result;
use log::{error, info};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::signal::ctrl_c;
use tokio::sync::{mpsc, RwLock};
use tokio::time::sleep;

use crate::clipboard::LocalClipboardTrait;
use crate::config::get_config_dir;
use crate::connection::ConnectionManager;
use crate::db::DB_POOL;
use crate::key_mouse_monitor::KeyMouseMonitorTrait;
use crate::message::{ClipboardSyncMessage, Payload};
use crate::remote_sync::RemoteSyncManagerTrait;
use crate::web::WebServer;

pub struct UniClipboard {
    clipboard: Arc<dyn LocalClipboardTrait>,
    remote_sync: Arc<dyn RemoteSyncManagerTrait>,
    key_mouse_monitor: Option<Arc<dyn KeyMouseMonitorTrait>>,
    is_running: Arc<RwLock<bool>>,
    is_paused: Arc<RwLock<bool>>,
    last_payload: Arc<RwLock<Option<Payload>>>,
    webserver: Arc<WebServer>,
    connection_manager: Arc<ConnectionManager>,
}

impl UniClipboard {
    pub fn new(
        webserver: Arc<WebServer>,
        clipboard: Arc<dyn LocalClipboardTrait>,
        remote_sync: Arc<dyn RemoteSyncManagerTrait>,
        key_mouse_monitor: Option<Arc<dyn KeyMouseMonitorTrait>>,
        connection_manager: Arc<ConnectionManager>,
    ) -> Self {
        Self {
            clipboard,
            remote_sync,
            key_mouse_monitor,
            is_running: Arc::new(RwLock::new(false)),
            is_paused: Arc::new(RwLock::new(false)),
            last_payload: Arc::new(RwLock::new(None)),
            webserver,
            connection_manager,
        }
    }

    #[cfg_attr(not(feature = "integration_tests"), ignore)]
    pub fn get_clipboard(&self) -> Arc<dyn LocalClipboardTrait> {
        self.clipboard.clone()
    }

    #[cfg_attr(not(feature = "integration_tests"), ignore)]
    pub fn get_remote_sync(&self) -> Arc<dyn RemoteSyncManagerTrait> {
        self.remote_sync.clone()
    }

    pub async fn start(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            anyhow::bail!("Already running");
        }
        *is_running = true;

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
        self.connection_manager.start().await?;

        // 启动本地剪切板监听
        let clipboard_receiver = self.clipboard.start_monitoring().await?;

        // 启动远程同步
        self.remote_sync.start().await?;

        // 启动本地到远程的同步任务
        self.start_local_to_remote_sync(clipboard_receiver).await?;

        self.start_remote_to_local_sync().await?;

        // 启动键盘鼠标监控
        if let Some(monitor) = self.key_mouse_monitor.as_ref() {
            monitor.start().await;
        }

        let webserver = self.webserver.clone();
        // 启动 Web 服务器
        tokio::spawn(async move {
            if let Err(e) = webserver.run().await {
                error!("Web server error: {:?}", e);
            }
        });

        Ok(())
    }

    async fn start_local_to_remote_sync(
        &self,
        mut clipboard_receiver: mpsc::Receiver<Payload>,
    ) -> Result<()> {
        let remote_sync = self.remote_sync.clone();
        let is_running = self.is_running.clone();
        let last_payload = self.last_payload.clone();

        tokio::spawn(async move {
            while *is_running.read().await {
                if let Some(payload) = clipboard_receiver.recv().await {
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

                    info!("Push to remote: {}", payload);
                    // ! 这里暂时使用 from_payload 方法，后续需要修改, 后续会引入存储器，由存储器生成 ClipboardSyncMessage
                    if let Err(e) = remote_sync
                        .push(ClipboardSyncMessage::from_payload(payload.clone()))
                        .await
                    {
                        // 恢复到之前的值
                        *last_payload.write().await = tmp;
                        // 处理错误，可能需要重试或记录日志
                        error!("Failed to push to remote: {:?}", e);
                    }
                }
            }
        });

        Ok(())
    }

    async fn start_remote_to_local_sync(&self) -> Result<()> {
        let clipboard = self.clipboard.clone();
        let remote_sync = self.remote_sync.clone();
        let is_running = self.is_running.clone();
        let last_payload = self.last_payload.clone();

        tokio::spawn(async move {
            while *is_running.read().await {
                match remote_sync.pull(Some(Duration::from_secs(10))).await {
                    Ok(content) => {
                        if content.contains_payload() {
                            let payload = content.payload().unwrap();
                            info!("Set local clipboard: {}", payload);
                            let tmp = last_payload.read().await.clone();
                            {
                                *last_payload.write().await = Some(payload.clone());
                            }
                            if let Err(e) = clipboard.set_clipboard_content(payload).await {
                                // 恢复到之前的值
                                *last_payload.write().await = tmp;
                                error!("Failed to set clipboard content: {:?}", e);
                            }
                        }
                        // ! 其他情况，等待用户主动下载
                    }
                    Err(e) => {
                        // 处理错误，可能需要重试或记录日志
                        error!("Failed to pull from remote: {:?}", e);
                    }
                }
                // 添加适当的延迟，避免过于频繁的同步
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if !*is_running {
            anyhow::bail!("Not running");
        }
        *is_running = false;

        // 停止连接管理器
        self.connection_manager.stop().await;

        // 停止本地剪切板监听
        self.clipboard.stop_monitoring().await?;

        // 停止远程同步
        self.remote_sync.stop().await?;

        // 停止 Web 服务器
        self.webserver.shutdown().await?;

        Ok(())
    }

    pub async fn pause(&self) -> Result<()> {
        let mut is_paused = self.is_paused.write().await;
        *is_paused = true;

        // 暂停本地剪切板监听
        self.clipboard.pause().await;

        // 暂停远程同步
        if let Err(e) = self.remote_sync.pause().await {
            error!("Failed to pause remote sync: {:?}", e);
        }

        Ok(())
    }

    pub async fn resume(&self) -> Result<()> {
        let mut is_paused = self.is_paused.write().await;
        *is_paused = false;

        // 恢复本地剪切板监听
        self.clipboard.resume().await;

        // 恢复远程同步
        if let Err(e) = self.remote_sync.resume().await {
            error!("Failed to resume remote sync: {:?}", e);
        }

        Ok(())
    }

    pub async fn wait_for_stop(&self) -> Result<()> {
        let mut last_is_sleep = false;

        loop {
            select! {
                _ = ctrl_c() => {
                    info!("收到 Ctrl+C，正在停止...");
                    break;
                }
                _ = async {
                    if let Some(monitor) = &self.key_mouse_monitor.as_ref() {
                        if monitor.is_sleep().await {
                            if !last_is_sleep {
                                if let Err(e) = self.pause().await {
                                    error!("暂停失���: {:?}", e);
                                }
                                last_is_sleep = true;
                                info!("剪贴板同步已暂停，因为键盘和鼠标处于睡眠状态");
                            }
                        } else {
                            if last_is_sleep {
                                if let Err(e) = self.resume().await {
                                    error!("恢复失败: {:?}", e);
                                }
                                last_is_sleep = false;
                                info!("剪贴板同步已恢复，因为键盘和鼠标已唤醒");
                            }
                        }
                    }
                    sleep(Duration::from_secs(1)).await;
                } => {}
            }
        }

        // 当收到 Ctrl+C 信号后，停止同步
        self.stop().await?;
        info!("剪贴板同步已停止");
        Ok(())
    }
}

pub struct UniClipboardBuilder {
    webserver: Option<WebServer>,
    clipboard: Option<Arc<dyn LocalClipboardTrait>>,
    remote_sync: Option<Arc<dyn RemoteSyncManagerTrait>>,
    key_mouse_monitor: Option<Arc<dyn KeyMouseMonitorTrait>>,
    connection_manager: Option<Arc<ConnectionManager>>,
}

impl UniClipboardBuilder {
    pub fn new() -> Self {
        Self {
            webserver: None,
            clipboard: None,
            remote_sync: None,
            key_mouse_monitor: None,
            connection_manager: None,
        }
    }

    pub fn set_webserver(mut self, webserver: WebServer) -> Self {
        self.webserver = Some(webserver);
        self
    }

    pub fn set_local_clipboard(mut self, clipboard: Arc<dyn LocalClipboardTrait>) -> Self {
        self.clipboard = Some(clipboard);
        self
    }

    pub fn set_remote_sync(mut self, remote_sync: Arc<dyn RemoteSyncManagerTrait>) -> Self {
        self.remote_sync = Some(remote_sync);
        self
    }

    pub fn set_connection_manager(mut self, connection_manager: Arc<ConnectionManager>) -> Self {
        self.connection_manager = Some(connection_manager);
        self
    }

    #[allow(dead_code)]
    pub fn set_key_mouse_monitor(
        mut self,
        key_mouse_monitor: Arc<dyn KeyMouseMonitorTrait>,
    ) -> Self {
        self.key_mouse_monitor = Some(key_mouse_monitor);
        self
    }

    pub fn build(self) -> Result<UniClipboard> {
        let clipboard = self
            .clipboard
            .ok_or_else(|| anyhow::anyhow!("No local clipboard set"))?;
        let remote_sync = self
            .remote_sync
            .ok_or_else(|| anyhow::anyhow!("No remote sync set"))?;

        let webserver = self
            .webserver
            .ok_or_else(|| anyhow::anyhow!("No web server set"))?;
        let connection_manager = self
            .connection_manager
            .ok_or_else(|| anyhow::anyhow!("No connection manager set"))?;
        Ok(UniClipboard::new(
            Arc::new(webserver),
            clipboard,
            remote_sync,
            self.key_mouse_monitor,
            connection_manager,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        connection::ConnectionManager, message::ClipboardSyncMessage,
        remote_sync::RemoteClipboardSync, web::WebSocketHandler, web::WebSocketMessageHandler,
    };

    use super::*;
    use anyhow::Result;
    use async_trait::async_trait;
    use bytes::Bytes;
    use serial_test::serial;
    use std::fs;
    use std::{env, net::SocketAddr, sync::Arc};
    use tokio::sync::Mutex;

    fn setup_test_env() {
        env::set_var("DATABASE_URL", "uniclipboard_tests.db");
    }

    #[ctor::ctor]
    fn setup() {
        setup_test_env();
    }

    // 这个函数会在模块中的所有测试运行之后执行
    #[ctor::dtor]
    fn teardown() {
        // 删除测试数据库文件
        if let Err(e) = fs::remove_file("uniclipboard_tests.db") {
            println!("清理测试数据库时出错: {}", e);
        }
    }

    // 模拟本地剪贴板
    struct MockLocalClipboard {
        content: Arc<Mutex<Option<Payload>>>,
    }

    #[async_trait::async_trait]
    impl LocalClipboardTrait for MockLocalClipboard {
        async fn start_monitoring(&self) -> Result<tokio::sync::mpsc::Receiver<Payload>> {
            // 简化实现，返回一个空的接收器
            let (_, rx) = tokio::sync::mpsc::channel(10);
            Ok(rx)
        }

        async fn read(&self) -> Result<Payload> {
            Ok(self.content.lock().await.clone().unwrap())
        }

        async fn write(&self, content: Payload) -> Result<()> {
            *self.content.lock().await = Some(content);
            Ok(())
        }

        async fn stop_monitoring(&self) -> Result<()> {
            Ok(())
        }

        async fn set_clipboard_content(&self, content: Payload) -> Result<()> {
            *self.content.lock().await = Some(content);
            Ok(())
        }

        async fn pause(&self) {}

        async fn resume(&self) {}
    }

    // 模拟远程同步管理器
    struct MockRemoteSync {
        content: Arc<Mutex<Option<Payload>>>,
    }

    #[async_trait]
    impl RemoteSyncManagerTrait for MockRemoteSync {
        async fn sync(&self) -> Result<()> {
            Ok(())
        }

        async fn set_sync_handler(&self, _handler: Arc<dyn RemoteClipboardSync>) {}

        async fn start(&self) -> Result<()> {
            Ok(())
        }

        async fn stop(&self) -> Result<()> {
            Ok(())
        }

        async fn push(&self, message: ClipboardSyncMessage) -> Result<()> {
            *self.content.lock().await = Some(message.payload.unwrap());
            Ok(())
        }

        async fn pull(
            &self,
            _timeout: Option<std::time::Duration>,
        ) -> Result<ClipboardSyncMessage> {
            Ok(ClipboardSyncMessage::from_payload(
                self.content.lock().await.clone().unwrap(),
            ))
        }

        async fn pause(&self) -> Result<()> {
            Ok(())
        }

        async fn resume(&self) -> Result<()> {
            Ok(())
        }
    }

    // 模拟键鼠监控器
    struct MockKeyMouseMonitor {
        is_sleep: Arc<Mutex<bool>>,
    }

    #[async_trait::async_trait]
    impl KeyMouseMonitorTrait for MockKeyMouseMonitor {
        async fn start(&self) {}

        async fn is_sleep(&self) -> bool {
            *self.is_sleep.lock().await
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_uni_clipboard_creation() {
        let clipboard = Arc::new(MockLocalClipboard {
            content: Arc::new(Mutex::new(None)),
        });
        let remote_sync = Arc::new(MockRemoteSync {
            content: Arc::new(Mutex::new(None)),
        });
        let key_mouse_monitor = Arc::new(MockKeyMouseMonitor {
            is_sleep: Arc::new(Mutex::new(false)),
        });
        let connection_manager = Arc::new(ConnectionManager::new());
        let websocket_message_handler =
            Arc::new(WebSocketMessageHandler::new(connection_manager.clone()));
        let websocket_handler = Arc::new(WebSocketHandler::new(
            websocket_message_handler.clone(),
            connection_manager.clone(),
        ));
        let webserver = WebServer::new(
            SocketAddr::new("0.0.0.0".parse().unwrap(), 8114),
            websocket_handler,
        );

        let uni_clipboard = UniClipboardBuilder::new()
            .set_local_clipboard(clipboard)
            .set_remote_sync(remote_sync)
            .set_key_mouse_monitor(key_mouse_monitor)
            .set_webserver(webserver)
            .set_connection_manager(connection_manager)
            .build()
            .expect("Failed to build UniClipboard");

        assert!(uni_clipboard.start().await.is_ok());
        assert!(uni_clipboard.stop().await.is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_uni_clipboard_sync() {
        let clipboard = Arc::new(MockLocalClipboard {
            content: Arc::new(Mutex::new(None)),
        });
        let remote_sync = Arc::new(MockRemoteSync {
            content: Arc::new(Mutex::new(None)),
        });
        let connection_manager = Arc::new(ConnectionManager::new());
        let websocket_message_handler =
            Arc::new(WebSocketMessageHandler::new(connection_manager.clone()));
        let websocket_handler = Arc::new(WebSocketHandler::new(
            websocket_message_handler.clone(),
            connection_manager.clone(),
        ));
        let webserver = WebServer::new(
            SocketAddr::new("0.0.0.0".parse().unwrap(), 8114),
            websocket_handler,
        );

        let uni_clipboard = UniClipboardBuilder::new()
            .set_local_clipboard(clipboard.clone())
            .set_remote_sync(remote_sync.clone())
            .set_webserver(webserver)
            .set_connection_manager(connection_manager)
            .build()
            .expect("Failed to build UniClipboard");

        assert!(uni_clipboard.start().await.is_ok());

        // 模拟远程同步
        let test_payload = Payload::new_text(
            Bytes::from("Test content".to_string()),
            "device_id".to_string(),
            chrono::Utc::now(),
        );
        assert!(remote_sync
            .push(ClipboardSyncMessage::from_payload(test_payload.clone()))
            .await
            .is_ok());

        // 等待同步
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        // 检查本地剪贴板是否已更新
        let local_content = clipboard.content.lock().await;
        assert_eq!(*local_content, Some(test_payload));

        assert!(uni_clipboard.stop().await.is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_uni_clipboard_pause_resume() {
        let clipboard = Arc::new(MockLocalClipboard {
            content: Arc::new(Mutex::new(None)),
        });
        let remote_sync = Arc::new(MockRemoteSync {
            content: Arc::new(Mutex::new(None)),
        });
        let connection_manager = Arc::new(ConnectionManager::new());
        let websocket_message_handler =
            Arc::new(WebSocketMessageHandler::new(connection_manager.clone()));
        let websocket_handler = Arc::new(WebSocketHandler::new(
            websocket_message_handler.clone(),
            connection_manager.clone(),
        ));
        let webserver = WebServer::new(
            SocketAddr::new("0.0.0.0".parse().unwrap(), 8114),
            websocket_handler,
        );

        let uni_clipboard = UniClipboardBuilder::new()
            .set_local_clipboard(clipboard)
            .set_remote_sync(remote_sync)
            .set_webserver(webserver)
            .set_connection_manager(connection_manager)
            .build()
            .expect("Failed to build UniClipboard");

        assert!(uni_clipboard.start().await.is_ok());
        assert!(uni_clipboard.pause().await.is_ok());
        assert!(uni_clipboard.resume().await.is_ok());
        assert!(uni_clipboard.stop().await.is_ok());
    }
}
