use crate::config::Config;
use crate::device::get_device_manager;
use crate::device::{subscribe_new_devices, Device, GLOBAL_DEVICE_MANAGER};
use crate::message::{ClipboardSyncMessage, WebSocketMessage};
use crate::network::WebSocketClient;
use crate::web::handlers::client::IncommingWebsocketClient;
use crate::web::handlers::message_handler::MessageSource;
use anyhow::Result;
use futures::future::join_all;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tokio::task::JoinHandle;
use tokio::time::interval;
use tokio_tungstenite::tungstenite::http::Uri;
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;
use warp::ws::Message as WarpMessage;

pub type DeviceId = String;
pub type IpPort = String;
pub type Clients = HashMap<IpPort, IncommingWebsocketClient>;

#[derive(Clone)]
pub struct IncomingConnectionManager {
    connections: Arc<RwLock<Clients>>,
}

impl IncomingConnectionManager {
    fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(Clients::default())),
        }
    }

    pub async fn add_connection(&self, client: IncommingWebsocketClient) {
        let mut clients = self.connections.write().await;
        clients.insert(client.id(), client);
    }

    pub async fn remove_connection(&self, ip_port: &IpPort) {
        self.disconnect(ip_port).await;
        let mut clients = self.connections.write().await;
        clients.remove(ip_port);
    }

    pub async fn count(&self) -> usize {
        self.connections.read().await.len()
    }

    async fn disconnect(&self, ip_port: &IpPort) {
        let client = {
            let clients = self.connections.read().await;
            clients.get(ip_port).cloned()
        };

        // send offline message
        if let Some(client) = client {
            let _ = client.stop().await;
        }
    }

    /// 断开所有连接
    ///
    /// 向所有已连接的设备发送离线消息
    pub async fn disconnect_all(&self) {
        info!("Disconnecting all connections");
        let clients = {
            let mut clients = self.connections.write().await;
            std::mem::take(&mut *clients)
        };

        let close_connections = clients.into_iter().map(|(id, tx)| async move {
            if let Err(e) = tx.send(WarpMessage::close()).await {
                error!("Failed to send close message to client {}: {}", id, e);
            }
            // 等待一小段时间，让客户端有机会处理关闭消息
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        });

        // 并发执行所有关闭操作
        join_all(close_connections).await;

        debug!("All connections have been closed");
    }

    async fn broadcast(
        &self,
        message: &WebSocketMessage,
        excludes: &Option<Vec<String>>,
    ) -> Result<()> {
        let message_str = serde_json::to_string(&message).unwrap();
        let message = WarpMessage::text(message_str);

        // 只在短时间内持有锁，仅用于克隆需要的数据
        let futures = {
            let clients_guard = self.connections.read().await;
            clients_guard
                .iter()
                .filter(|(&ref client_id, _)| {
                    if let Some(exclude_ids) = &excludes {
                        !exclude_ids.contains(&client_id.to_string())
                    } else {
                        true
                    }
                })
                .map(|(client_id, tx)| {
                    let message = message.clone();
                    let client_id = client_id.clone();
                    let tx = tx.clone(); // 克隆发送器
                    (client_id, tx, message)
                })
                .collect::<Vec<_>>()
        }; // 锁在这里被释放

        // 在锁释放后执行实际的发送操作
        let send_futures = futures
            .into_iter()
            .map(|(client_id, tx, message)| async move {
                if let Err(e) = tx.send(message).await {
                    error!("Failed to send message to {}: {}", client_id, e);
                    return Err(e);
                }
                debug!("Sent message to {}", client_id);
                Ok(())
            });

        join_all(send_futures).await;
        Ok(())
    }
}

#[derive(Clone)]
pub struct OutgoingConnectionManager {
    connections: Arc<
        RwLock<
            HashMap<
                DeviceId,
                (
                    Arc<RwLock<WebSocketClient>>,
                    broadcast::Receiver<TungsteniteMessage>,
                    tokio::task::JoinHandle<()>,
                    tokio::task::JoinHandle<()>,
                ),
            >,
        >,
    >,
    messages_tx: Arc<broadcast::Sender<(String, TungsteniteMessage)>>,
}

impl OutgoingConnectionManager {
    fn new() -> Self {
        let (message_tx, _) = broadcast::channel(20);
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            messages_tx: Arc::new(message_tx),
        }
    }

    /// 连接指定设备
    pub async fn connect_device(&self, device: &Device) -> Result<()> {
        let uri = format!(
            "ws://{}:{}/ws",
            device.ip.as_ref().unwrap(),
            device.server_port.as_ref().unwrap()
        )
        .parse::<Uri>()
        .unwrap();

        let mut client = WebSocketClient::new(uri);
        client.connect().await?;
        client.register(None).await?;
        client.sync_device_list().await?;

        info!("Connected to device: {}", device);

        self.add_connection(device.id.clone(), client).await;
        Ok(())
    }

    /// 连接对等设备
    async fn connect_with_peer_device(
        &self,
        peer_device_addr: &str,
        peer_device_port: u16,
    ) -> Result<()> {
        if peer_device_addr.is_empty() || peer_device_port == 0 {
            return Err(anyhow::anyhow!("Peer device address or port is not set"));
        }

        let uri = format!("ws://{}:{}/ws", peer_device_addr, peer_device_port)
            .parse::<Uri>()
            .unwrap();
        let mut client = WebSocketClient::new(uri);
        client.connect().await?;
        client.register(None).await?;
        client.sync_device_list().await?;
        self.add_connection(format!("{}:{}", peer_device_addr, peer_device_port), client)
            .await;
        Ok(())
    }

    pub async fn subscribe_outgoing_connections_message(
        &self,
    ) -> broadcast::Receiver<(String, TungsteniteMessage)> {
        self.messages_tx.subscribe()
    }

    /// 添加一个连接
    ///
    /// 创建一个异步任务，将接收到的消息转发到 outgoing_connections_message_tx
    /// 创建一个健康检查任务，定期向所有连接发送 ping 消息，如果某个连接没有响应，则认为该连接已断开
    pub async fn add_connection(&self, id: DeviceId, client: WebSocketClient) {
        let mut clients = self.connections.write().await;
        let message_rx = client.subscribe();
        let arc_client = Arc::new(RwLock::new(client));
        let outgoing_connections_message_tx = self.messages_tx.clone();
        let self_clone = self.clone();

        let arc_client_clone = arc_client.clone();
        let id_clone = id.clone();
        // 启动消息转发
        let forward_message_task = tokio::spawn(async move {
            let mut message_rx = { arc_client_clone.clone().read().await.subscribe() };
            loop {
                let message = message_rx.recv().await;
                if let Ok(message) = message {
                    let _ = outgoing_connections_message_tx.send((id_clone.clone(), message));
                }
            }
        });

        let arc_client_clone = arc_client.clone();
        let id_clone = id.clone();
        // 启动健康检查
        let health_check_task = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));

            loop {
                interval.tick().await;
                if !arc_client_clone.clone().read().await.is_connected() {
                    info!("Health check: device [{}] is disconnected", id_clone);
                    break;
                }
            }
            // ! 设备状态没有发生变更
            self_clone.remove_connection(&id_clone).await;
        });

        // 设置设备状态为 online
        if let Err(e) = GLOBAL_DEVICE_MANAGER.set_online(&id) {
            error!("Failed to set device {} online: {}", id, e);
        }

        clients.insert(
            id,
            (
                arc_client,
                message_rx,
                forward_message_task,
                health_check_task,
            ),
        );
    }

    async fn remove_connection(&self, id: &DeviceId) {
        self.disconnect(id).await;
        let mut clients = self.connections.write().await;
        if let Some(client) = clients.get_mut(id) {
            client.2.abort();
            client.3.abort();
            clients.remove(id);
        }
    }

    #[allow(dead_code)]
    pub async fn count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// 断开连接
    ///
    /// 1. 断开连接
    /// 2. 设置状态为 offline
    async fn disconnect(&self, id: &DeviceId) {
        let ws_client = {
            let clients = self.connections.read().await;
            clients.get(id).map(|conn| conn.0.clone())
        };

        if let Some(ws_client) = ws_client {
            let _ = ws_client.write().await.disconnect().await;
        }

        if let Err(e) = GLOBAL_DEVICE_MANAGER.set_offline(id) {
            error!("Failed to set device {} offline: {}", id, e);
        }
    }

    /// 断开所有连接
    pub async fn disconnect_all(&self) {
        for (_device_id, (client, _, _, _)) in self.connections.read().await.iter() {
            let _ = client.write().await.disconnect().await;
        }
    }

    async fn broadcast(
        &self,
        message: &WebSocketMessage,
        excludes: &Option<Vec<String>>,
    ) -> Result<()> {
        let clients = self.connections.read().await;
        let mut errors = Vec::new();

        for (id, client) in clients.iter() {
            if let Some(exclude_ids) = excludes {
                if exclude_ids.contains(id) {
                    continue;
                }
            }

            let client = client.0.read().await;
            match client.send_with_websocket_message(message).await {
                Ok(_) => {}
                Err(e) => {
                    error!("Failed to send message to client {}: {}", id, e);
                    errors.push((id.clone(), e));
                }
            }
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
}

#[derive(Clone)]
pub struct ConnectionManager {
    pub incoming: IncomingConnectionManager,
    pub outgoing: OutgoingConnectionManager,
    addr_device_id_map: Arc<RwLock<HashMap<IpPort, DeviceId>>>,
    // TODO: 需要解耦 clipboard_message_sync_sender
    clipboard_message_sync_sender: Arc<broadcast::Sender<ClipboardSyncMessage>>,
    listen_new_devices_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
    try_connect_offline_devices_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
    config: Config,
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
            config: Config::get_instance(),
        }
    }

    /// 创建一个新的 ConnectionManager 实例，使用指定的配置
    pub fn with_config(config: Config) -> Self {
        let (clipboard_message_sync_sender, _) = broadcast::channel(100);
        Self {
            incoming: IncomingConnectionManager::new(),
            outgoing: OutgoingConnectionManager::new(),
            addr_device_id_map: Arc::new(RwLock::new(HashMap::new())),
            clipboard_message_sync_sender: Arc::new(clipboard_message_sync_sender),
            listen_new_devices_handle: Arc::new(RwLock::new(None)),
            try_connect_offline_devices_handle: Arc::new(RwLock::new(None)),
            config,
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
        info!("Start to connect to devices");
        // 获取设备管理器的锁
        let devices = get_device_manager()
            .get_all_devices_except_self()
            .map_err(|_| anyhow::anyhow!("Failed to get all devices"))?;

        // 使用实例的 config 字段，而不是全局 CONFIG 变量
        let peer_device_addr = self.config.peer_device_addr.clone();
        let peer_device_port = self.config.peer_device_port;

        // 如果 devices 为空，则尝试从配置中获取对等设备
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
    pub async fn send_clipboard_sync(&self, message: ClipboardSyncMessage) {
        let _ = self.clipboard_message_sync_sender.send(message);
    }

    // TODO: 需要解耦
    pub async fn subscribe_clipboard_sync(&self) -> broadcast::Receiver<ClipboardSyncMessage> {
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
