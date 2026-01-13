//! Clipboard change handler port
//!
//! This port defines the callback interface for handling clipboard change events
//! from the platform layer. It follows the Dependency Inversion Principle:
//! - Platform layer (low-level) depends on this abstraction
//! - App layer (high-level) implements this interface

use anyhow::Result;
use crate::SystemClipboardSnapshot;

/// Callback handler for clipboard change events.
///
/// The platform layer calls this when clipboard content changes.
/// The snapshot is already read by the platform layer.
#[async_trait::async_trait]
pub trait ClipboardChangeHandler: Send + Sync {
    /// Called when clipboard content changes.
    ///
    /// # Parameters
    /// - `snapshot`: The current clipboard state captured by platform layer
    async fn on_clipboard_changed(&self, snapshot: SystemClipboardSnapshot) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that ClipboardChangeHandler is object-safe
    #[test]
    fn test_clipboard_change_handler_is_object_safe() {
        fn assert_object_safe(_trait_obj: &dyn ClipboardChangeHandler) {}
        assert!(true, "ClipboardChangeHandler is object-safe");
    }
}
