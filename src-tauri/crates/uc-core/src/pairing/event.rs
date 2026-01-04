#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PairingEvent {
    /// 收到来自对端设备的配对请求  
    ///
    /// 这是一个**网络层已发生的事实**，表示本设备被动进入配对流程。
    /// - 通常由 libp2p / QUIC 收到 pairing request 触发
    /// - 只描述“请求已经到达”，不包含是否接受
    IncomingRequest,

    /// 本地用户明确“接受”了本次配对请求  
    ///
    /// 这是一个**用户意图已经落地的事实**：
    /// - UI 上点击了「接受配对」
    /// - CLI 中输入了确认指令
    /// - 一旦发生，不可撤销
    UserAccepted,

    /// 本地用户明确“拒绝”了本次配对请求  
    ///
    /// 表示用户主动终止配对流程：
    /// - 不论当前处于哪个阶段，都会导致配对被拒绝
    /// - 属于用户主权行为，而非系统失败
    UserRejected,

    /// 收到对端发送的挑战信息（通常包含 PIN 或其派生信息）  
    ///
    /// 这是 Initiator 侧进入校验阶段的标志：
    /// - 表示对端已经同意配对
    /// - 配对流程进入“需要用户确认”的阶段
    ChallengeReceived,

    /// 本地用户已完成 PIN 输入并得到校验结果  
    ///
    /// - `success = true`：PIN 校验通过，流程可继续
    /// - `success = false`：PIN 校验失败，配对应立即失败
    ///
    /// 注意：这是“校验结果已产生”的事实，
    /// 而不是“开始校验”的命令
    PinVerified {
        success: bool,
    },

    /// Responder 收到 Initiator 对挑战的回应（PIN 校验结果）
    ResponseReceived {
        success: bool,
    },

    /// 收到对端发送的最终确认结果  
    ///
    /// 表示对端已经完成最终确认：
    /// - `success = true`：双方确认一致，可进入已配对状态
    /// - `success = false`：对端确认失败，配对终止
    ///
    /// 这是配对流程中**最后一个关键网络事实**
    ConfirmReceived {
        success: bool,
    },

    /// 配对流程因超时而被系统判定为失效  
    ///
    /// - 通常由定时器 / watchdog 触发
    /// - 属于系统行为，而非用户或对端的直接操作
    /// - 一旦发生，配对会话应被视为过期
    Timeout,
}
