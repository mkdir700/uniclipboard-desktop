use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub use crate::security::KeySlot;
use crate::{ids::SpaceId, network::SessionId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpaceAccessState {
    Idle,

    // Joiner：已开始加入流程，等待 Sponsor 的 Offer
    WaitingOffer {
        pairing_session_id: SessionId,
        expires_at: DateTime<Utc>,
    },

    // Joiner：Offer 已被接受，等待用户输入 passphrase
    WaitingUserPassphrase {
        pairing_session_id: SessionId,
        space_id: SpaceId,
        expires_at: DateTime<Utc>,
    },

    // Joiner：用户已提交 passphrase，等待 Sponsor 裁决
    WaitingDecision {
        pairing_session_id: SessionId,
        space_id: SpaceId,
        sent_at: DateTime<Utc>,
    },

    // Sponsor：已发起授权，等待 Joiner 的证明
    WaitingJoinerProof {
        pairing_session_id: SessionId,
        space_id: SpaceId,
        expires_at: DateTime<Utc>,
    },

    Granted {
        pairing_session_id: SessionId,
        space_id: SpaceId,
    },

    Denied {
        pairing_session_id: SessionId,
        space_id: SpaceId,
        reason: DenyReason,
    },

    Cancelled {
        pairing_session_id: SessionId,
        reason: CancelReason,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DenyReason {
    Expired,
    InvalidProof,
    SpaceMismatch,
    SessionMismatch,
    InternalError,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CancelReason {
    UserCancelled,
    Timeout,
    SessionClosed,
}
