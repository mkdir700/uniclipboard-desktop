use crate::application::device_service::GLOBAL_DEVICE_MANAGER;
use crate::domain::device::Device;
use crate::infrastructure::network::WebSocketClient;
use crate::message::WebSocketMessage;
use anyhow::Result;
use log::{error, info};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
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
}

impl OutgoingConnectionManager {
    pub fn new() -> Self {
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
