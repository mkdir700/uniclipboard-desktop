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
use crate::infrastructure::runtime::{AppRuntimeHandle, ClipboardCommand};
use crate::infrastructure::storage::file_storage::FileStorageManager;
use crate::infrastructure::storage::record_manager::ClipboardRecordManager;
use crate::infrastructure::uniclipboard::ClipboardSyncService;
use crate::interface::RemoteClipboardSync;

/// Application runtime - single owner of all core components
pub struct AppRuntime {
    clipboard_service: ClipboardSyncService,
    config: Arc<Setting>,
    is_running: Arc<AtomicBool>,
    // Channel for commands from Tauri handlers
    clipboard_cmd_rx: Option<mpsc::Receiver<ClipboardCommand>>,
    // Stored sender to create handles
    clipboard_cmd_tx: mpsc::Sender<ClipboardCommand>,
}

impl AppRuntime {
    /// Constructs a new AppRuntime with the given command receivers and initializes core services.
    ///
    /// Initializes file storage, clipboard record manager, the platform clipboard,
    /// and the ClipboardSyncService.
    ///
    /// # Returns
    ///
    /// `Ok(Self)` with the initialized AppRuntime on success, or an error if initialization fails.
    pub async fn new_with_channels(
        user_setting: Setting,
        device_id: String,
        device_name: String,
        _app_handle: AppHandle,
        clipboard_cmd_rx: mpsc::Receiver<ClipboardCommand>,
    ) -> Result<Self> {
        let config = Arc::new(user_setting.clone());

        // 1. Initialize core managers
        let file_storage = FileStorageManager::new()?;
        let record_manager = ClipboardRecordManager::new(
            user_setting.storage.max_history_items as usize,
        );

        // 2. Initialize Platform Clipboard
        let clipboard = Arc::new(LocalClipboard::with_user_setting(user_setting.clone())?);

        // 3. Initialize ClipboardSyncService (without P2P sync initially)
        // P2P sync will be initialized separately via P2PService
        let clipboard_service = ClipboardSyncService::new(
            device_id,
            clipboard,
            None, // No remote sync initially - P2P sync is separate
            record_manager,
            file_storage,
        );

        Ok(Self {
            clipboard_service,
            config,
            is_running: Arc::new(AtomicBool::new(false)),
            clipboard_cmd_rx: Some(clipboard_cmd_rx),
            clipboard_cmd_tx: mpsc::channel(100).0, // Will be replaced, unused
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

        let mut runtime = Self::new_with_channels(
            user_setting,
            device_id,
            device_name,
            app_handle,
            clipboard_cmd_rx,
        )
        .await?;

        // Store the sender
        runtime.clipboard_cmd_tx = clipboard_cmd_tx;

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

        let is_running = self.is_running.clone();

        // Wrap clipboard_service in Arc to be used by ClipboardService wrapper
        let clipboard_service_arc = Arc::new(self.clipboard_service);

        info!("AppRuntime started successfully, entering command loop");

        loop {
            if !is_running.load(Ordering::SeqCst) {
                break;
            }

            // Only handle clipboard commands now
            if let Some(cmd) = clipboard_cmd_rx.recv().await {
                let service =
                    crate::application::clipboard_service::ClipboardService::new(
                        clipboard_service_arc.clone(),
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
        }

        Ok(())
    }

    /// Get the runtime handle
    pub fn handle(&self) -> AppRuntimeHandle {
        AppRuntimeHandle::new(self.clipboard_cmd_tx.clone(), self.config.clone())
    }
}
