use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::clipboard::LocalClipboard;
use crate::config::Config;
use crate::connection::ConnectionManager;
use crate::remote_sync::{RemoteSyncManager, RemoteSyncManagerTrait, WebSocketSync};
use crate::web::server::WebServer;
use crate::web::{WebSocketHandler, WebSocketMessageHandler};

pub struct AppContext {
    pub local_clipboard: Arc<LocalClipboard>,
    pub remote_sync_manager: Arc<RemoteSyncManager>,
    #[allow(unused)]
    pub connection_manager: Arc<ConnectionManager>,
    #[allow(unused)]
    pub websocket_message_handler: Arc<WebSocketMessageHandler>,
    #[allow(unused)]
    pub websocket_handler: Arc<WebSocketHandler>,
    #[allow(unused)]
    pub websocket_sync: Arc<WebSocketSync>,
    pub webserver: WebServer,
}

pub struct AppContextBuilder {
    config: Config,
}

impl AppContextBuilder {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn build(self) -> Result<AppContext> {
        let local_clipboard = Arc::new(LocalClipboard::with_config(self.config.clone())?);
        let remote_sync_manager = Arc::new(RemoteSyncManager::with_config(self.config.clone()));
        let connection_manager = Arc::new(ConnectionManager::with_config(self.config.clone()));
        let websocket_message_handler =
            Arc::new(WebSocketMessageHandler::new(connection_manager.clone()));
        let websocket_handler = Arc::new(WebSocketHandler::new(
            websocket_message_handler.clone(),
            connection_manager.clone(),
        ));
        let websocket_sync = Arc::new(WebSocketSync::with_config(
            websocket_message_handler.clone(),
            connection_manager.clone(),
            self.config.clone(),
        ));
        let webserver = WebServer::new(
            SocketAddr::new(
                self.config.webserver_addr.unwrap().parse()?,
                self.config.webserver_port.unwrap(),
            ),
            websocket_handler.clone(),
        );

        remote_sync_manager.set_sync_handler(websocket_sync.clone()).await;

        // 返回 AppContext 实例
        Ok(AppContext {
            local_clipboard,
            remote_sync_manager,
            connection_manager,
            websocket_message_handler,
            websocket_handler,
            websocket_sync,
            webserver,
        })
    }
}
