use std::sync::Arc;

use super::event_bus::{PlatformCommandReceiver, PlatformEventReceiver, PlatformEventSender};
use crate::clipboard::watcher::ClipboardWatcher;
use crate::clipboard::LocalClipboard;
use crate::ipc::{PlatformCommand, PlatformEvent};
use crate::ports::PlatformCommandExecutorPort;
use anyhow::Result;
use clipboard_rs::{
    ClipboardWatcher as RSClipboardWatcher, ClipboardWatcherContext, WatcherShutdown,
};
use tokio::task::JoinHandle;
use uc_core::ports::SystemClipboardPort;

pub struct PlatformRuntime<E>
where
    E: PlatformCommandExecutorPort,
{
    #[allow(dead_code)]
    local_clipboard: Arc<dyn SystemClipboardPort>,
    #[allow(dead_code)]
    event_tx: PlatformEventSender,
    event_rx: PlatformEventReceiver,
    command_rx: PlatformCommandReceiver,
    #[allow(dead_code)]
    executor: Arc<E>,
    shutting_down: bool,
    #[allow(dead_code)]
    watcher_join: Option<JoinHandle<()>>,
    #[allow(dead_code)]
    watcher_handle: Option<WatcherShutdown>,
}

impl<E> PlatformRuntime<E>
where
    E: PlatformCommandExecutorPort,
{
    pub fn new(
        event_tx: PlatformEventSender,
        event_rx: PlatformEventReceiver,
        command_rx: PlatformCommandReceiver,
        executor: Arc<E>,
    ) -> Result<PlatformRuntime<E>, anyhow::Error> {
        let local_clipboard = Arc::new(LocalClipboard::new()?);

        Ok(Self {
            local_clipboard,
            event_tx,
            event_rx,
            command_rx,
            executor,
            shutting_down: false,
            watcher_join: None,
            watcher_handle: None,
        })
    }

    pub async fn start(mut self) {
        while !self.shutting_down {
            tokio::select! {
                Some(event) = self.event_rx.recv() => {
                    self.handle_event(event).await;
                }
                Some(cmd) = self.command_rx.recv() => {
                    self.handle_command(cmd).await;
                }
            }
        }
    }

    #[allow(dead_code)]
    fn start_clipboard_watcher(&mut self) -> Result<()> {
        let mut watcher_ctx = ClipboardWatcherContext::new()
            .map_err(|e| anyhow::anyhow!("Failed to create watcher context: {}", e))?;

        let handler = ClipboardWatcher::new(self.local_clipboard.clone(), self.event_tx.clone());

        let shutdown = watcher_ctx.add_handler(handler).get_shutdown_channel();

        let join = tokio::task::spawn_blocking(move || {
            log::info!("start clipboard watch");
            watcher_ctx.start_watch();
            log::info!("clipboard watch stopped");
        });

        self.watcher_join = Some(join);
        self.watcher_handle = Some(shutdown);
        Ok(())
    }

    async fn handle_event(&self, event: PlatformEvent) {
        match event {
            PlatformEvent::ClipboardChanged { snapshot } => {
                log::debug!(
                    "Clipboard changed: {} representations, {} bytes",
                    snapshot.representation_count(),
                    snapshot.total_size_bytes()
                );
                // TODO: In future tasks, this will trigger the SyncClipboard use case
                // For now, just log the event
            }
            PlatformEvent::ClipboardSynced { peer_count } => {
                log::debug!("Clipboard synced to {} peers", peer_count);
            }
            PlatformEvent::Started => {
                log::info!("Platform runtime started");
            }
            PlatformEvent::Stopped => {
                log::info!("Platform runtime stopped");
            }
            PlatformEvent::Error { message } => {
                log::error!("Platform error: {}", message);
            }
        }
    }

    async fn handle_command(&mut self, command: PlatformCommand) {
        match command {
            PlatformCommand::Shutdown => {
                self.shutting_down = true;
                log::info!("Platform runtime shutting down");
            }
            PlatformCommand::ReadClipboard => {
                match self.local_clipboard.read_snapshot() {
                    Ok(snapshot) => {
                        log::debug!(
                            "Read clipboard: {} representations, {} bytes",
                            snapshot.representation_count(),
                            snapshot.total_size_bytes()
                        );
                        // TODO: Send response back through a response channel
                        // For now, just log
                    }
                    Err(e) => {
                        log::error!("Failed to read clipboard: {:?}", e);
                    }
                }
            }
            PlatformCommand::WriteClipboard { content: _ } => {
                // Convert ClipboardContent to SystemClipboardSnapshot
                // For now, we'll need to handle this conversion
                log::debug!("WriteClipboard command received (conversion needed)");
                // TODO: Implement proper conversion from ClipboardContent to SystemClipboardSnapshot
                // This requires mapping clipboard-rs types to our snapshot format
            }
            PlatformCommand::StartClipboardWatcher => {
                log::debug!("StartClipboardWatcher command received");
                if let Err(e) = self.start_clipboard_watcher() {
                    log::error!("Failed to start clipboard watcher: {:?}", e);
                }
            }
            PlatformCommand::StopClipboardWatcher => {
                log::debug!("StopClipboardWatcher command received");
                if let Some(handle) = self.watcher_handle.take() {
                    handle.stop();
                    log::info!("Clipboard watcher stopped");
                }
            }
        }
    }
}
