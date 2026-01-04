use crate::clipboard::Payload;
use crate::ids::DeviceId;

/// Domain 对外输出的统一“业务决策”
///
/// 注意：
/// - 不包含 IO / async
/// - 不包含时间
/// - 不包含技术细节
#[derive(Debug, Clone)]
pub enum DomainDecision {
    /// 什么都不做
    Ignore,

    /// =============== 剪切板同步 ============

    /// 本地剪贴板内容成为当前版本（需要被持久化）
    PersistLocalClipboard { payload: Payload },

    /// 将本地内容广播给其他设备
    BroadcastClipboard { payload: Payload },

    /// 将远端内容应用到本地
    ApplyRemoteClipboard { payload: Payload, origin: DeviceId },

    /// 发现同步冲突
    EnterConflict { payload: Payload, origin: DeviceId },

    /// =============== 配对 ==================

    /// 收到对端配对请求，需要用户决策
    IncomingPairingRequest,

    /// 发送 pairing challenge 给对端
    SendChallenge,

    /// 向对端发送 challenge response
    SendResponse,

    /// 向对端发送最终确认
    SendConfirm,

    /// 向对端拒绝配对（或仅本地拒绝）
    RejectPairing,

    /// 向用户展示 challenge / PIN 以供核对
    PresentChallengeToUser,

    /// 配对成功（进入 Paired 终态）
    PairingSucceeded,

    /// 配对失败（校验失败 / 协议失败）
    PairingFailed,

    /// 配对超时
    PairingExpired,
}
