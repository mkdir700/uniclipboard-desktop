use crate::{
    clipboard::{ClipboardContent, ContentHash},
    DeviceId,
};

#[derive(Debug, Clone)]
pub enum SyncEvent {
    /// 本地剪贴板发生变化
    LocalClipboardChanged { content: ClipboardContent },

    /// 收到远端剪贴板内容
    RemoteClipboardReceived {
        content: ClipboardContent,
        origin: DeviceId,
        content_hash: ContentHash,
    },

    /// 冲突已解决（由上层决定）
    ConflictResolved { chosen: ClipboardContent },
}
