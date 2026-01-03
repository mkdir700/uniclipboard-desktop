//! Application services container
//!
//! This module provides a centralized container for all core application services.
//! It manages the initialization lifecycle and wiring between services.

use log::warn;
use std::sync::Arc;

use crate::api::encryption::get_unified_encryption;
use crate::config::Setting;
use crate::domain::transfer_message::ClipboardTransferMessage;
use crate::error::{AppError, Result};
use crate::infrastructure::clipboard::LocalClipboard;
use crate::infrastructure::p2p::ClipboardMessage;
use crate::infrastructure::storage::db::pool::DB_POOL;
use crate::infrastructure::storage::file_storage::FileStorageManager;
use crate::infrastructure::sync::libp2p_sync::Libp2pSync;
use crate::interface::RemoteClipboardSync;
use crate::services::clipboard::ClipboardService;
use crate::services::p2p::P2PService;
use crate::services::storage::StorageService;
use tauri::AppHandle;
use tokio::sync::mpsc;

/// Application services container
///
/// Holds all core application services with proper initialization and wiring.
/// This is managed by Tauri and accessible via `State<'_, AppServices>`.
pub struct AppServices {
    /// Storage service (database + file storage)
    pub storage: Arc<StorageService>,
    /// Clipboard service (local + remote sync)
    pub clipboard: Arc<ClipboardService>,
    /// P2P service (device discovery and pairing)
    pub p2p: Arc<P2PService>,
}

impl AppServices {
    /// Create a new AppServices instance with all services initialized.
    ///
    /// This method initializes services in the correct dependency order:
    /// 1. StorageService (no dependencies)
    /// 2. P2PService (requires AppHandle)
    /// 3. Libp2pSync (requires P2PService network channel + encryption)
    /// 4. ClipboardService (requires StorageService + LocalClipboard + optional Libp2pSync)
    ///
    /// # Arguments
    /// * `config` - Application configuration
    /// * `device_id` - 6-digit device ID from database
    /// * `device_name` - Human-readable device name
    /// * `app_handle` - Tauri AppHandle for event emission
    pub async fn new(
        config: Arc<Setting>,
        device_id: String,
        device_name: String,
        app_handle: AppHandle,
    ) -> Result<Self> {
        // 1. Initialize StorageService (synchronous)
        let db = Arc::new(DB_POOL.pool.clone());
        let file_storage = Arc::new(FileStorageManager::new()?);
        let storage = Arc::new(StorageService::new(
            db,
            file_storage.clone(),
            config.storage.max_history_items as usize,
        ));

        // 2. Initialize P2PService (asynchronous, spawns NetworkManager)
        let p2p = Arc::new(
            P2PService::new(device_name.clone(), app_handle)
                .await
                .map_err(|e| AppError::internal(format!("Failed to create P2PService: {}", e)))?,
        );

        // 3. Initialize Libp2pSync (P2P clipboard sync service)
        let remote_sync: Option<Arc<dyn RemoteClipboardSync>> =
            if let Some(encryption) = get_unified_encryption().await {
                // Get network_cmd_tx from P2PService
                let network_cmd_tx = p2p.get_network_cmd_tx().await?;

                // Create Libp2pSync instance (keep concrete type for handle_incoming_message)
                let libp2p_sync = Arc::new(Libp2pSync::new(
                    network_cmd_tx,
                    device_name.clone(),
                    device_id.clone(),
                    encryption,
                ));

                // Set up clipboard message handler in P2PService
                // This connects NetworkManager's clipboard events to Libp2pSync
                let libp2p_sync_clone = libp2p_sync.clone();
                p2p.set_clipboard_handler(Box::new(move |msg: ClipboardMessage| {
                    let sync = libp2p_sync_clone.clone();
                    Box::pin(async move {
                        if let Err(e) = sync.handle_incoming_message(msg).await {
                            log::error!("Failed to handle incoming P2P clipboard message: {}", e);
                        }
                    })
                }))
                .await;

                // Cast to trait object for ClipboardService
                Some(libp2p_sync as Arc<dyn RemoteClipboardSync>)
            } else {
                warn!("Unified encryption not initialized, P2P clipboard sync disabled");
                None
            };

        // 4. Initialize ClipboardService (synchronous, depends on StorageService)
        let local_clipboard =
            Arc::new(LocalClipboard::with_user_setting(config.as_ref().clone())?);
        let clipboard = Arc::new(ClipboardService::new(
            device_id,
            local_clipboard,
            remote_sync, // Integrate P2P sync
            storage.clone(),
            file_storage,
        ));

        Ok(Self { storage, clipboard, p2p })
    }
}
