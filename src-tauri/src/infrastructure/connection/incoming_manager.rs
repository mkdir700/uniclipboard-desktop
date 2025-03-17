use crate::infrastructure::web::handlers::client::IncommingWebsocketClient;
use crate::message::WebSocketMessage;
use anyhow::Result;
use futures::future::join_all;
use log::{debug, error, info};
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::ws::Message as WarpMessage;

use super::{Clients, IpPort};

#[derive(Clone)]
pub struct IncomingConnectionManager {
    pub connections: Arc<RwLock<Clients>>,
}

impl IncomingConnectionManager {
    pub fn new() -> Self {
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

    pub async fn broadcast(
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
