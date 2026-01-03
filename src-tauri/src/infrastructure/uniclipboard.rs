//! Clipboard Sync Service
//!
//! Handles bidirectional clipboard synchronization between local and remote.

use anyhow::Result;
use log::{error, info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};

use crate::domain::content_type::ContentType;
use crate::domain::transfer_message::ClipboardTransferMessage;
use crate::infrastructure::event::publish_clipboard_new_content;
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
    remote_sync: Option<Arc<dyn RemoteClipboardSync>>,
    record_manager: ClipboardRecordManager,
    file_storage: FileStorageManager,
    is_running: Arc<AtomicBool>,
    /// Last payload to prevent duplicate processing (echo cancellation)
    last_payload: Arc<RwLock<Option<Payload>>>,
}

impl ClipboardSyncService {
    /// Constructs a new ClipboardSyncService with the given device identifier and dependencies.
    ///
    /// The returned service is initially not running and has no last payload recorded. If `remote_sync`
    /// is `None`, remote synchronization is disabled and inbound/outbound remote operations will be skipped.
    ///
    /// # Examples
    ///
    /// ```
    /// // `device_id`, `clipboard`, `record_manager`, and `file_storage` are assumed to be created earlier.
    /// let service = ClipboardSyncService::new(
    ///     "my-device".to_string(),
    ///     clipboard,
    ///     None, // no remote sync configured
    ///     record_manager,
    ///     file_storage,
    /// );
    /// ```
    pub fn new(
        device_id: String,
        clipboard: Arc<dyn LocalClipboardTrait>,
        remote_sync: Option<Arc<dyn RemoteClipboardSync>>,
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
        }
    }

    /// Determines whether payloads of the given content type should be downloaded according to user sync settings.
    ///
    /// This consults the global settings instance and returns the configured flag for the provided `ContentType`.
    ///
    /// # Returns
    ///
    /// `true` if the user has enabled downloading for `content_type`, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// // Run the async method to get the decision.
    /// // `svc` is an instance of `ClipboardSyncService`.
    /// let allowed = futures::executor::block_on(svc.should_download_content(&ContentType::Image));
    /// println!("Download image content allowed: {}", allowed);
    /// ```
    async fn should_download_content(&self, content_type: &ContentType) -> bool {
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

    /// Starts the clipboard synchronization service, initializing local monitoring, outbound and inbound synchronization, and the optional remote sync service.
    ///
    /// On success, background tasks for processing outbound (local-to-remote) and inbound (remote-to-local) clipboard flows are started and the service transitions to a running state.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the service started successfully; `Err` if the service was already running or if any required subsystem failed to start.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assuming `svc` is an initialized `ClipboardSyncService` instance.
    /// // This example demonstrates the typical call site; adapt test runtime as needed.
    /// # async fn run_example(svc: &ClipboardSyncService) -> anyhow::Result<()> {
    /// svc.start().await?;
    /// # Ok(())
    /// # }
    /// ```
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
        if let Some(ref sync) = self.remote_sync {
            info!("Starting remote sync service");
            sync.start().await?;
        } else {
            info!("No remote sync configured, skipping remote sync start");
        }

        // Start remote-to-local sync
        info!("Starting remote to local sync");
        self.start_inbound_sync().await?;

        info!("ClipboardSyncService started successfully");
        Ok(())
    }

    /// Spawn a background task that processes local clipboard payloads: persist content to file storage, create/update a record, publish a local event, and optionally push the content to the configured remote sync service.
    ///
    /// The task performs echo-cancellation against the service's internal `last_payload`, stores payload content to `file_storage`, saves metadata via `record_manager`, publishes a clipboard-new-content event on success, and — if a remote sync is configured — serializes the payload and pushes a `ClipboardTransferMessage`. If pushing to the remote fails, the previous `last_payload` is restored to avoid losing synchronization state.
    ///
    /// Arguments:
    /// - `clipboard_receiver`: a receiver that yields local `Payload` instances to be processed.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the background task was spawned successfully.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::sync::mpsc;
    ///
    /// // Assume `service` is an initialized ClipboardSyncService and `Payload` is available.
    /// let (tx, rx) = mpsc::channel(16);
    /// // Spawn the outbound sync task that will consume from `rx`.
    /// service.start_outbound_sync(rx).await.unwrap();
    /// // Send a payload from elsewhere:
    /// // tx.send(payload).await.unwrap();
    /// ```
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
                            if let Some(ref sync) = remote_sync {
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

                                if let Err(e) = sync.push(sync_message).await {
                                    // Restore previous payload on failure
                                    *last_payload.write().await = prev_payload;
                                    error!("Failed to push to remote: {:?}", e);
                                }
                            } else {
                                info!("No remote sync configured, skipping push to remote");
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

    /// Start the inbound synchronization loop that pulls remote clipboard messages and writes accepted content to the local clipboard.
    ///
    /// Runs a background task that continuously:
    /// - pulls messages from the configured remote sync (if any),
    /// - applies content-type download policy,
    /// - materializes message content into a Payload and stores it to local file storage,
    /// - persists a clipboard record, and
    /// - writes the payload to the local clipboard while maintaining echo-cancellation state.
    ///
    /// The task stops when the service's running flag is cleared. If no remote sync is configured the task sleeps and retries periodically.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use tokio;
    /// # async fn example(svc: Arc<crate::clipboard::ClipboardSyncService>) {
    /// svc.start_inbound_sync().await.unwrap();
    /// # }
    /// ```
    async fn start_inbound_sync(&self) -> Result<()> {
        let clipboard = self.clipboard.clone();
        let remote_sync = self.remote_sync.clone();
        let is_running = self.is_running.clone();
        let last_payload = self.last_payload.clone();
        let record_manager = self.record_manager.clone();
        let file_storage = self.file_storage.clone();

        tokio::spawn(async move {
            while is_running.load(Ordering::SeqCst) {
                let Some(ref sync) = remote_sync else {
                    // No remote sync configured, wait and check again
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    continue;
                };

                match sync.pull(Some(Duration::from_secs(10))).await {
                    Ok(message) => {
                        info!(
                            "Pulled clipboard message from remote sync (message-id: {})",
                            message.message_id
                        );

                        // Check download policy
                        let content_type = message.metadata.get_content_type();
                        let should_download = {
                            let setting = crate::config::Setting::get_instance();
                            match content_type {
                                ContentType::Text => setting.sync.content_types.text,
                                ContentType::Image => setting.sync.content_types.image,
                                ContentType::File => setting.sync.content_types.file,
                                ContentType::RichText => setting.sync.content_types.rich_text,
                                ContentType::Link => setting.sync.content_types.link,
                                ContentType::CodeSnippet => setting.sync.content_types.code_snippet,
                            }
                        };

                        if !should_download {
                            info!(
                                "Content type {:?} not in download scope, skipping",
                                content_type
                            );
                            continue;
                        }

                        info!(
                            "Content type {:?} accepted by download policy",
                            content_type
                        );

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
                        match record_manager
                            .add_or_update_record_with_metadata(&metadata)
                            .await
                        {
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
                                    info!(
                                        "Successfully wrote clipboard content to local clipboard"
                                    );
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
