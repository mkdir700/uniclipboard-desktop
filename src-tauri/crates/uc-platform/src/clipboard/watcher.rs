use std::sync::Arc;
use tracing::warn;

use clipboard_rs::ClipboardHandler;

use crate::ipc::PlatformEvent;
use crate::runtime::event_bus::PlatformEventSender;
use uc_core::ports::SystemClipboardPort;

pub struct ClipboardWatcher {
    local_clipboard: Arc<dyn SystemClipboardPort>,
    sender: PlatformEventSender,
}

impl ClipboardWatcher {
    pub fn new(local_clipboard: Arc<dyn SystemClipboardPort>, sender: PlatformEventSender) -> Self {
        Self {
            local_clipboard,
            sender,
        }
    }
}

impl ClipboardHandler for ClipboardWatcher {
    fn on_clipboard_change(&mut self) {
        match self.local_clipboard.read_snapshot() {
            Ok(snapshot) => {
                if let Err(err) = self
                    .sender
                    .try_send(PlatformEvent::ClipboardChanged { snapshot })
                {
                    warn!(error = %err, "Failed to notify clipboard change");
                }
            }

            Err(e) => {
                warn!(error = %e, "Failed to read clipboard snapshot");
            }
        }
    }
}
