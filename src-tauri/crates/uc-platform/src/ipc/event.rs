use std::time::SystemTime;
use uc_core::SystemClipboardSnapshot;

#[derive(Debug, Clone)]
pub struct PlatformStatus {
    pub state: PlatformState,
    pub last_clipboard_at: Option<SystemTime>,
    pub updated_at: SystemTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformState {
    Idle,
    Running,
    Suspended,
    Error,
}

#[derive(Debug, Clone)]
pub enum PlatformEvent {
    /// 平台启动完成（runtime 已就绪）
    Started,

    /// 平台已停止（所有后台任务已退出）
    Stopped,

    /// 本地剪切板发生变化
    ClipboardChanged { snapshot: SystemClipboardSnapshot },

    /// 剪切板内容已成功同步到至少一个设备
    ClipboardSynced { peer_count: usize },

    /// 操作失败（一次性错误）
    Error { message: String },
}
