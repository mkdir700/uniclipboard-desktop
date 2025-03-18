use crate::application::device_service::{
    get_device_manager, subscribe_new_devices, GLOBAL_DEVICE_MANAGER,
};
use crate::config::Setting;
use crate::core::transfer::ClipboardTransferMessage;
use crate::domain::device::Device;
use crate::infrastructure::web::handlers::message_handler::MessageSource;
use crate::message::WebSocketMessage;
use anyhow::Result;
use futures::future::join_all;
use log::{error, info, warn};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tokio::task::JoinHandle;
use tokio::time::interval;

use super::incoming_manager::IncomingConnectionManager;
use super::outgoing_manager::OutgoingConnectionManager;
use super::{DeviceId, IpPort};

#[derive(Clone)]
pub struct ConnectionManager {
    pub incoming: IncomingConnectionManager,
    pub outgoing: OutgoingConnectionManager,
    addr_device_id_map: Arc<RwLock<HashMap<IpPort, DeviceId>>>,
    // TODO: 需要解耦 clipboard_message_sync_sender
    clipboard_message_sync_sender: Arc<broadcast::Sender<ClipboardTransferMessage>>,
    listen_new_devices_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
    try_connect_offline_devices_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
    user_setting: Setting,
}

impl ConnectionManager {
    pub fn new() -> Self {
        let (clipboard_message_sync_sender, _) = broadcast::channel(100);
        Self {
            incoming: IncomingConnectionManager::new(),
            outgoing: OutgoingConnectionManager::new(),
            addr_device_id_map: Arc::new(RwLock::new(HashMap::new())),
            clipboard_message_sync_sender: Arc::new(clipboard_message_sync_sender),
            listen_new_devices_handle: Arc::new(RwLock::new(None)),
            try_connect_offline_devices_handle: Arc::new(RwLock::new(None)),
            user_setting: Setting::get_instance(),
        }
    }

    /// 创建一个新的 ConnectionManager 实例，使用指定的配置
    pub fn with_user_setting(user_setting: Setting) -> Self {
        let (clipboard_message_sync_sender, _) = broadcast::channel(100);
        Self {
            incoming: IncomingConnectionManager::new(),
            outgoing: OutgoingConnectionManager::new(),
            addr_device_id_map: Arc::new(RwLock::new(HashMap::new())),
            clipboard_message_sync_sender: Arc::new(clipboard_message_sync_sender),
            listen_new_devices_handle: Arc::new(RwLock::new(None)),
            try_connect_offline_devices_handle: Arc::new(RwLock::new(None)),
            user_setting,
        }
    }

    /// 监听新增的设备
    ///
    /// 当有新设备上线且未连接时，尝试连接该设备
    async fn listen_new_devices(&self) -> JoinHandle<()> {
        let mut new_devices_rx = subscribe_new_devices();
        let self_clone = self.clone();

        tokio::spawn(async move {
            while let Ok(device) = new_devices_rx.recv().await {
                let is_connected = self_clone.is_connected(&device).await;
                if is_connected {
                    info!("Device {} is already connected, skip...", device);
                    continue;
                }
                match self_clone.outgoing.connect_device(&device).await {
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
        let outgoing_clone = self.outgoing.clone();

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

                    let mut connection_failed_devices = vec![];
                    for device in devices {
                        if let Err(e) = outgoing_clone.connect_device(&device).await {
                            connection_failed_devices.push((device, e));
                        }
                    }

                    if !connection_failed_devices.is_empty() {
                        warn!(
                            "Failed to connect to devices: {:?}",
                            connection_failed_devices
                        );
                    }
                }
            }
        })
    }

    /// 启动
    ///
    /// 1. 尝试连接设备列表中的所有设备
    /// 2. 开启 outgoing 的监听新设备
    pub async fn start(&self) -> Result<()> {
        info!("Start to connecat to devices");
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
                    .outgoing
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
                        match self.outgoing.connect_device(&device_clone).await {
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

        // 开启 outgoing 的监听新设备
        *self.listen_new_devices_handle.write().await = Some(self.listen_new_devices().await);
        // 尝试连接离线的设备
        *self.try_connect_offline_devices_handle.write().await =
            Some(self.try_connect_offline_devices().await);

        info!("Connection manager started");

        Ok(())
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

        self.outgoing.disconnect_all().await;
        self.incoming.disconnect_all().await;

        for (ip_port, _) in self.addr_device_id_map.read().await.iter() {
            if let Ok(addr) = ip_port.parse::<SocketAddr>() {
                self.remove_connection(MessageSource::IpPort(addr)).await;
            } else {
                error!("Invalid ip_port: {}", ip_port);
            }
        }

        for (device_id, _) in self.incoming.connections.read().await.iter() {
            self.remove_connection(MessageSource::DeviceId(device_id.clone()))
                .await;
        }

        info!("Connection manager stopped");
    }

    pub async fn update_device_ip_port(&self, device_id: DeviceId, ip_port: IpPort) {
        let mut map = self.addr_device_id_map.write().await;
        map.insert(ip_port, device_id);
    }

    pub async fn broadcast(
        &self,
        message: &WebSocketMessage,
        excludes: &Option<Vec<String>>,
    ) -> Result<()> {
        let mut errors: Vec<anyhow::Error> = Vec::new();

        if let Err(e) = self.outgoing.broadcast(message, excludes).await {
            errors.push(e);
        }
        if let Err(e) = self.incoming.broadcast(message, excludes).await {
            errors.push(e);
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to send message to some clients: {:?}",
                errors
            ))
        }
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
        let device_id = device.id.clone();
        let ip_port = format!(
            "{}:{}",
            device.ip.as_ref().unwrap_or(&"".to_string()),
            device.port.as_ref().unwrap_or(&0)
        );
        self.outgoing
            .connections
            .read()
            .await
            .contains_key(&device_id)
            || self
                .incoming
                .connections
                .read()
                .await
                .contains_key(&ip_port)
    }

    /// The `disconnect` function in Rust asynchronously disconnects a device based on its ID.
    ///
    /// Arguments:
    ///
    /// * `id`: The `id` parameter in the `disconnect` function is of type `DeviceId`, which is a reference
    /// to the identifier of a device.
    // pub async fn disconnect(&self, id: &String) {
    //     let ip_port = self.device_ip_port_map.read().await.get(id).cloned();
    //     if let Some(ip_port) = ip_port {
    //         self.incoming.disconnect(&ip_port).await;
    //     } else {
    //         self.outgoing.disconnect(id).await;
    //     }
    // }

    /// 断开指定设备并移除连接
    ///
    /// `id` 可能是 ip:port 或者 device_id
    pub async fn remove_connection(&self, id: MessageSource) {
        match id {
            MessageSource::IpPort(addr) => {
                self.remove_connection_by_addr(&format!("{}:{}", addr.ip(), addr.port()))
                    .await
            }
            MessageSource::DeviceId(device_id) => {
                self.remove_connection_by_device_id(&device_id).await
            }
        }
    }

    async fn remove_connection_by_device_id(&self, device_id: &DeviceId) {
        self.outgoing.remove_connection(device_id).await;
        if let Err(e) = GLOBAL_DEVICE_MANAGER.set_offline(device_id) {
            error!("Failed to set device {} offline: {}", device_id, e);
        }
    }

    async fn remove_connection_by_addr(&self, ip_port: &IpPort) {
        let device_id = self.addr_device_id_map.read().await.get(ip_port).cloned();
        self.incoming.remove_connection(ip_port).await;
        if let Some(device_id) = device_id {
            if let Err(e) = GLOBAL_DEVICE_MANAGER.set_offline(&device_id) {
                error!("Failed to set device {} offline: {}", device_id, e);
            }
        }
    }
}
