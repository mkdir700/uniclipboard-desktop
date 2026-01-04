use crate::clipboard::Payload;
use crate::ids::DeviceId;

#[derive(Debug, Clone)]
pub enum SyncEvent {
    /// 本地剪贴板发生变化
    LocalClipboardChanged { payload: Payload },

    /// 收到远端剪贴板内容
    RemoteClipboardReceived {
        payload: Payload,
        origin: DeviceId,
        content_hash: String,
    },

    /// 冲突已解决（由上层决定）
    ConflictResolved { chosen: Payload },
}
