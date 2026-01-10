use super::super::common::CommonClipboardImpl;
use anyhow::Result;
use async_trait::async_trait;
use clipboard_rs::{Clipboard, ClipboardContext, RustImageData};
use std::sync::{Arc, Mutex};
use tokio::task::spawn_blocking;
use uc_core::clipboard::{ClipboardContent, MimeType};
use uc_core::clipboard::{SystemClipboardRepresentation, SystemClipboardSnapshot};
use uc_core::ports::LocalClipboardPort;

/// macOS clipboard implementation using clipboard-rs
pub struct MacOSClipboard {
    inner: Arc<Mutex<ClipboardContext>>,
}

impl MacOSClipboard {
    pub fn new() -> Result<Self> {
        let context = ClipboardContext::new()
            .map_err(|e| anyhow::anyhow!("Failed to create clipboard context: {}", e))?;
        Ok(Self {
            inner: Arc::new(Mutex::new(context)),
        })
    }
}

#[async_trait]
impl LocalClipboardPort for MacOSClipboard {
    fn read_snapshot(&self) -> Result<SystemClipboardSnapshot> {
        let mut ctx = self.inner.lock().unwrap();
        CommonClipboardImpl::read_snapshot(&mut ctx)
    }

    fn write_snapshot(&self, snapshot: SystemClipboardSnapshot) -> Result<()> {
        let mut ctx = self.inner.lock().unwrap();
        CommonClipboardImpl::write_snapshot(&mut ctx, snapshot)
    }

    fn write(&self, content: ClipboardContent) -> Result<()> {
        let mut ctx = self.inner.lock().unwrap();
        CommonClipboardImpl::write_content(&mut ctx, &content)
    }
}
