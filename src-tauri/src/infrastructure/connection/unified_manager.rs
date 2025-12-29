//! 统一连接管理器
//!
//! 合并了 OutgoingConnectionManager 和 IncomingConnectionManager 的功能，
//! 统一管理所有 WebSocket 连接（主动连接和被动接受）。

/// 健康检查间隔（秒）
const HEALTH_CHECK_INTERVAL_SECS: u64 = 5;

use crate::application::device_service::GLOBAL_DEVICE_MANAGER;
use crate::config::Setting;
use crate::domain::device::Device;
use crate::infrastructure::network::WebSocketClient;
use crate::infrastructure::web::handlers::client::IncommingWebsocketClient;
use crate::message::WebSocketMessage;
use anyhow::Result;
use futures::StreamExt;
use log::{error, info, warn};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::http::Uri;
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;
use warp::ws::Message as WarpMessage;

/// 连接方向
#[derive(Clone)]
pub enum ConnectionDirection {
    /// 主动连接（作为客户端）
    Outgoing(Arc<RwLock<WebSocketClient>>),
    /// 被动接受（作为服务端）
    Incoming(Arc<IncommingWebsocketClient>),
}

/// 统一连接
pub struct UnifiedConnection {
    pub device_id: String,
    pub direction: ConnectionDirection,
    pub connected: Arc<std::sync::atomic::AtomicBool>,
    pub health_check_task: JoinHandle<()>,
}

/// 统一连接管理器
#[derive(Clone)]
pub struct UnifiedConnectionManager {
    /// 当前设备的 ID
    my_device_id: String,

    /// 所有连接
    /// key: device_id, value: UnifiedConnection
    connections: Arc<RwLock<HashMap<String, UnifiedConnection>>>,

    /// 待处理的入站连接（等待识别 device_id）
    /// key: ip:port, value: IncommingWebsocketClient
    pending_connections: Arc<RwLock<HashMap<String, Arc<IncommingWebsocketClient>>>>,

    /// 出站连接消息发送器（用于订阅出站连接的消息）
    messages_tx: Arc<broadcast::Sender<(String, TungsteniteMessage)>>,

    /// 配置
    user_setting: Setting,
}

impl UnifiedConnectionManager {
    pub fn new(user_setting: Setting) -> Self {
        let my_device_id = user_setting.get_device_id().clone();
        let (messages_tx, _) = broadcast::channel(20);
        Self {
            my_device_id,
            connections: Arc::new(RwLock::new(HashMap::new())),
            pending_connections: Arc::new(RwLock::new(HashMap::new())),
            messages_tx: Arc::new(messages_tx),
            user_setting,
        }
    }

    /// 订阅出站连接消息
    pub async fn subscribe_outgoing_connections_messages(
        &self,
    ) -> broadcast::Receiver<(String, TungsteniteMessage)> {
        self.messages_tx.subscribe()
    }

    /// 判断是否应该作为客户端发起连接
    /// 规则：设备 ID 较小的一方作为客户端
    pub fn should_initiate_connection(&self, peer_device_id: &str) -> bool {
        self.my_device_id.as_str() < peer_device_id
    }

    /// 连接到指定设备
    pub async fn connect_device(&self, device: &Device) -> Result<()> {
        // 检查是否应该发起连接
        if !self.should_initiate_connection(&device.id) {
            info!("等待设备 {} 发起连接（对方 ID 更大）", device.id);
            return Ok(());  // 返回 Ok 表示已正确处理（跳过连接）
        }

        // 检查是否已连接
        if self.is_connected(&device.id).await {
            warn!("设备 {} 已连接，跳过", device.id);
            return Ok(());
        }

        // 构建 URI
        let uri = format!(
            "ws://{}:{}/ws",
            device.ip.as_ref().ok_or_else(|| anyhow::anyhow!("Missing IP"))?,
            device.server_port.as_ref().ok_or_else(|| anyhow::anyhow!("Missing port"))?
        )
        .parse::<Uri>()?;

        // 创建客户端并连接
        let mut client = WebSocketClient::with_user_setting(uri, self.user_setting.clone());

        // 连接失败时，不设置设备状态
        client.connect().await?;
        client.register(None).await?;
        client.sync_device_list().await?;

        // 设置设备在线
        if let Err(e) = GLOBAL_DEVICE_MANAGER.set_online(&device.id) {
            error!("Failed to set device {} online: {}", device.id, e);
            // 注意：此处不返回错误，因为连接已成功建立
        }

        // 添加到连接表
        self.add_outgoing_connection(device.id.clone(), Arc::new(RwLock::new(client))).await;

        info!("成功连接到设备 {} (Outgoing)", device.id);
        Ok(())
    }

    /// 连接到对等设备（通过 IP 和端口）
    pub async fn connect_with_peer_device(
        &self,
        peer_device_addr: &str,
        peer_device_port: u16,
    ) -> Result<()> {
        if peer_device_addr.is_empty() || peer_device_port == 0 {
            return Err(anyhow::anyhow!("Peer device address or port is not set"));
        }

        let device_id = format!("{}:{}", peer_device_addr, peer_device_port);
        let uri = format!("ws://{}:{}/ws", peer_device_addr, peer_device_port)
            .parse::<Uri>()
            .map_err(|_| anyhow::anyhow!("Invalid URI: {}", device_id))?;

        let mut client = WebSocketClient::with_user_setting(uri, self.user_setting.clone());
        client.connect().await?;
        client.register(None).await?;
        client.sync_device_list().await?;

        // 添加到连接表
        self.add_outgoing_connection(device_id.clone(), Arc::new(RwLock::new(client))).await;

        info!("成功连接到对等设备 {} (Outgoing)", device_id);
        Ok(())
    }

    /// 处理入站连接
    pub async fn handle_incoming_connection(
        &self,
        ws: warp::ws::WebSocket,
        addr: SocketAddr,
    ) -> Result<()> {
        let ip_port = format!("{}:{}", addr.ip(), addr.port());

        // 创建客户端
        let mut client = IncommingWebsocketClient::new(addr, ws);
        client.start().await?;

        // 临时存储，等待 Register 消息
        self.pending_connections
            .write()
            .await
            .insert(ip_port.clone(), Arc::new(client));

        info!("收到来自 {} 的入站连接，等待注册", ip_port);
        Ok(())
    }

    /// 处理设备注册消息
    pub async fn handle_register(&self, device_id: String, addr: SocketAddr) -> Result<()> {
        let ip_port = format!("{}:{}", addr.ip(), addr.port());

        // 从 pending_connections 中获取客户端
        let client = self
            .pending_connections
            .write()
            .await
            .remove(&ip_port)
            .ok_or_else(|| anyhow::anyhow!("Pending connection not found: {}", ip_port))?;

        // 设置设备在线
        if let Err(e) = GLOBAL_DEVICE_MANAGER.set_online(&device_id) {
            error!("Failed to set device {} online: {}", device_id, e);
        }

        // 检查连接方向
        if self.should_initiate_connection(&device_id) {
            // 我应该作为客户端，关闭这个 incoming 连接
            warn!("拒绝来自 {} 的入站连接（我应该作为客户端）", device_id);
            let _ = client.stop().await;
            return Ok(());
        }

        // 检查是否已连接
        if self.is_connected(&device_id).await {
            warn!("设备 {} 已连接，拒绝重复连接", device_id);
            let _ = client.stop().await;
            return Ok(());
        }

        // 添加到连接表
        self.add_incoming_connection(device_id.clone(), client).await;

        info!("接受来自 {} 的入站连接 (Incoming)", device_id);
        Ok(())
    }

    /// 添加出站连接
    async fn add_outgoing_connection(
        &self,
        device_id: String,
        client: Arc<RwLock<WebSocketClient>>,
    ) {
        let connected = Arc::new(std::sync::atomic::AtomicBool::new(true));

        // 启动消息转发
        let forward_message_task = self.start_forward_outgoing_messages(
            device_id.clone(),
            client.clone(),
        ).await;

        // 启动健康检查
        let health_check_task =
            self.start_outgoing_health_check(device_id.clone(), client.clone(), connected.clone())
                .await;

        let connection = UnifiedConnection {
            device_id: device_id.clone(),
            direction: ConnectionDirection::Outgoing(client),
            connected,
            health_check_task: tokio::spawn(async move {
                // 等待消息转发和健康检查任务完成
                let _ = tokio::join!(forward_message_task, health_check_task);
            }),
        };

        self.connections.write().await.insert(device_id, connection);
    }

    /// 启动出站连接的消息转发
    async fn start_forward_outgoing_messages(
        &self,
        device_id: String,
        client: Arc<RwLock<WebSocketClient>>,
    ) -> JoinHandle<()> {
        let outgoing_connections_message_tx = self.messages_tx.clone();
        tokio::spawn(async move {
            let mut message_rx = client.read().await.subscribe();
            loop {
                let message = message_rx.recv().await;
                if let Ok(message) = message {
                    let _ = outgoing_connections_message_tx.send((device_id.clone(), message));
                }
            }
        })
    }

    /// 添加入站连接
    async fn add_incoming_connection(&self, device_id: String, client: Arc<IncommingWebsocketClient>) {
        let connected = Arc::new(std::sync::atomic::AtomicBool::new(true));

        // 注意：IncommingWebsocketClient 内部已有健康检查，这里不需要额外启动

        let connection = UnifiedConnection {
            device_id: device_id.clone(),
            direction: ConnectionDirection::Incoming(client),
            connected: connected.clone(),
            health_check_task: tokio::spawn(async move {
                // 空任务，健康检查由 IncommingWebsocketClient 内部处理
                tokio::time::sleep(Duration::from_secs(u64::MAX)).await;
            }),
        };

        self.connections.write().await.insert(device_id, connection);
    }

    /// 启动出站连接的健康检查
    async fn start_outgoing_health_check(
        &self,
        device_id: String,
        client: Arc<RwLock<WebSocketClient>>,
        connected: Arc<std::sync::atomic::AtomicBool>,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(HEALTH_CHECK_INTERVAL_SECS));

            loop {
                interval.tick().await;

                if !client.read().await.is_connected() {
                    info!("设备 {} 健康检查失败，标记为断开", device_id);
                    connected.store(false, std::sync::atomic::Ordering::Relaxed);
                    break;
                }
            }
        })
    }

    /// 移除连接
    pub async fn remove_connection(&self, device_id: &str) {
        if let Some(connection) = self.connections.write().await.remove(device_id) {
            connection.health_check_task.abort();
            connection.connected.store(false, std::sync::atomic::Ordering::Relaxed);
        }

        // 设置设备离线
        if let Err(e) = GLOBAL_DEVICE_MANAGER.set_offline(device_id) {
            error!("Failed to set device {} offline: {}", device_id, e);
        }

        info!("移除设备 {} 的连接", device_id);
    }

    /// 移除连接（通过 IP:Port）
    pub async fn remove_connection_by_addr(&self, addr: SocketAddr) {
        let ip_port = format!("{}:{}", addr.ip(), addr.port());

        // 查找对应的 device_id
        let device_id = {
            let connections = self.connections.read().await;
            let mut found_id = None;
            for (id, conn) in connections.iter() {
                if let ConnectionDirection::Incoming(client) = &conn.direction {
                    if client.id() == ip_port {
                        found_id = Some(id.clone());
                        break;
                    }
                }
            }
            found_id
        };

        if let Some(device_id) = device_id {
            self.remove_connection(&device_id).await;
        } else {
            warn!("未找到对应 IP:Port {} 的连接", ip_port);
        }
    }

    /// 检查是否已连接
    pub async fn is_connected(&self, device_id: &str) -> bool {
        let connections = self.connections.read().await;
        if let Some(connection) = connections.get(device_id) {
            connection.connected.load(std::sync::atomic::Ordering::Relaxed)
        } else {
            false
        }
    }

    /// 获取连接数量
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// 广播消息到所有连接
    pub async fn broadcast(
        &self,
        message: &WebSocketMessage,
        excludes: &Option<Vec<String>>,
    ) -> Result<()> {
        let connections = self.connections.read().await;
        let mut errors = Vec::new();

        for (device_id, connection) in connections.iter() {
            // 检查是否需要排除
            if let Some(exclude_ids) = excludes {
                if exclude_ids.contains(device_id) {
                    continue;
                }
            }

            // 根据连接方向发送消息
            let result = match &connection.direction {
                ConnectionDirection::Outgoing(client) => {
                    client.read().await.send_with_websocket_message(message).await
                }
                ConnectionDirection::Incoming(client) => {
                    let message_str = serde_json::to_string(message)?;
                    let warp_message = WarpMessage::text(message_str);
                    client.send(warp_message).await
                }
            };

            if let Err(e) = result {
                error!("发送消息到 {} 失败: {}", device_id, e);
                errors.push((device_id.clone(), e));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("广播消息部分失败: {:?}", errors))
        }
    }

    /// 断开所有连接
    pub async fn disconnect_all(&self) {
        info!("断开所有连接");

        let connections: Vec<_> = self.connections.read().await.keys().cloned().collect();

        for device_id in connections {
            self.remove_connection(&device_id).await;
        }
    }
}
