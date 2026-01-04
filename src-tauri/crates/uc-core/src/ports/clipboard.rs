//! Clipboard port - abstracts local clipboard access
//!
//! This port defines the interface for clipboard operations including
//! reading, writing, and monitoring clipboard changes.

use async_trait::async_trait;
use anyhow::Result;
use crate::clipboard::Payload;

/// Clipboard port - abstracts local clipboard access
///
/// This trait provides a platform-agnostic interface to clipboard functionality,
/// allowing use cases to interact with the clipboard without depending on
/// platform-specific implementations.
#[async_trait]
pub trait ClipboardPort: Send + Sync {
    /// Read current clipboard content
    ///
    /// Returns the current clipboard content as a Payload, which can contain
    /// text, images, files, or other supported content types.
    async fn read(&self) -> Result<Payload>;

    /// Write content to clipboard
    ///
    /// Sets the clipboard content to the provided payload.
    async fn write(&self, payload: Payload) -> Result<()>;

    /// Start monitoring clipboard changes
    ///
    /// Returns a receiver that will yield new payloads whenever the clipboard changes.
    /// This allows the application to react to clipboard updates in real-time.
    async fn start_monitoring(&self) -> Result<tokio::sync::mpsc::Receiver<Payload>>;
}
