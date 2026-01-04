use crate::{decision::DomainDecision, pairing::PairingEvent, PairingState};

pub struct PairingDomain {
    state: PairingState,
}

impl PairingDomain {
    pub fn new() -> Self {
        Self {
            state: PairingState::Idle,
        }
    }

    pub fn apply(&mut self, event: PairingEvent) -> DomainDecision {
        use PairingEvent::*;
        use PairingState::*;

        match (&self.state, event) {
            // =========================================================
            // Responder flow（接收方流程）
            // =========================================================

            // 空闲中收到对端配对请求：进入“收到请求，等待用户决策”状态
            (Idle, PairingEvent::IncomingRequest) => {
                self.state = PairingState::IncomingRequest;
                DomainDecision::IncomingPairingRequest
            }

            // 用户接受该配对请求：进入“已接受，准备发 challenge 并等待对端 response”状态
            (PairingState::IncomingRequest, UserAccepted) => {
                self.state = PendingResponse;
                DomainDecision::SendChallenge
            }

            // 用户拒绝该配对请求：进入 Rejected 终态（用户主权终止）
            (PairingState::IncomingRequest, UserRejected) => {
                self.state = PairingState::Rejected;
                DomainDecision::RejectPairing
            }

            // 接收方收到发起方对 challenge 的响应，且校验通过：
            // 进入“等待最终确认收敛”状态（通常下一步是等待/发送 confirm）
            (PendingResponse, PairingEvent::ResponseReceived { success: true }) => {
                self.state = WaitingConfirm;
                DomainDecision::SendConfirm
            }

            // 接收方收到发起方的响应但校验失败：进入 Failed 终态
            (PendingResponse, PairingEvent::ResponseReceived { success: false }) => {
                self.state = Failed;
                DomainDecision::PairingFailed
            }

            // =========================================================
            // Initiator flow（发起方流程）
            // =========================================================

            // 发起方已发送 pairing request，收到对端 challenge：
            // 进入“等待用户核对 PIN / challenge”状态
            (Requesting, ChallengeReceived) => {
                self.state = PendingChallenge;
                DomainDecision::PresentChallengeToUser
            }

            // 用户校验 PIN 成功：进入 Verifying（已验证，等待最终确认）状态
            // 注意：发送 response 属于应用层副作用，不在 domain 中执行
            (PendingChallenge, PinVerified { success: true }) => {
                self.state = Verifying;
                DomainDecision::SendResponse
            }

            // 用户校验 PIN 失败：进入 Failed 终态
            (PendingChallenge, PinVerified { success: false }) => {
                self.state = Failed;
                DomainDecision::PairingFailed
            }

            // =========================================================
            // Shared（双方共享的最终确认逻辑）
            // =========================================================

            // 在 Verifying 或 WaitingConfirm 阶段收到最终确认成功：配对完成
            (Verifying, ConfirmReceived { success: true })
            | (WaitingConfirm, ConfirmReceived { success: true }) => {
                self.state = Paired;
                DomainDecision::PairingSucceeded
            }

            // 在 Verifying 或 WaitingConfirm 阶段收到最终确认失败：配对失败
            (Verifying, ConfirmReceived { success: false })
            | (WaitingConfirm, ConfirmReceived { success: false }) => {
                self.state = Failed;
                DomainDecision::PairingFailed
            }

            // =========================================================
            // Global（全局规则）
            // =========================================================

            // 任何“活跃配对中”的状态发生超时：进入 Expired 终态
            (state, Timeout) if state.is_active() => {
                self.state = Expired;
                DomainDecision::PairingExpired
            }

            // 无论任何状态，用户拒绝/取消：进入 Rejected 终态
            // （用于支持：流程中途用户取消配对）
            (_, UserRejected) => {
                self.state = Rejected;
                DomainDecision::PairingFailed
            }

            // 其它组合一律视为非法转移：
            // - 重复消息
            // - 乱序消息
            // - 协议不一致
            // 应用层可选择：忽略 / 记录告警 / 触发失败策略
            _ => DomainDecision::Ignore,
        }
    }
}
