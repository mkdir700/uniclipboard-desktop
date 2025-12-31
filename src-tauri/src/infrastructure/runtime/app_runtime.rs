//! Application runtime
//!
//! Single owner of all core application components.
use anyhow::Result;
use log::{error, info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

use crate::config::Setting;
use crate::infrastructure::clipboard::LocalClipboard;
use crate::infrastructure::connection::connection_manager::ConnectionManager;
use crate::infrastructure::p2p::pairing::PairingCommand;
use crate::infrastructure::runtime::p2p_runtime::P2PRuntime;
use crate::infrastructure::runtime::{
    AppRuntimeHandle, ClipboardCommand, ConnectionCommand, P2PCommand,
};
use crate::infrastructure::storage::file_storage::FileStorageManager;
use crate::infrastructure::storage::record_manager::ClipboardRecordManager;
use crate::infrastructure::sync::RemoteSyncManager;
use crate::infrastructure::uniclipboard::ClipboardSyncService;
use crate::infrastructure::web::handlers::websocket::WebSocketHandler;
use crate::infrastructure::web::handlers::websocket_message::WebSocketMessageHandler;
use crate::infrastructure::web::WebServer;
use crate::interface::{RemoteClipboardSync, RemoteSyncManagerTrait};

/// Application runtime - single owner of all core components
pub struct AppRuntime {
    clipboard_service: ClipboardSyncService,
    p2p_runtime: Arc<P2PRuntime>,
    webserver: WebServer,
    connection_manager: Arc<ConnectionManager>,
    config: Arc<Setting>,
    is_running: Arc<AtomicBool>,
    // Channels for commands from Tauri handlers
    clipboard_cmd_rx: Option<mpsc::Receiver<ClipboardCommand>>,
    p2p_cmd_rx: Option<mpsc::Receiver<P2PCommand>>,
    connection_cmd_rx: Option<mpsc::Receiver<ConnectionCommand>>,
    // Stored senders to create handles
    clipboard_cmd_tx: mpsc::Sender<ClipboardCommand>,
    p2p_cmd_tx: mpsc::Sender<P2PCommand>,
    connection_cmd_tx: mpsc::Sender<ConnectionCommand>,
}

impl AppRuntime {
    /// Create a new application runtime
    pub async fn new(
        user_setting: Setting,
        device_id: String,
        device_name: String,
    ) -> Result<Self> {
        let config = Arc::new(user_setting.clone());

        // 1. Initialize core managers
        let file_storage = FileStorageManager::new()?;
        let record_manager =
            ClipboardRecordManager::new(user_setting.storage.max_history_items as usize);

        // 2. Initialize Platform Clipboard
        let clipboard = Arc::new(LocalClipboard::with_user_setting(user_setting.clone())?);

        // 3. Initialize P2P Runtime
        let p2p_runtime = Arc::new(P2PRuntime::new(device_name, config.clone()).await?);

        // 4. Initialize Connection Manager (Legacy/WebSocket)
        let connection_manager =
            Arc::new(ConnectionManager::with_user_setting(user_setting.clone()));

        // Initialize RemoteSyncManager
        let remote_sync_manager =
            Arc::new(RemoteSyncManager::with_user_setting(user_setting.clone()));

        // Set P2P Sync as default handler
        remote_sync_manager
            .set_sync_handler(p2p_runtime.p2p_sync())
            .await;

        // 5. Initialize Web Server
        let websocket_message_handler =
            Arc::new(WebSocketMessageHandler::new(connection_manager.clone()));
        let websocket_handler = Arc::new(WebSocketHandler::new(
            websocket_message_handler,
            connection_manager.clone(),
        ));
        let web_addr = std::net::SocketAddr::new(
            "0.0.0.0".parse().unwrap(),
            user_setting.network.webserver_port,
        );
        let webserver = WebServer::new(web_addr, websocket_handler);

        // 6. Initialize ClipboardSyncService
        // Use RemoteSyncManager as the sync interface
        let remote_sync = remote_sync_manager.clone() as Arc<dyn RemoteClipboardSync>;
        let clipboard_service = ClipboardSyncService::new(
            device_id,
            clipboard,
            remote_sync,
            record_manager,
            file_storage,
        );

        // Create command channels
        let (clipboard_cmd_tx, clipboard_cmd_rx) = mpsc::channel(100);
        let (p2p_cmd_tx, p2p_cmd_rx) = mpsc::channel(100);
        let (connection_cmd_tx, connection_cmd_rx) = mpsc::channel(100);

        Ok(Self {
            clipboard_service,
            p2p_runtime,
            webserver,
            connection_manager,
            config,
            is_running: Arc::new(AtomicBool::new(false)),
            clipboard_cmd_rx: Some(clipboard_cmd_rx),
            p2p_cmd_rx: Some(p2p_cmd_rx),
            connection_cmd_rx: Some(connection_cmd_rx),
            clipboard_cmd_tx,
            p2p_cmd_tx,
            connection_cmd_tx,
        })
    }

    /// Start the application runtime
    pub async fn start(mut self) -> Result<()> {
        if self.is_running.load(Ordering::SeqCst) {
            return Ok(());
        }
        self.is_running.store(true, Ordering::SeqCst);
        info!("Starting AppRuntime...");

        // Start components
        // 1. Connection Manager
        self.connection_manager.start().await?;

        // 2. Web Server
        let webserver = self.webserver; // Move out
        tokio::spawn(async move {
            if let Err(e) = webserver.run().await {
                error!("Web server error: {}", e);
            }
        });

        // 3. ClipboardSyncService
        self.clipboard_service.start().await?;

        // 4. Command Loop
        let mut clipboard_cmd_rx = self
            .clipboard_cmd_rx
            .take()
            .expect("Clipboard RX channel missing");
        let mut p2p_cmd_rx = self.p2p_cmd_rx.take().expect("P2P RX channel missing");
        let mut connection_cmd_rx = self
            .connection_cmd_rx
            .take()
            .expect("Connection RX channel missing");

        let p2p_runtime = self.p2p_runtime.clone(); // Clone Arc for the loop
        let connection_manager = self.connection_manager.clone();
        let is_running = self.is_running.clone();

        // UniClipboard is owned by Self, but Self is consumed. The loop runs in THIS task (main thread blocked or spawned task).
        // clipboard_service.start() spawns tasks but uses internal Arc states.
        // We need access to ClipboardSyncService for some commands?
        // ClipboardSyncService only exposes `get_record_manager`.
        // We can keep `clipboard_service` alive in this scope.

        info!("AppRuntime started successfully, entering command loop");

        loop {
            if !is_running.load(Ordering::SeqCst) {
                break;
            }

            tokio::select! {
                Some(cmd) = clipboard_cmd_rx.recv() => {
                    match cmd {
                        ClipboardCommand::GetItems { respond_to } => {
                            let result: anyhow::Result<Vec<crate::infrastructure::storage::db::models::clipboard_record::DbClipboardRecord>> = self.clipboard_service.get_record_manager().get_records(None, None, None, None).await;
                            let response = result.map_err(|e: anyhow::Error| e.to_string());
                            let _ = respond_to.send(response);
                        }
                    }
                }
                Some(cmd) = p2p_cmd_rx.recv() => {
                   Self::handle_p2p_command(cmd, &p2p_runtime).await;
                }
                Some(cmd) = connection_cmd_rx.recv() => {
                    Self::handle_connection_command(cmd, &connection_manager).await;
                }
            }
        }

        Ok(())
    }

    async fn handle_p2p_command(cmd: P2PCommand, p2p_runtime: &Arc<P2PRuntime>) {
        match cmd {
            P2PCommand::GetLocalPeerId { respond_to } => {
                let _ = respond_to.send(Ok(p2p_runtime.local_peer_id().to_string()));
            }
            P2PCommand::GetPeers { respond_to } => {
                let peers = p2p_runtime.discovered_peers().await;
                let _ = respond_to.send(Ok(peers));
            }
            P2PCommand::InitiatePairing {
                peer_id,
                device_name,
                respond_to,
            } => {
                let (tx, rx) = oneshot::channel();
                let _ = p2p_runtime
                    .pairing_cmd_tx()
                    .send(PairingCommand::Initiate {
                        peer_id,
                        device_name,
                        respond_to: tx, // Send internal channel
                    })
                    .await;

                tokio::spawn(async move {
                    // Receive from internal channel (anyhow::Result)
                    let result = match rx.await {
                        Ok(res) => res.map_err(|e| e.to_string()),
                        Err(_) => Err("Pairing actor dropped response".to_string()),
                    };
                    // Respond to API (Result<String, String>)
                    let _ = respond_to.send(result);
                });
            }
            P2PCommand::VerifyPin {
                session_id,
                pin_matches,
                respond_to,
            } => {
                let (tx, rx) = oneshot::channel();

                let _ = p2p_runtime
                    .pairing_cmd_tx()
                    .send(PairingCommand::VerifyPin {
                        session_id,
                        pin_match: pin_matches,
                        respond_to: tx,
                    })
                    .await;

                tokio::spawn(async move {
                    let result = match rx.await {
                        Ok(res) => res.map_err(|e| e.to_string()),
                        Err(_) => Err("Pairing actor dropped response".to_string()),
                    };
                    let _ = respond_to.send(result);
                });
            }
            P2PCommand::RejectPairing {
                session_id,
                peer_id,
                respond_to,
            } => {
                let (tx, rx) = oneshot::channel();
                let _ = p2p_runtime
                    .pairing_cmd_tx()
                    .send(PairingCommand::Reject {
                        session_id,
                        peer_id,
                        respond_to: tx,
                    })
                    .await;

                tokio::spawn(async move {
                    let result = match rx.await {
                        Ok(res) => res.map_err(|e| e.to_string()),
                        Err(_) => Err("Pairing actor dropped response".to_string()),
                    };
                    let _ = respond_to.send(result);
                });
            }
            P2PCommand::UnpairDevice {
                peer_id,
                respond_to,
            } => {
                // TODO: Implement unpairing
                let _ = respond_to.send(Err("Unpair not implemented".to_string()));
            }
        }
    }

    async fn handle_connection_command(
        cmd: ConnectionCommand,
        connection_manager: &Arc<ConnectionManager>,
    ) {
        match cmd {
            ConnectionCommand::ManualConnect {
                ip,
                port,
                respond_to,
            } => {
                let result = connection_manager
                    .outgoing
                    .connect_to_device_manual(&ip, port)
                    .await
                    .map_err(|e| e.to_string());
                let _ = respond_to.send(result);
            }
            ConnectionCommand::RespondConnection {
                requester_device_id,
                accept,
                respond_to,
            } => {
                let result = connection_manager
                    .pending_connections
                    .respond_to_incoming_request(&requester_device_id, accept)
                    .await;
                match result {
                    Ok(tx) => {
                        let _ = tx.send(accept);
                        let _ = respond_to.send(Ok(()));
                    }
                    Err(e) => {
                        let _ = respond_to.send(Err(e.to_string()));
                    }
                }
            }
            ConnectionCommand::CancelConnectionRequest { respond_to } => {
                connection_manager.pending_connections.clear_all().await;
                let _ = respond_to.send(Ok(()));
            }
        }
    }

    /// Get the runtime handle
    pub fn handle(&self) -> AppRuntimeHandle {
        AppRuntimeHandle::new(
            self.clipboard_cmd_tx.clone(),
            self.p2p_cmd_tx.clone(),
            self.connection_cmd_tx.clone(),
            self.config.clone(),
        )
    }
}
