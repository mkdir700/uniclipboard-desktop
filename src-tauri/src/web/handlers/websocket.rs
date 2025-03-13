use std::net::SocketAddr;
use std::sync::Arc;

use super::client::{IncommingWebsocketClient, WebSocketMessage};
use log::{error, info};
use warp::ws::WebSocket;

use crate::connection::ConnectionManager;
use crate::web::handlers::message_handler::MessageSource;
use crate::web::handlers::websocket_message::WebSocketMessageHandler;

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
        let client = match addr {
            Some(addr) => IncommingWebsocketClient::new(addr, ws),
            None => return,
        };
        match client.start().await {
            Ok(_) => (),
            Err(e) => {
                error!("Failed to initialize client: {}", e);
                return;
            }
        };

        let client_id = client.id();

        let mut rx = client.subscribe_messages();
        {
            let client_id = client.id();
            self.connection_manager
                .incoming
                .add_connection(client)
                .await;
            let count = self.connection_manager.incoming.count().await;
            info!("Client {} connected, current clients: {}", client_id, count);
        }

        info!("Client [{}] connected", client_id);
        loop {
            let result = rx.recv().await;
            let msg = match result {
                Ok(msg) => msg,
                Err(e) => {
                    error!("Error receiving ws message: {}", e);
                    break;
                }
            };
            match msg {
                WebSocketMessage::Message(msg) => {
                    if let Some(addr) = addr {
                        self.message_handler
                            .handle_message(msg, MessageSource::IpPort(addr))
                            .await;
                    } else {
                        error!("Client [{}] connected, but addr is None", client_id);
                    }
                }
                WebSocketMessage::Close => {
                    break;
                }
            }
        }
        if let Some(addr) = addr {
            self.client_disconnected(addr).await;
        }
    }

    async fn client_disconnected(&self, addr: SocketAddr) {
        info!("Client [{}] disconnected", addr);
        self.connection_manager
            .remove_connection(MessageSource::IpPort(addr))
            .await;
    }
}
