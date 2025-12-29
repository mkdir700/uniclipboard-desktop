use std::net::SocketAddr;
use std::sync::Arc;

use crate::infrastructure::{
    connection::connection_manager::ConnectionManager,
    web::handlers::message_handler::MessageSource,
};

use super::{
    client::WebSocketMessage,
    websocket_message::WebSocketMessageHandler,
};
use log::error;
use log::info;
use warp::ws::WebSocket;

pub struct WebSocketHandler {
    message_handler: Arc<WebSocketMessageHandler>,
    connection_manager: Arc<ConnectionManager>,
}

impl WebSocketHandler {
    pub fn new(
        websocket_message_handler: Arc<WebSocketMessageHandler>,
        connection_manager: Arc<ConnectionManager>,
    ) -> Self {
        Self {
            message_handler: websocket_message_handler,
            connection_manager,
        }
    }

    pub async fn client_connected(&self, ws: WebSocket, addr: Option<SocketAddr>) {
        let Some(addr) = addr else {
            error!("Client connected but addr is None");
            return;
        };

        // 使用统一连接管理器处理入站连接
        if let Err(e) = self
            .connection_manager
            .unified
            .handle_incoming_connection(ws, addr)
            .await
        {
            error!("Failed to handle incoming connection from {}: {}", addr, e);
            return;
        }

        info!("Client [{}] connected", addr);
    }

    pub async fn client_disconnected(&self, addr: SocketAddr) {
        info!("Client [{}] disconnected", addr);
        self.connection_manager
            .remove_connection(MessageSource::IpPort(addr))
            .await;
    }
}
