use super::content::ContentHash;

/// Event representing a user-initiated action on clipboard content.
#[derive(Debug, Clone, PartialEq)]
pub enum ClipboardContentActionEvent {
    /// User requested an action on clipboard content identified by hash.
    UserRequested {
        content_hash: ContentHash,
        action: ClipboardContentAction,
    },
}

/// Actions that can be performed on clipboard content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardContentAction {
    /// Copy content from history to system clipboard.
    CopyToSystemClipboard,
    /// Delete content from history.
    Delete,
    /// Pin content to prevent automatic deletion.
    Pin,
    /// Unpin previously pinned content.
    Unpin,
}
