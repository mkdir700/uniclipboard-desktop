use anyhow::Result;
use libp2p::identity::Keypair;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::config::Setting;
use crate::infrastructure::clipboard::LocalClipboard;
use crate::infrastructure::connection::connection_manager::ConnectionManager;
use crate::infrastructure::p2p::pairing::PairingCommand;
use crate::infrastructure::p2p::{NetworkCommand, NetworkEvent, PairingManager};
use crate::infrastructure::storage::file_storage::FileStorageManager;
use crate::infrastructure::storage::peer_storage::PeerStorage;
use crate::infrastructure::storage::record_manager::ClipboardRecordManager;
use crate::infrastructure::sync::Libp2pSync;
use crate::infrastructure::sync::RemoteSyncManager;
use crate::infrastructure::sync::WebSocketSync;
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
    pub webserver: Arc<WebServer>,
    pub record_manager: Arc<ClipboardRecordManager>,
    pub file_storage: Arc<FileStorageManager>,
    /// P2P network command sender
    #[allow(dead_code)]
    pub p2p_command_tx: Option<mpsc::Sender<NetworkCommand>>,
    /// P2P pairing manager
    #[allow(dead_code)]
    pub pairing_manager: Option<Arc<PairingManager>>,
    /// P2P sync instance
    #[allow(dead_code)]
    pub p2p_sync: Option<Arc<Libp2pSync>>,
    /// Local peer ID (libp2p PeerId)
    #[allow(dead_code)]
    pub local_peer_id: Option<String>,
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

    /// 构建 P2P 组件
    ///
    /// 初始化 libp2p 网络、配对管理器和 P2P 同步
    async fn build_p2p_components(
        &self,
        device_name: String,
    ) -> Result<(
        mpsc::Sender<NetworkCommand>,
        Arc<PairingManager>,
        Arc<Libp2pSync>,
        String,
    )> {
        // Create channels for P2P network
        let (command_tx, command_rx) = mpsc::channel(100);
        let (event_tx, event_rx) = mpsc::channel(100);
        // Create channel for Pairing commands
        let (_pairing_cmd_tx, pairing_cmd_rx) = mpsc::channel(100);

        // Generate libp2p keypair
        let local_key = Keypair::generate_ed25519();
        let local_peer_id = libp2p::PeerId::from(local_key.public()).to_string();

        // Create pairing manager
        let pairing_device_name = device_name.clone();
        let pairing_manager = Arc::new(PairingManager::new(
            command_tx.clone(),
            event_tx.clone(),
            pairing_cmd_rx,
            pairing_device_name,
        ));

        // Spawn network manager task
        let device_name_clone = device_name.clone();
        tokio::spawn(async move {
            let mut network_manager = crate::infrastructure::p2p::NetworkManager::new(
                command_rx,
                event_tx,
                local_key,
                device_name_clone,
            )
            .await
            .expect("Failed to create NetworkManager");

            log::info!("Starting P2P network manager");
            network_manager.run().await;
        });

        // Create PeerStorage
        let peer_storage = Arc::new(PeerStorage::new().expect("Failed to create PeerStorage"));

        // Create P2P sync instance (after network manager is spawned)
        let p2p_sync = Arc::new(Libp2pSync::new(
            command_tx.clone(),
            device_name.clone(),
            local_peer_id.clone(),
            peer_storage,
        ));

        Ok((command_tx, pairing_manager, p2p_sync, local_peer_id))
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
        let webserver = Arc::new(WebServer::new(
            SocketAddr::new(
                // self.user_setting.webserver_addr.unwrap().parse()?,
                "0.0.0.0".parse().unwrap(),
                self.user_setting.network.webserver_port,
            ),
            websocket_handler.clone(),
        ));

        remote_sync_manager
            .set_sync_handler(websocket_sync.clone())
            .await;

        let clipboard_history = Arc::new(ClipboardRecordManager::new(
            self.user_setting.storage.max_history_items as usize,
        ));
        let file_storage =
            Arc::new(FileStorageManager::new().expect("Failed to create FileStorageManager"));

        // Get device name for P2P - use system hostname or default
        // Get device name for P2P - use settings or system hostname
        let device_name = if !self.user_setting.general.device_name.is_empty() {
            self.user_setting.general.device_name.clone()
        } else {
            gethostname::gethostname()
                .to_str()
                .unwrap_or("Unknown Device")
                .to_string()
        };

        // Build P2P components
        let (p2p_command_tx, pairing_manager, p2p_sync, local_peer_id) =
            self.build_p2p_components(device_name).await?;

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
            p2p_command_tx: Some(p2p_command_tx),
            pairing_manager: Some(pairing_manager),
            p2p_sync: Some(p2p_sync),
            local_peer_id: Some(local_peer_id),
        })
    }
}
