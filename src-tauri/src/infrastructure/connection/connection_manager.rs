use crate::application::device_service::{
    get_device_manager, subscribe_new_devices, GLOBAL_DEVICE_MANAGER,
};
use crate::config::Setting;
use crate::domain::transfer_message::ClipboardTransferMessage;
use crate::domain::device::Device;
use crate::infrastructure::web::handlers::message_handler::MessageSource;
use crate::infrastructure::connection::PendingConnectionsManager;
use crate::infrastructure::connection::unified_manager::UnifiedConnectionManager;
use crate::message::WebSocketMessage;
use anyhow::Result;
use futures::future::join_all;
use log::{error, info, warn};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tokio::task::JoinHandle;
use tokio::time::interval;

use super::{DeviceId, IpPort};

#[derive(Clone)]
pub struct ConnectionManager {
    /// 统一连接管理器
    pub unified: Arc<UnifiedConnectionManager>,

    /// Clipboard 消息同步发送器
    clipboard_message_sync_sender: Arc<broadcast::Sender<ClipboardTransferMessage>>,

    /// 后台任务句柄
    listen_new_devices_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
    try_connect_offline_devices_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
    pending_connections_cleanup_handle: Arc<RwLock<Option<JoinHandle<()>>>>,

    /// 待处理连接管理器
    pub pending_connections: Arc<PendingConnectionsManager>,

    /// 配置
    user_setting: Setting,
}

impl ConnectionManager {
    pub fn new() -> Self {
        let (clipboard_message_sync_sender, _) = broadcast::channel(100);
        let pending_connections = Arc::new(PendingConnectionsManager::new());
        let user_setting = Setting::get_instance();
        Self {
            unified: Arc::new(UnifiedConnectionManager::new(user_setting.clone())),
            clipboard_message_sync_sender: Arc::new(clipboard_message_sync_sender),
            listen_new_devices_handle: Arc::new(RwLock::new(None)),
            try_connect_offline_devices_handle: Arc::new(RwLock::new(None)),
            pending_connections_cleanup_handle: Arc::new(RwLock::new(None)),
            user_setting,
            pending_connections,
        }
    }

    /// 创建一个新的 ConnectionManager 实例，使用指定的配置
    pub fn with_user_setting(user_setting: Setting) -> Self {
        let (clipboard_message_sync_sender, _) = broadcast::channel(100);
        let pending_connections = Arc::new(PendingConnectionsManager::new());
        Self {
            unified: Arc::new(UnifiedConnectionManager::new(user_setting.clone())),
            clipboard_message_sync_sender: Arc::new(clipboard_message_sync_sender),
            listen_new_devices_handle: Arc::new(RwLock::new(None)),
            try_connect_offline_devices_handle: Arc::new(RwLock::new(None)),
            pending_connections_cleanup_handle: Arc::new(RwLock::new(None)),
            user_setting,
            pending_connections,
        }
    }

    /// 监听新增的设备
    ///
    /// 当有新设备上线且未连接时，尝试连接该设备
    async fn listen_new_devices(&self) -> JoinHandle<()> {
        let mut new_devices_rx = subscribe_new_devices();
        let unified = self.unified.clone();

        tokio::spawn(async move {
            while let Ok(device) = new_devices_rx.recv().await {
                let is_connected = unified.is_connected(&device.id).await;
                if is_connected {
                    info!("Device {} is already connected, skip...", device);
                    continue;
                }
                match unified.connect_device(&device).await {
                    Ok(_) => info!("A new device connected: {}", device),
                    Err(e) => error!("Failed to connect to new device: {}, error: {}", device, e),
                }
            }
        })
    }

    /// 定时去连接离线的设备
    ///
    /// 1. 获取所有离线的设备
    /// 2. 尝试连接这些设备
    async fn try_connect_offline_devices(&self) -> JoinHandle<()> {
        let unified = self.unified.clone();

        tokio::spawn(async move {
            // 暂时每分钟检查一次
            let mut interval = interval(Duration::from_secs(60));

            loop {
                interval.tick().await;
                if let Ok(devices) = get_device_manager().get_offline_devices() {
                    let count = devices.len();
                    if count > 0 {
                        info!("Found {} offline devices, try to connect them", count);
                    }

                    for device in devices {
                        if let Err(e) = unified.connect_device(&device).await {
                            error!("Failed to connect to device {}: {}", device, e);
                        }
                    }
                }
            }
        })
    }

    /// 启动
    ///
    /// 1. 尝试连接设备列表中的所有设备
    /// 2. 开启监听新设备
    pub async fn start(&self) -> Result<()> {
        info!("Start to connect to devices");
        // 获取设备管理器的锁
        let devices = get_device_manager()
            .get_all_devices_except_self()
            .map_err(|_| anyhow::anyhow!("Failed to get all devices"))?;

        let peer_device_addr = self.user_setting.network.peer_device_addr.clone();
        let peer_device_port = self.user_setting.network.peer_device_port;

        // 如果 devices 为空，则尝试从配置中获取对等设备
        //  这种情况可能是，用户手动配置的对等设备
        if devices.is_empty() {
            if let (Some(peer_device_addr), Some(peer_device_port)) =
                (peer_device_addr, peer_device_port)
            {
                info!(
                    "Start to connect to peer device: {}:{}",
                    peer_device_addr, peer_device_port
                );
                match self
                    .unified
                    .connect_with_peer_device(&peer_device_addr, peer_device_port)
                    .await
                {
                    Ok(_) => info!(
                        "Connected to peer device: {}:{}",
                        peer_device_addr, peer_device_port
                    ),
                    Err(e) => error!(
                        "Failed to connect to peer device: {}:{}, error: {}",
                        peer_device_addr, peer_device_port, e
                    ),
                }
            } else {
                warn!("Peer device address or port is not set, so skip connecting to peer device");
            }
        } else {
            info!("Start to connect to {} devices", devices.len());
            // 创建所有连接任务
            let connection_futures: Vec<_> = devices
                .iter()
                .map(|device| {
                    let device_clone = device.clone();
                    async move {
                        match self.unified.connect_device(&device_clone).await {
                            Ok(_) => {
                                info!("Successfully connected to device: {}", device_clone);
                                Ok(device_clone.id.clone())
                            }
                            Err(e) => {
                                warn!("Failed to connect to device {}: {}", device_clone, e);
                                Err((device_clone.id.clone(), e))
                            }
                        }
                    }
                })
                .collect();

            // 并行执行所有连接
            let results = join_all(connection_futures).await;

            // 处理结果
            let (successes, errors): (Vec<_>, Vec<_>) =
                results.into_iter().partition(Result::is_ok);

            info!("Connected to {} devices", successes.len());
            if !errors.is_empty() {
                warn!(
                    "Failed to connect to {} devices: {:?}",
                    errors.len(),
                    errors
                        .into_iter()
                        .map(Result::unwrap_err)
                        .collect::<Vec<_>>()
                );
            } else {
                info!("All devices connected successfully");
            }
        }

        // 开启监听新设备
        *self.listen_new_devices_handle.write().await = Some(self.listen_new_devices().await);
        // 尝试连接离线的设备
        *self.try_connect_offline_devices_handle.write().await =
            Some(self.try_connect_offline_devices().await);

        // 启动待处理连接清理任务
        *self.pending_connections_cleanup_handle.write().await =
            Some(self.start_pending_connections_cleanup().await);

        info!("Connection manager started");

        Ok(())
    }

    /// 启动待处理连接清理任务
    async fn start_pending_connections_cleanup(&self) -> JoinHandle<()> {
        let pending_connections = self.pending_connections.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                // 清理超过 60 秒的过期出站请求
                pending_connections.cleanup_expired_outgoing(60).await;
            }
        })
    }

    /// 停止
    ///
    /// 1. 断开所有连接
    /// 2. 移除所有连接
    /// 3. 设置所有设备为 offline
    pub async fn stop(&self) {
        if let Some(handle) = self.listen_new_devices_handle.write().await.take() {
            handle.abort();
        }
        if let Some(handle) = self.try_connect_offline_devices_handle.write().await.take() {
            handle.abort();
        }
        if let Some(handle) = self
            .pending_connections_cleanup_handle
            .write()
            .await
            .take()
        {
            handle.abort();
        }

        // 清空所有待处理连接
        self.pending_connections.clear_all().await;

        // 断开所有连接
        self.unified.disconnect_all().await;

        info!("Connection manager stopped");
    }

    pub async fn broadcast(
        &self,
        message: &WebSocketMessage,
        excludes: &Option<Vec<String>>,
    ) -> Result<()> {
        self.unified.broadcast(message, excludes).await
    }

    // TODO: 需要解耦
    pub async fn send_clipboard_sync(&self, message: ClipboardTransferMessage) {
        let _ = self.clipboard_message_sync_sender.send(message);
    }

    // TODO: 需要解耦
    pub async fn subscribe_clipboard_sync(&self) -> broadcast::Receiver<ClipboardTransferMessage> {
        self.clipboard_message_sync_sender.subscribe()
    }

    pub async fn is_connected(&self, device: &Device) -> bool {
        self.unified.is_connected(&device.id).await
    }

    /// 断开指定设备并移除连接
    ///
    /// `id` 可能是 ip:port 或者 device_id
    pub async fn remove_connection(&self, id: MessageSource) {
        match id {
            MessageSource::IpPort(addr) => {
                self.unified.remove_connection_by_addr(addr).await;
            }
            MessageSource::DeviceId(device_id) => {
                self.unified.remove_connection(&device_id).await;
            }
        }
    }
}
