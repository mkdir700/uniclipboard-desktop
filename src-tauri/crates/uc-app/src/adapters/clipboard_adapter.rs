//! Clipboard port adapter - bridges LocalClipboard infrastructure to ClipboardPort

use async_trait::async_trait;
use uc_core::clipboard::Payload;
use uc_core::ports::LocalClipboardPort;

/// Adapter that wraps LocalClipboard to implement ClipboardPort
///
/// TODO: This is a placeholder implementation. In Phase 4, we will implement
/// the actual adapter that wraps the existing LocalClipboard from the infrastructure layer.
pub struct LocalClipboardAdapter {
    _private: (),
}

impl LocalClipboardAdapter {
    /// Create a new LocalClipboardAdapter
    ///
    /// TODO: This will accept the actual LocalClipboard in Phase 4
    pub fn new() -> Self {
        Self { _private: () }
    }
}

#[async_trait]
impl LocalClipboardPort for LocalClipboardAdapter {
    /// Read current clipboard content
    async fn read(&self) -> anyhow::Result<Payload> {
        // TODO: Implement in Phase 4
        Err(anyhow::anyhow!("Not yet implemented"))
    }

    /// Write content to clipboard
    async fn write(&self, _payload: Payload) -> anyhow::Result<()> {
        // TODO: Implement in Phase 4
        Ok(())
    }

    /// Start monitoring clipboard changes
    async fn start_monitoring(&self) -> anyhow::Result<tokio::sync::mpsc::Receiver<Payload>> {
        // TODO: Implement in Phase 4
        let (_tx, rx) = tokio::sync::mpsc::channel(100);
        Ok(rx)
    }
}
