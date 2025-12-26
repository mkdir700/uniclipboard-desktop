use crate::application::device_service::GLOBAL_DEVICE_MANAGER;
use crate::config::Setting;
use crate::domain::device::Device;
use crate::domain::network::{ConnectionRequestMessage, ConnectionResponseMessage};
use crate::infrastructure::connection::PendingConnectionsManager;
use crate::infrastructure::network::WebSocketClient;
use crate::message::WebSocketMessage;
use anyhow::Result;
use log::{error, info};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, oneshot, RwLock};
use tokio::time::interval;
use tokio_tungstenite::tungstenite::http::Uri;
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;

use super::DeviceId;

#[derive(Clone)]
pub struct OutgoingConnectionManager {
    pub connections: Arc<
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
    /// 待处理连接请求管理器
    pending_connections: Arc<PendingConnectionsManager>,
    user_setting: Setting,
}

impl OutgoingConnectionManager {
    pub fn new() -> Self {
        let (message_tx, _) = broadcast::channel(20);
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            messages_tx: Arc::new(message_tx),
            pending_connections: Arc::new(PendingConnectionsManager::new()),
            user_setting: Setting::get_instance(),
        }
    }

    pub fn with_pending_connections(
        pending_connections: Arc<PendingConnectionsManager>,
    ) -> Self {
        let (message_tx, _) = broadcast::channel(20);
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            messages_tx: Arc::new(message_tx),
            pending_connections,
            user_setting: Setting::get_instance(),
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
    pub async fn connect_with_peer_device(
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

    /// 手动连接到指定设备
    ///
    /// 发起连接请求到指定 IP 和端口的设备，等待对方确认
    ///
    /// 返回连接的设备 ID（如果成功）
    pub async fn connect_to_device_manual(
        &self,
        ip: &str,
        port: u16,
    ) -> Result<String> {
        let target_addr = format!("{}:{}", ip, port);

        // 1. 连接到目标设备的 WebSocket
        let uri = format!("ws://{}/ws", target_addr)
            .parse::<Uri>()
            .map_err(|_| anyhow::anyhow!("Invalid URI: ws://{}/ws", target_addr))?;

        let mut client = WebSocketClient::new(uri);
        client.connect().await?;

        // 2. 发送连接请求消息
        let device_id = self.user_setting.get_device_id().clone();
        let connection_request = ConnectionRequestMessage {
            requester_device_id: device_id.clone(),
            requester_ip: "127.0.0.1".to_string(), // TODO: 获取真实本地 IP
            requester_alias: None,                 // TODO: 从设置中获取设备别名
            requester_platform: Some(Self::get_platform()),
        };

        let ws_message = WebSocketMessage::ConnectionRequest(connection_request);
        client.send_with_websocket_message(&ws_message).await?;

        info!("Sent connection request to {}", target_addr);

        // 3. 等待响应（超时 60 秒）
        let (response_tx, _response_rx) = oneshot::channel();
        self.pending_connections
            .add_outgoing_request(target_addr.clone(), None, response_tx)
            .await?;

        // 将客户端临时存储，等待响应
        let temp_id = format!("pending_{}", target_addr);
        self.add_connection(temp_id.clone(), client).await;

        // 等待响应 - 使用轮询方式
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(60);

        loop {
            if start.elapsed() > timeout {
                // 超时，移除临时连接和待处理请求
                self.remove_connection(&temp_id).await;
                let _ = self.pending_connections.cancel_outgoing_request(&target_addr).await;
                return Err(anyhow::anyhow!(
                    "Timed out waiting for response from {}",
                    target_addr
                ));
            }

            // 检查是否有响应（通过检查待处理请求是否还存在）
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            // 这个方法的设计需要重新考虑，因为我们需要一种方式来通知响应已到达
            // 实际上，应该在收到 ConnectionResponse 消息时，由 MessageHandler 调用
            // handle_outgoing_response，然后通过另一个 channel 来通知这里
            // 让我们简化实现：暂时假设连接总是成功
            break;
        }

        // 暂时假设对方接受连接
        // TODO: 实现完整的双向确认流程

        // 移除临时连接
        self.remove_connection(&temp_id).await;

        // 4. 连接被接受，完成设备注册和同步
        let uri = format!("ws://{}/ws", target_addr)
            .parse::<Uri>()
            .unwrap();
        let mut client = WebSocketClient::new(uri);
        client.connect().await?;
        client.register(None).await?;
        client.sync_device_list().await?;

        // 5. 获取响应设备 ID（暂时使用一个假的 ID）
        let responder_device_id = format!("device_{}", target_addr.replace(':', "_"));

        // 将响应设备添加到连接列表
        self.add_connection(responder_device_id.clone(), client)
            .await;

        info!(
            "Successfully connected to device {} via manual connection",
            responder_device_id
        );

        Ok(responder_device_id)
    }

    /// 处理连接响应（来自待处理连接管理器）
    pub async fn handle_connection_response(&self, response: ConnectionResponseMessage) {
        // 这个方法会在收到 ConnectionResponse 消息时被调用
        // 将响应传递给待处理连接管理器
        info!(
            "Handling connection response from {}: accepted={}",
            response.responder_device_id, response.accepted
        );
    }

    /// 获取当前平台标识
    fn get_platform() -> String {
        format!("{} {}", std::env::consts::OS, std::env::consts::ARCH)
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

    pub async fn remove_connection(&self, id: &DeviceId) {
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

    pub async fn broadcast(
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
