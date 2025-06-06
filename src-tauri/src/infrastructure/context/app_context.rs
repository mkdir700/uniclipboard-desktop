use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::config::Setting;
use crate::infrastructure::clipboard::LocalClipboard;
use crate::infrastructure::connection::connection_manager::ConnectionManager;
use crate::infrastructure::storage::record_manager::ClipboardRecordManager;
use crate::infrastructure::sync::RemoteSyncManager;
use crate::infrastructure::sync::WebSocketSync;
use crate::infrastructure::storage::file_storage::FileStorageManager;
use crate::infrastructure::web::WebServer;
use crate::infrastructure::web::WebSocketHandler;
use crate::infrastructure::web::WebSocketMessageHandler;
use crate::interface::RemoteSyncManagerTrait;

/// 应用上下文
/// 
/// 包含应用运行所需的所有核心组件实例
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
    pub record_manager: Arc<ClipboardRecordManager>,
    pub file_storage: Arc<FileStorageManager>,
}

/// 应用上下文构建器
///
/// 负责初始化和配置应用上下文的所有组件
pub struct AppContextBuilder {
    user_setting: Setting,
}

impl AppContextBuilder {
    /// 创建新的应用上下文构建器
    pub fn new(user_setting: Setting) -> Self {
        Self { user_setting }
    }

    /// 构建应用上下文
    pub async fn build(self) -> Result<AppContext> {
        let local_clipboard = Arc::new(LocalClipboard::with_user_setting(
            self.user_setting.clone(),
        )?);
        let remote_sync_manager = Arc::new(RemoteSyncManager::with_user_setting(
            self.user_setting.clone(),
        ));
        let connection_manager = Arc::new(ConnectionManager::with_user_setting(
            self.user_setting.clone(),
        ));
        let websocket_message_handler =
            Arc::new(WebSocketMessageHandler::new(connection_manager.clone()));
        let websocket_handler = Arc::new(WebSocketHandler::new(
            websocket_message_handler.clone(),
            connection_manager.clone(),
        ));
        let websocket_sync = Arc::new(WebSocketSync::with_config(
            websocket_message_handler.clone(),
            connection_manager.clone(),
            self.user_setting.clone(),
        ));
        let webserver = WebServer::new(
            SocketAddr::new(
                // self.user_setting.webserver_addr.unwrap().parse()?,
                "0.0.0.0".parse().unwrap(),
                self.user_setting.network.webserver_port,
            ),
            websocket_handler.clone(),
        );

        remote_sync_manager
            .set_sync_handler(websocket_sync.clone())
            .await;

        let clipboard_history = Arc::new(ClipboardRecordManager::new(
            self.user_setting.storage.max_history_items as usize,
        ));
        let file_storage = Arc::new(FileStorageManager::new().expect("Failed to create FileStorageManager"));

        // 返回 AppContext 实例
        Ok(AppContext {
            local_clipboard,
            remote_sync_manager,
            connection_manager,
            websocket_message_handler,
            websocket_handler,
            websocket_sync,
            webserver,
            record_manager: clipboard_history,
            file_storage,
        })
    }
}
