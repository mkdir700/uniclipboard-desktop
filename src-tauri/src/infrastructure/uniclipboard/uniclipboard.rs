//! Clipboard Sync Service
//!
//! Handles bidirectional clipboard synchronization between local and remote.

use anyhow::Result;
use log::{error, info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};

use crate::application::download_decision_service::DownloadDecisionMaker;
use crate::domain::transfer_message::ClipboardTransferMessage;
use crate::infrastructure::event::publish_clipboard_new_content;
use crate::infrastructure::storage::db::models::clipboard_record::{Filter, OrderBy};
use crate::infrastructure::storage::file_storage::FileStorageManager;
use crate::infrastructure::storage::record_manager::ClipboardRecordManager;
use crate::interface::local_clipboard_trait::LocalClipboardTrait;
use crate::interface::RemoteClipboardSync;
use crate::message::Payload;

/// Unified clipboard synchronization service
///
/// Manages both directions:
/// - Local clipboard changes -> Push to remote peers
/// - Remote messages -> Write to local clipboard
pub struct ClipboardSyncService {
    device_id: String,
    clipboard: Arc<dyn LocalClipboardTrait>,
    remote_sync: Arc<dyn RemoteClipboardSync>,
    record_manager: ClipboardRecordManager,
    file_storage: FileStorageManager,
    is_running: Arc<AtomicBool>,
    /// Last payload to prevent duplicate processing (echo cancellation)
    last_payload: Arc<RwLock<Option<Payload>>>,
    download_decision_maker: DownloadDecisionMaker,
}

impl ClipboardSyncService {
    pub fn new(
        device_id: String,
        clipboard: Arc<dyn LocalClipboardTrait>,
        remote_sync: Arc<dyn RemoteClipboardSync>,
        record_manager: ClipboardRecordManager,
        file_storage: FileStorageManager,
    ) -> Self {
        Self {
            device_id,
            clipboard,
            remote_sync,
            record_manager,
            file_storage,
            is_running: Arc::new(AtomicBool::new(false)),
            last_payload: Arc::new(RwLock::new(None)),
            download_decision_maker: DownloadDecisionMaker::new(),
        }
    }

    /// Get record manager reference
    pub fn get_record_manager(&self) -> &ClipboardRecordManager {
        &self.record_manager
    }

    /// Get file storage manager reference
    pub fn get_file_storage_manager(&self) -> &FileStorageManager {
        &self.file_storage
    }

    /// Get local clipboard reference
    pub fn get_local_clipboard(&self) -> Arc<dyn LocalClipboardTrait> {
        self.clipboard.clone()
    }

    /// Start clipboard synchronization
    pub async fn start(&self) -> Result<()> {
        if self
            .is_running
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            anyhow::bail!("Already running");
        }

        // Start local clipboard monitoring
        info!("Starting local clipboard monitoring");
        let clipboard_receiver = self.clipboard.start_monitoring().await?;

        // Start local-to-remote sync
        info!("Starting local to remote sync");
        self.start_outbound_sync(clipboard_receiver).await?;

        // Start remote sync service
        info!("Starting remote sync service");
        self.remote_sync.start().await?;

        // Start remote-to-local sync
        info!("Starting remote to local sync");
        self.start_inbound_sync().await?;

        info!("ClipboardSyncService started successfully");
        Ok(())
    }

    /// Handle local clipboard changes -> push to remote
    async fn start_outbound_sync(
        &self,
        mut clipboard_receiver: mpsc::Receiver<Payload>,
    ) -> Result<()> {
        let device_id = self.device_id.clone();
        let remote_sync = self.remote_sync.clone();
        let is_running = self.is_running.clone();
        let last_payload = self.last_payload.clone();
        let record_manager = self.record_manager.clone();
        let file_storage = self.file_storage.clone();

        tokio::spawn(async move {
            while is_running.load(Ordering::SeqCst) {
                if let Some(payload) = clipboard_receiver.recv().await {
                    info!("New local clipboard content: {}", payload);

                    // Check for duplicates (echo cancellation)
                    {
                        let last = last_payload.read().await;
                        if let Some(ref last_p) = *last {
                            if last_p.is_duplicate(&payload) {
                                info!("Skipping duplicate content");
                                continue;
                            }
                        }
                    }

                    // Store as last payload (atomically capture old value and write new value)
                    let prev_payload =
                        std::mem::replace(&mut *last_payload.write().await, Some(payload.clone()));

                    // Step 1: Persist to file storage
                    let storage_path = match file_storage.store(&payload).await {
                        Ok(path) => path,
                        Err(e) => {
                            error!("Failed to store payload: {:?}", e);
                            continue;
                        }
                    };

                    // Step 2: Build metadata and save record
                    let metadata = (&payload, &storage_path).into();
                    let result = record_manager
                        .add_or_update_record_with_metadata(&metadata)
                        .await;

                    match result {
                        Ok(record_id) => {
                            info!(
                                "Record saved with ID: {}, publishing clipboard-new-content event",
                                record_id
                            );
                            publish_clipboard_new_content(record_id.clone());
                            info!("Event published to event bus");

                            // Step 3: Push to remote (include content in message)
                            let content_bytes = match payload.to_bytes() {
                                Ok(bytes) => bytes,
                                Err(e) => {
                                    error!("Failed to serialize payload: {}", e);
                                    *last_payload.write().await = prev_payload;
                                    continue;
                                }
                            };

                            let sync_message = ClipboardTransferMessage::from((
                                metadata,
                                device_id.clone(),
                                content_bytes,
                            ));

                            if let Err(e) = remote_sync.push(sync_message).await {
                                // Restore previous payload on failure
                                *last_payload.write().await = prev_payload;
                                error!("Failed to push to remote: {:?}", e);
                            }
                        }
                        Err(e) => {
                            error!("Failed to add record: {:?}", e);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Handle remote messages -> write to local clipboard
    async fn start_inbound_sync(&self) -> Result<()> {
        let clipboard = self.clipboard.clone();
        let remote_sync = self.remote_sync.clone();
        let is_running = self.is_running.clone();
        let last_payload = self.last_payload.clone();
        let record_manager = self.record_manager.clone();
        let download_decision_maker = self.download_decision_maker.clone();
        let file_storage = self.file_storage.clone();

        tokio::spawn(async move {
            while is_running.load(Ordering::SeqCst) {
                match remote_sync.pull(Some(Duration::from_secs(10))).await {
                    Ok(message) => {
                        info!("Pulled clipboard message from remote sync (message-id: {})", message.message_id);

                        // Check download policy
                        if !download_decision_maker.should_download(&message).await {
                            info!("Content type {:?} not in download scope, skipping", message.metadata.get_content_type());
                            continue;
                        }

                        info!("Content type {:?} accepted by download policy", message.metadata.get_content_type());

                        // Build Payload directly from message (content is already included)
                        let payload = match Payload::try_from(&message) {
                            Ok(p) => p,
                            Err(e) => {
                                error!("Failed to build payload from message: {:?}", e);
                                continue;
                            }
                        };

                        // Store content to local file system
                        let file_path = match file_storage.store(&payload).await {
                            Ok(path) => path,
                            Err(e) => {
                                error!("Failed to store content: {:?}", e);
                                continue;
                            }
                        };

                        // Save to database
                        let metadata = (&payload, &file_path).into();
                        match record_manager.add_or_update_record_with_metadata(&metadata).await {
                            Ok(record_id) => {
                                info!("Saved clipboard record to database: {}", record_id);

                                // Atomically capture old value and write new value
                                let prev_payload = std::mem::replace(
                                    &mut *last_payload.write().await,
                                    Some(payload.clone()),
                                );

                                // Write to local clipboard
                                if let Err(e) = clipboard.set_clipboard_content(payload).await {
                                    error!("Failed to set clipboard content: {:?}", e);
                                    *last_payload.write().await = prev_payload;
                                } else {
                                    info!("Successfully wrote clipboard content to local clipboard");
                                    publish_clipboard_new_content(record_id);
                                }
                            }
                            Err(e) => {
                                error!("Failed to save record: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to pull from remote: {:?}", e);
                    }
                }
            }
        });

        Ok(())
    }
}
