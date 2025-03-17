use crate::config::Setting as Config;
use crate::infrastructure::connection::connection_manager::ConnectionManager;
use crate::interface::RemoteClipboardSync;
use crate::message::{ClipboardSyncMessage, WebSocketMessage};
use anyhow::Result;
use async_trait::async_trait;
use log::info;
use std::sync::Arc;
use std::time::Duration;
use crate::infrastructure::web::WebSocketMessageHandler;

#[derive(Clone)]
pub struct WebSocketSync {
    websocket_message_handler: Arc<WebSocketMessageHandler>,
    connection_manager: Arc<ConnectionManager>,
    config: Config,
}

impl WebSocketSync {
    pub fn new(
        websocket_message_handler: Arc<WebSocketMessageHandler>,
        connection_manager: Arc<ConnectionManager>,
    ) -> Self {
        Self {
            websocket_message_handler,
            connection_manager,
            config: Config::get_instance(),
        }
    }

    /// 创建一个新的 WebSocketSync 实例，使用指定的配置
    pub fn with_config(
        websocket_message_handler: Arc<WebSocketMessageHandler>,
        connection_manager: Arc<ConnectionManager>,
        config: Config,
    ) -> Self {
        Self {
            websocket_message_handler,
            connection_manager,
            config,
        }
    }
}

#[async_trait]
impl RemoteClipboardSync for WebSocketSync {
    /// 暂停远程同步
    ///
    /// 仅客户端会被暂停，服务端不会被暂停
    async fn pause(&self) -> Result<()> {
        self.stop().await
    }
    async fn resume(&self) -> Result<()> {
        self.start().await
    }

    /// 向所有已连接的客户端广播消息
    async fn push(&self, message: ClipboardSyncMessage) -> Result<()> {
        let message = WebSocketMessage::ClipboardSync(message);
        self.connection_manager.broadcast(&message, &None).await?;
        Ok(())
    }

    /// 从任意已连接的客户端接收剪贴板同步消息
    async fn pull(&self, timeout: Option<Duration>) -> Result<ClipboardSyncMessage> {
        let _ = timeout;
        // TODO: 从连接管理器中获取到消息，这个逻辑不太合理，需要优化
        let mut rx = self.connection_manager.subscribe_clipboard_sync().await;
        let clip_message = match rx.recv().await {
            Ok(msg) => msg,
            Err(e) => return Err(e.into()),
        };
        info!("A new clipboard message received: {}", clip_message);
        Ok(clip_message)
    }

    async fn sync(&self) -> Result<()> {
        // 在这个简单的实现中，sync 可以是一个 no-op
        // 或者可以发送一个特殊的同步消息
        Ok(())
    }

    /// 向已知的设备发起 ws 连接
    ///
    /// 并向其他设备同步当前设备已知的设备列表
    async fn start(&self) -> Result<()> {
        // 处理 outgoing 连接的消息
        self.websocket_message_handler
            .start_handle_outgoing_connections_messages()
            .await;

        Ok(())
    }

    /// 断开所有已连接的客户端
    async fn stop(&self) -> Result<()> {
        self.connection_manager.outgoing.disconnect_all().await;
        Ok(())
    }
}
