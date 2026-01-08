// PlatformEvent 的设计铁律（记住这 5 条）
// 1️⃣ 只描述「事实」，不描述「意图」
// ✅ ClipboardChanged
// ❌ SyncClipboardRequested
// 2️⃣ 不携带“下一步怎么做”的信息
// ✅ PairingRequested { peer_id }
// ❌ NeedUserToAcceptPairing
// 3️⃣ 不能暴露实现细节
// ✅ NetworkDisconnected
// ❌ Libp2pQuicTimeout
// 4️⃣ UI / CLI 必须能直接消费
// UI 拿到 event，不需要再问 platform “这是什么意思”
// 5️⃣ 数量要少，语义要稳
// event 一旦定义，就是对外协议
// 如果 UI/CLI 需要“知道这件事发生过”，
// 而且不需要知道“怎么发生的”，
// 那它就应该是 PlatformEvent

use std::time::SystemTime;

use uc_core::clipboard::ClipboardContent;

#[derive(Debug, Clone)]
pub struct PlatformStatus {
    pub state: PlatformState,
    pub connected_peers: usize,
    pub paired_peers: usize,
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
    ClipboardChanged { content: ClipboardContent },

    /// 剪切板内容已成功同步到至少一个设备
    ClipboardSynced { peer_count: usize },

    /// 操作失败（一次性错误）
    Error { message: String },
}
