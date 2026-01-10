use std::sync::Arc;

use clipboard_rs::ClipboardHandler;

use crate::ipc::PlatformEvent;
use crate::runtime::event_bus::PlatformEventSender;
use uc_core::ports::LocalClipboardPort;

pub struct ClipboardWatcher {
    local_clipboard: Arc<dyn LocalClipboardPort>,
    sender: PlatformEventSender,
}

impl ClipboardWatcher {
    pub fn new(local_clipboard: Arc<dyn LocalClipboardPort>, sender: PlatformEventSender) -> Self {
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
                    log::warn!("failed to notify clipboard change: {}", err);
                }
            }

            Err(e) => {
                log::warn!("failed to read clipboard snapshot: {}", e);
            }
        }
    }
}
