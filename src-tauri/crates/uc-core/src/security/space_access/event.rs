use chrono::{DateTime, Utc};

use crate::{ids::SpaceId, network::SessionId, security::space_access::state::DenyReason};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpaceAccessEvent {
    // 流程启动（由 App / Orchestrator 触发）
    JoinRequested {
        pairing_session_id: SessionId,
        ttl_secs: u64,
    },
    SponsorAuthorizationRequested {
        pairing_session_id: SessionId,
        space_id: SpaceId,
        ttl_secs: u64,
    },

    // ===== Offer 阶段 =====

    // Joiner：Offer 已被成功接收并校验
    OfferAccepted {
        pairing_session_id: SessionId,
        space_id: SpaceId,
        expires_at: DateTime<Utc>,
    },

    // ===== 用户输入 =====

    // Joiner：用户已提交 passphrase（事实）
    PassphraseSubmitted,

    // ===== Proof 阶段 =====

    // Sponsor：Joiner 的证明已被校验通过
    ProofVerified {
        pairing_session_id: SessionId,
        space_id: SpaceId,
    },

    // Sponsor：Joiner 的证明无效
    ProofRejected {
        pairing_session_id: SessionId,
        space_id: SpaceId,
        reason: DenyReason,
    },

    // ===== 裁决 =====

    // Joiner：收到最终裁决
    AccessGranted {
        pairing_session_id: SessionId,
        space_id: SpaceId,
    },
    AccessDenied {
        pairing_session_id: SessionId,
        space_id: SpaceId,
        reason: DenyReason,
    },

    // ===== 控制流 =====
    CancelledByUser,
    Timeout,
    SessionClosed,
}
