use crate::connection::ConnectionManager;
use crate::message::WebSocketMessage;
use crate::web::handlers::message_handler::{MessageHandler, MessageSource};
use log::{debug, error};
use std::sync::Arc;
use tokio_tungstenite::tungstenite::protocol::Message as TungsteniteMessage;
use warp::ws::Message as WarpMessage;

#[derive(Clone)]
pub struct WebSocketMessageHandler {
    pub connection_manager: Arc<ConnectionManager>,
    pub message_handler: Arc<MessageHandler>,
}

impl WebSocketMessageHandler {
    pub fn new(connection_manager: Arc<ConnectionManager>) -> Self {
        let message_handler = Arc::new(MessageHandler::new(connection_manager.clone()));
        Self {
            connection_manager,
            message_handler,
        }
    }

    /// 开启一个异步任务，从 outgoing_connections 中接收消息并处理
    /// 动态的添加和删除 connections
    pub async fn start_handle_outgoing_connections_messages(&self) {
        let mut rx = self
            .connection_manager
            .outgoing
            .subscribe_outgoing_connections_message()
            .await;
        let self_clone = Arc::new(self.clone());

        tokio::spawn(async move {
            loop {
                let message = rx.recv().await;
                if let Ok((device_id, message)) = message {
                    match message {
                        TungsteniteMessage::Text(text) => {
                            self_clone
                                .handle_message(
                                    WarpMessage::text(text),
                                    MessageSource::DeviceId(device_id),
                                )
                                .await;
                        }
                        _ => {}
                    }
                }
            }
        });
    }

    pub async fn handle_message(&self, msg: WarpMessage, message_source: MessageSource) {
        if msg.is_text() {
            if let Ok(text) = msg.to_str() {
                if text == "connect" {
                    return;
                }
                match serde_json::from_str::<WebSocketMessage>(text) {
                    Ok(websocket_message) => match websocket_message {
                        WebSocketMessage::ClipboardSync(data) => {
                            self.message_handler
                                .handle_clipboard_sync(data, message_source)
                                .await;
                        }
                        WebSocketMessage::DeviceListSync(data) => {
                            self.message_handler
                                .handle_device_list_sync(data, message_source)
                                .await;
                        }
                        WebSocketMessage::Register(register_device_message) => {
                            if let MessageSource::IpPort(addr) = message_source {
                                debug!(
                                    "Received register message: {:?}, source: {:?}",
                                    register_device_message, addr
                                );
                                self.message_handler
                                    .handle_register(register_device_message, addr)
                                    .await;
                            } else {
                                error!("Register message source is not IpPort");
                            }
                        }
                        WebSocketMessage::Unregister(device_id) => {
                            self.message_handler.handle_unregister(device_id).await;
                        }
                    },
                    Err(e) => {
                        error!(
                            "Error parsing WebSocket message: {:?}, message: {}",
                            e, text
                        );
                    }
                }
            }
        }
    }
}
