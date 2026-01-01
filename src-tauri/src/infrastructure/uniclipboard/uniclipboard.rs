//! Clipboard Sync Service
//!
//! Handles bidirectional clipboard synchronization between local and remote.

use anyhow::Result;
use log::{error, info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};

use crate::application::clipboard_content_receiver_service::ClipboardContentReceiver;
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
    content_receiver: ClipboardContentReceiver,
}

impl ClipboardSyncService {
    pub fn new(
        device_id: String,
        clipboard: Arc<dyn LocalClipboardTrait>,
        remote_sync: Arc<dyn RemoteClipboardSync>,
        record_manager: ClipboardRecordManager,
        file_storage: FileStorageManager,
    ) -> Self {
        let content_receiver = ClipboardContentReceiver::new(
            Arc::new(file_storage.clone()),
            Arc::new(record_manager.clone()),
        );

        Self {
            device_id,
            clipboard,
            remote_sync,
            record_manager,
            file_storage,
            is_running: Arc::new(AtomicBool::new(false)),
            last_payload: Arc::new(RwLock::new(None)),
            download_decision_maker: DownloadDecisionMaker::new(),
            content_receiver,
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
        if self.is_running.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
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
                            publish_clipboard_new_content(record_id.clone());

                            // Step 3: Push to remote
                            let sync_message = ClipboardTransferMessage::from((
                                metadata,
                                device_id.clone(),
                                record_id,
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
        let content_receiver = self.content_receiver.clone();

        tokio::spawn(async move {
            while is_running.load(Ordering::SeqCst) {
                match remote_sync.pull(Some(Duration::from_secs(10))).await {
                    Ok(message) => {
                        // Check download policy
                        if !download_decision_maker.should_download(&message).await {
                            info!("Content type not in download scope, skipping");
                            continue;
                        }

                        if download_decision_maker.exceeds_max_size(&message) {
                            info!("Content exceeds max size, skipping");
                            continue;
                        }

                        // Save to record
                        if let Err(e) = record_manager
                            .add_record_with_transfer_message(&message)
                            .await
                        {
                            error!("Failed to add clipboard record: {:?}", e);
                        }

                        // Receive and process content
                        match content_receiver.receive(message).await {
                            Ok(payload) => {
                                info!("Setting local clipboard: {}", payload);

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
                                    // Publish event for latest record
                                    if let Ok(records) = record_manager
                                        .get_records(
                                            Some(OrderBy::UpdatedAtDesc),
                                            Some(1),
                                            Some(0),
                                            Some(Filter::All),
                                        )
                                        .await
                                    {
                                        if let Some(latest) = records.first() {
                                            publish_clipboard_new_content(latest.id.clone());
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to receive clipboard content: {:?}", e);
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
