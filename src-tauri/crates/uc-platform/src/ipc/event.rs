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
    // =========================================================
    // 1. 生命周期（platform 本身）
    // =========================================================
    /// 平台启动完成（runtime 已就绪）
    Started,

    /// 平台已停止（所有后台任务已退出）
    Stopped,

    /// 平台进入不可用状态（例如 fatal error）
    Crashed { reason: String },

    // =========================================================
    // 2. 状态快照（给 UI/CLI 用）
    // =========================================================
    /// 平台整体状态发生变化
    StatusUpdated(PlatformStatus),

    // =========================================================
    // 3. 用户可感知事件（产品级）
    // =========================================================
    /// 本地剪切板发生变化
    ClipboardChanged { content: ClipboardContent },

    /// 收到新的配对请求
    PairingRequested {
        request_id: String,
        peer_id: String,
        device_name: String,
    },

    /// 与某个设备完成配对
    Paired {
        peer_id: String,
        device_name: String,
    },

    /// 与某个设备断开配对
    Unpaired { peer_id: String },

    /// 剪切板内容已成功同步到至少一个设备
    ClipboardSynced { peer_count: usize },

    // =========================================================
    // 4. 后台运行事件（非交互，但 UI 可能展示）
    // =========================================================
    /// 网络连接已建立（至少一个可用连接）
    NetworkConnected,

    /// 网络连接已全部断开
    NetworkDisconnected,

    /// 后台服务被暂停（例如系统睡眠）
    Suspended,

    /// 后台服务已恢复
    Resumed,

    // =========================================================
    // 5. 问题与异常（统一出口）
    // =========================================================
    /// 可恢复问题（不影响整体运行）
    Warning { message: String },

    /// 操作失败（一次性错误）
    Error { message: String },
}
