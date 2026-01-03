//! Clipboard Service
//!
//! High-level clipboard operations with local/remote synchronization.
//! Integrates local clipboard operations, remote synchronization, and storage management.

use crate::application::clipboard_service::ClipboardItemResponse;
use crate::domain::clipboard_metadata::ClipboardMetadata;
use crate::domain::content_type::ContentType;
use crate::domain::transfer_message::ClipboardTransferMessage;
use crate::error::{AppError, Result};
use crate::infrastructure::storage::db::models::clipboard_record::{Filter, OrderBy};
use crate::infrastructure::storage::file_storage::FileStorageManager;
use crate::infrastructure::storage::record_manager::ClipboardStats;
use crate::interface::local_clipboard_trait::LocalClipboardTrait;
use crate::interface::RemoteClipboardSync;
use crate::message::Payload;
use crate::services::storage::StorageService;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// High-level clipboard service
///
/// Integrates local clipboard operations, remote synchronization,
/// and storage management without using channels.
pub struct ClipboardService {
    /// Device identifier
    device_id: String,

    /// Local clipboard interface
    local_clipboard: Arc<dyn LocalClipboardTrait>,

    /// Optional remote sync service
    remote_sync: Option<Arc<dyn RemoteClipboardSync>>,

    /// Storage service (database + file storage)
    storage: Arc<StorageService>,

    /// File storage manager
    file_storage: Arc<FileStorageManager>,

    /// Service state
    is_running: Arc<AtomicBool>,

    /// Last payload for echo cancellation
    last_payload: Arc<RwLock<Option<Payload>>>,
}

impl ClipboardService {
    /// Create a new ClipboardService instance
    pub fn new(
        device_id: String,
        local_clipboard: Arc<dyn LocalClipboardTrait>,
        remote_sync: Option<Arc<dyn RemoteClipboardSync>>,
        storage: Arc<StorageService>,
        file_storage: Arc<FileStorageManager>,
    ) -> Self {
        Self {
            device_id,
            local_clipboard,
            remote_sync,
            storage,
            file_storage,
            is_running: Arc::new(AtomicBool::new(false)),
            last_payload: Arc::new(RwLock::new(None)),
        }
    }

    // ========== Query Operations (delegated to StorageService) ==========

    /// Get clipboard statistics
    pub async fn get_clipboard_stats(&self) -> Result<ClipboardStats> {
        self.storage.get_clipboard_stats().await
    }

    /// Get clipboard items with filtering
    pub async fn get_clipboard_items(
        &self,
        filter: Option<Filter>,
        order_by: Option<OrderBy>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<ClipboardItemResponse>> {
        self.storage
            .get_clipboard_items(filter, order_by, limit, offset)
            .await
    }

    /// Get single clipboard item
    pub async fn get_clipboard_item(
        &self,
        id: &str,
        full_content: bool,
    ) -> Result<Option<ClipboardItemResponse>> {
        self.storage.get_clipboard_item_by_id(id, full_content).await
    }

    // ========== Modify Operations ==========

    /// Delete clipboard item
    pub async fn delete_clipboard_item(&self, id: &str) -> Result<bool> {
        self.storage.delete_clipboard_item(id).await?;
        Ok(true)
    }

    /// Clear all clipboard items
    pub async fn clear_clipboard_items(&self) -> Result<usize> {
        self.storage.clear_clipboard_items().await
    }

    /// Copy clipboard item to system clipboard
    pub async fn copy_clipboard_item(&self, id: &str) -> Result<bool> {
        // Get raw database record from storage
        let record = self
            .storage
            .get_clipboard_record_by_id(id)
            .await?
            .ok_or_else(|| AppError::validation("Clipboard item not found"))?;

        // Update active time
        self.storage
            .update_clipboard_item_active_time(id, None)
            .await?;

        // Convert DbClipboardRecord to Payload (已有实现: Payload::try_from(record))
        let payload = Payload::try_from(record)
            .map_err(|e| AppError::clipboard(format!("Failed to create payload: {}", e)))?;

        // Write to local clipboard
        self.local_clipboard
            .write(payload)
            .await
            .map_err(|e| AppError::clipboard(format!("Failed to write to clipboard: {}", e)))?;

        Ok(true)
    }

    /// Toggle favorite status
    pub async fn toggle_clipboard_item_favorite(
        &self,
        id: &str,
        is_favorited: bool,
    ) -> Result<bool> {
        self.storage
            .toggle_clipboard_item_favorite(id, is_favorited)
            .await?;
        Ok(true)
    }

    // ========== Service Lifecycle & Synchronization ==========

    /// Start the clipboard service
    ///
    /// This spawns background tasks for:
    /// - Local clipboard monitoring
    /// - Outbound sync (local -> remote)
    /// - Inbound sync (remote -> local)
    pub async fn start(&self) -> Result<()> {
        if self
            .is_running
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err(AppError::internal("Clipboard service already running"));
        }

        // Start local clipboard monitoring
        let clipboard_receiver = self
            .local_clipboard
            .start_monitoring()
            .await
            .map_err(|e| AppError::clipboard(format!("Failed to start monitoring: {}", e)))?;

        // Start outbound sync (local -> remote)
        self.start_outbound_sync(clipboard_receiver).await?;

        // Start remote sync service
        if let Some(ref sync) = self.remote_sync {
            sync
                .start()
                .await
                .map_err(|e| AppError::p2p(format!("Failed to start remote sync: {}", e)))?;
        }

        // Start inbound sync (remote -> local)
        self.start_inbound_sync().await?;

        log::info!("ClipboardService started successfully");
        Ok(())
    }

    /// Start outbound sync (local -> remote)
    async fn start_outbound_sync(
        &self,
        mut clipboard_receiver: mpsc::Receiver<Payload>,
    ) -> Result<()> {
        let device_id = self.device_id.clone();
        let remote_sync = self.remote_sync.clone();
        let storage = self.storage.clone();
        let file_storage = self.file_storage.clone();
        let is_running = self.is_running.clone();
        let last_payload = self.last_payload.clone();

        tokio::spawn(async move {
            while is_running.load(Ordering::SeqCst) {
                let Some(payload) = clipboard_receiver.recv().await else {
                    break;
                };

                // Echo cancellation
                {
                    let last = last_payload.read().await;
                    if let Some(ref last_p) = *last {
                        if last_p.is_duplicate(&payload) {
                            log::info!("Skipping duplicate content");
                            continue;
                        }
                    }
                }

                // Store to file
                let storage_path = match file_storage.store(&payload).await {
                    Ok(path) => path,
                    Err(e) => {
                        log::error!("Failed to store payload: {:?}", e);
                        continue;
                    }
                };

                // Save to database
                let metadata: ClipboardMetadata = (&payload, &storage_path).into();
                let record_id = match storage.save_clipboard_item(&metadata).await {
                    Ok(id) => id,
                    Err(e) => {
                        log::error!("Failed to save record: {:?}", e);
                        continue;
                    }
                };

                // Publish event
                crate::infrastructure::event::publish_clipboard_new_content(record_id.clone());

                // Update last_payload
                *last_payload.write().await = Some(payload.clone());

                // Push to remote (if configured)
                if let Some(ref sync) = remote_sync {
                    let content_bytes = match payload.to_bytes() {
                        Ok(bytes) => bytes,
                        Err(e) => {
                            log::error!("Failed to serialize payload: {}", e);
                            continue;
                        }
                    };

                    let sync_message = ClipboardTransferMessage::from((
                        metadata,
                        device_id.clone(),
                        content_bytes,
                    ));

                    if let Err(e) = sync.push(sync_message).await {
                        log::error!("Failed to push to remote: {:?}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Start inbound sync (remote -> local)
    async fn start_inbound_sync(&self) -> Result<()> {
        let clipboard = self.local_clipboard.clone();
        let remote_sync = self.remote_sync.clone();
        let storage = self.storage.clone();
        let file_storage = self.file_storage.clone();
        let is_running = self.is_running.clone();
        let last_payload = self.last_payload.clone();

        tokio::spawn(async move {
            while is_running.load(Ordering::SeqCst) {
                let Some(ref sync) = remote_sync else {
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                    continue;
                };

                match sync.pull(Some(std::time::Duration::from_secs(10))).await {
                    Ok(message) => {
                        // Check download policy
                        let content_type = message.metadata.get_content_type();
                        let should_download = Self::should_download_content(&content_type).await;

                        if !should_download {
                            log::info!("Content type {:?} not in download scope", content_type);
                            continue;
                        }

                        // Build Payload from message (已有实现: Payload::try_from(&message))
                        let payload = match Payload::try_from(&message) {
                            Ok(p) => p,
                            Err(e) => {
                                log::error!("Failed to build payload: {:?}", e);
                                continue;
                            }
                        };

                        // Store to file system
                        let file_path = match file_storage.store(&payload).await {
                            Ok(path) => path,
                            Err(e) => {
                                log::error!("Failed to store content: {:?}", e);
                                continue;
                            }
                        };

                        // Save to database
                        let metadata: ClipboardMetadata = (&payload, &file_path).into();
                        match storage.save_clipboard_item(&metadata).await {
                            Ok(record_id) => {
                                // Update last_payload
                                *last_payload.write().await = Some(payload.clone());

                                // Write to local clipboard
                                if let Err(e) = clipboard.set_clipboard_content(payload).await {
                                    log::error!("Failed to set clipboard content: {:?}", e);
                                } else {
                                    crate::infrastructure::event::publish_clipboard_new_content(record_id);
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to save record: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to pull from remote: {:?}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Check if content type should be downloaded (from sync settings)
    async fn should_download_content(content_type: &ContentType) -> bool {
        let setting = crate::config::Setting::get_instance();
        match content_type {
            ContentType::Text => setting.sync.content_types.text,
            ContentType::Image => setting.sync.content_types.image,
            ContentType::File => setting.sync.content_types.file,
            ContentType::RichText => setting.sync.content_types.rich_text,
            ContentType::Link => setting.sync.content_types.link,
            ContentType::CodeSnippet => setting.sync.content_types.code_snippet,
        }
    }
}
