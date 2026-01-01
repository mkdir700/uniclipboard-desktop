//! Application runtime
//!
//! Single owner of all core application components.
use anyhow::Result;
use log::info;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::{mpsc, oneshot};

use crate::config::Setting;
use crate::infrastructure::clipboard::LocalClipboard;
use crate::infrastructure::p2p::pairing::PairingCommand;
use crate::infrastructure::runtime::p2p_runtime::P2PRuntime;
use crate::infrastructure::runtime::{AppRuntimeHandle, ClipboardCommand, P2PCommand};
use crate::infrastructure::storage::file_storage::FileStorageManager;
use crate::infrastructure::storage::record_manager::ClipboardRecordManager;
use crate::infrastructure::sync::RemoteSyncManager;
use crate::infrastructure::uniclipboard::ClipboardSyncService;
use crate::interface::{RemoteClipboardSync, RemoteSyncManagerTrait};

/// Application runtime - single owner of all core components
pub struct AppRuntime {
    clipboard_service: ClipboardSyncService,
    p2p_runtime: Arc<P2PRuntime>,
    config: Arc<Setting>,
    is_running: Arc<AtomicBool>,
    // Channels for commands from Tauri handlers
    clipboard_cmd_rx: Option<mpsc::Receiver<ClipboardCommand>>,
    p2p_cmd_rx: Option<mpsc::Receiver<P2PCommand>>,
    // Stored senders to create handles
    clipboard_cmd_tx: mpsc::Sender<ClipboardCommand>,
    p2p_cmd_tx: mpsc::Sender<P2PCommand>,
}

impl AppRuntime {
    /// Create a new application runtime
    pub async fn new_with_channels(
        user_setting: Setting,
        device_id: String,
        device_name: String,
        app_handle: AppHandle,
        clipboard_cmd_rx: mpsc::Receiver<ClipboardCommand>,
        p2p_cmd_rx: mpsc::Receiver<P2PCommand>,
    ) -> Result<Self> {
        let config = Arc::new(user_setting.clone());

        // 1. Initialize core managers
        let file_storage = FileStorageManager::new()?;
        let record_manager =
            ClipboardRecordManager::new(user_setting.storage.max_history_items as usize);

        // 2. Initialize Platform Clipboard
        let clipboard = Arc::new(LocalClipboard::with_user_setting(user_setting.clone())?);

        // 3. Initialize P2P Runtime with AppHandle
        let p2p_runtime =
            Arc::new(P2PRuntime::new(device_name.clone(), config.clone(), app_handle).await?);

        // Initialize RemoteSyncManager
        let remote_sync_manager =
            Arc::new(RemoteSyncManager::with_user_setting(user_setting.clone()));

        // Set P2P Sync as default handler
        remote_sync_manager
            .set_sync_handler(p2p_runtime.p2p_sync())
            .await;

        // 4. Initialize ClipboardSyncService
        // Use RemoteSyncManager as the sync interface
        let remote_sync = remote_sync_manager.clone() as Arc<dyn RemoteClipboardSync>;
        let clipboard_service = ClipboardSyncService::new(
            device_id,
            clipboard,
            remote_sync,
            record_manager,
            file_storage,
        );

        Ok(Self {
            clipboard_service,
            p2p_runtime,
            config,
            is_running: Arc::new(AtomicBool::new(false)),
            clipboard_cmd_rx: Some(clipboard_cmd_rx),
            p2p_cmd_rx: Some(p2p_cmd_rx),
            clipboard_cmd_tx: mpsc::channel(100).0, // Will be replaced, unused
            p2p_cmd_tx: mpsc::channel(100).0,       // Will be replaced, unused
        })
    }

    /// Create a new application runtime (creates its own channels)
    pub async fn new(
        user_setting: Setting,
        device_id: String,
        device_name: String,
        app_handle: AppHandle,
    ) -> Result<Self> {
        let (clipboard_cmd_tx, clipboard_cmd_rx) = mpsc::channel(100);
        let (p2p_cmd_tx, p2p_cmd_rx) = mpsc::channel(100);

        let mut runtime = Self::new_with_channels(
            user_setting,
            device_id,
            device_name,
            app_handle,
            clipboard_cmd_rx,
            p2p_cmd_rx,
        )
        .await?;

        // Store the senders
        runtime.clipboard_cmd_tx = clipboard_cmd_tx;
        runtime.p2p_cmd_tx = p2p_cmd_tx;

        Ok(runtime)
    }

    /// Start the application runtime
    pub async fn start(mut self) -> Result<()> {
        if self.is_running.load(Ordering::SeqCst) {
            return Ok(());
        }
        self.is_running.store(true, Ordering::SeqCst);
        info!("Starting AppRuntime...");

        // Start ClipboardSyncService
        self.clipboard_service.start().await?;

        // Command Loop
        let mut clipboard_cmd_rx = self
            .clipboard_cmd_rx
            .take()
            .expect("Clipboard RX channel missing");
        let mut p2p_cmd_rx = self.p2p_cmd_rx.take().expect("P2P RX channel missing");

        let p2p_runtime = self.p2p_runtime.clone(); // Clone Arc for the loop
        let is_running = self.is_running.clone();

        // Wrap clipboard_service in Arc to be used by ClipboardService wrapper
        let clipboard_service_arc = Arc::new(self.clipboard_service);

        info!("AppRuntime started successfully, entering command loop");

        loop {
            if !is_running.load(Ordering::SeqCst) {
                break;
            }

            tokio::select! {
                Some(cmd) = clipboard_cmd_rx.recv() => {
                    let service = crate::application::clipboard_service::ClipboardService::new(
                         clipboard_service_arc.clone()
                    );

                    match cmd {
                         ClipboardCommand::GetStats { respond_to } => {
                            let result = service.get_clipboard_stats().await.map_err(|e| e.to_string());
                            let _ = respond_to.send(result);
                        }
                        ClipboardCommand::GetItems {
                            order_by,
                            limit,
                            offset,
                            filter,
                            respond_to,
                        } => {
                            let result = service
                                .get_clipboard_items(order_by, limit, offset, filter)
                                .await
                                .map_err(|e| e.to_string());
                            let _ = respond_to.send(result);
                        }
                        ClipboardCommand::GetItem {
                            id,
                            full_content,
                            respond_to,
                        } => {
                            let result = service
                                .get_clipboard_item(&id, full_content)
                                .await
                                .map_err(|e| e.to_string());
                            let _ = respond_to.send(result);
                        }
                        ClipboardCommand::DeleteItem { id, respond_to } => {
                            let result = service
                                .delete_clipboard_item(&id)
                                .await
                                .map_err(|e| e.to_string());
                            let _ = respond_to.send(result);
                        }
                        ClipboardCommand::ClearItems { respond_to } => {
                            let result = service
                                .clear_clipboard_items()
                                .await
                                .map_err(|e| e.to_string());
                            let _ = respond_to.send(result);
                        }
                        ClipboardCommand::CopyItem { id, respond_to } => {
                            let result = service
                                .copy_clipboard_item(&id)
                                .await
                                .map_err(|e| e.to_string());
                            let _ = respond_to.send(result);
                        }
                        ClipboardCommand::ToggleFavorite {
                            id,
                            is_favorited,
                            respond_to,
                        } => {
                            let result = if is_favorited {
                                service.favorite_clipboard_item(&id).await
                            } else {
                                service.unfavorite_clipboard_item(&id).await
                            };
                            let _ = respond_to.send(result.map_err(|e| e.to_string()));
                        }
                    }
                }
                Some(cmd) = p2p_cmd_rx.recv() => {
                   Self::handle_p2p_command(cmd, &p2p_runtime).await;
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
            P2PCommand::GetLocalDeviceInfo { respond_to } => {
                use crate::infrastructure::runtime::LocalDeviceInfo;
                let device_info = LocalDeviceInfo {
                    peer_id: p2p_runtime.local_peer_id().to_string(),
                    device_name: p2p_runtime.device_name().to_string(),
                };
                let _ = respond_to.send(Ok(device_info));
            }
            P2PCommand::GetPairedPeers { respond_to } => {
                let peers = p2p_runtime
                    .p2p_sync()
                    .peer_storage()
                    .get_all_peers();
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
                let result = p2p_runtime
                    .unpair_peer(&peer_id)
                    .map_err(|e| e.to_string());
                let _ = respond_to.send(result);
            }
            P2PCommand::AcceptPairing {
                session_id,
                respond_to,
            } => {
                let (tx, rx) = oneshot::channel();
                let _ = p2p_runtime
                    .pairing_cmd_tx()
                    .send(PairingCommand::AcceptPairing {
                        session_id,
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
        }
    }

    /// Get the runtime handle
    pub fn handle(&self) -> AppRuntimeHandle {
        AppRuntimeHandle::new(
            self.clipboard_cmd_tx.clone(),
            self.p2p_cmd_tx.clone(),
            self.config.clone(),
        )
    }
}
