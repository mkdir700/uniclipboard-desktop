pub enum ClipboardContentActionEvent {
    UserRequested {
        content_hash: String,
        action: ClipboardContentAction,
    },
}
pub enum ClipboardContentAction {
    CopyToSystemClipboard,
    Delete,
    Pin,
    Unpin,
}
