use super::super::common::CommonClipboardImpl;
use anyhow::Result;
use async_trait::async_trait;
use clipboard_rs::ClipboardContext;
use std::sync::{Arc, Mutex};
use uc_core::clipboard::SystemClipboardSnapshot;
use uc_core::ports::SystemClipboardPort;

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
impl SystemClipboardPort for MacOSClipboard {
    fn read_snapshot(&self) -> Result<SystemClipboardSnapshot> {
        let mut ctx = self.inner.lock().unwrap();
        CommonClipboardImpl::read_snapshot(&mut ctx)
    }

    fn write_snapshot(&self, snapshot: SystemClipboardSnapshot) -> Result<()> {
        let mut ctx = self.inner.lock().unwrap();
        CommonClipboardImpl::write_snapshot(&mut ctx, snapshot)
    }
}
