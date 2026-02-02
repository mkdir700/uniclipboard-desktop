use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub use crate::security::KeySlot;
use crate::{ids::SpaceId, network::SessionId, security::protocol::DenyReason};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpaceAccessState {
    Idle,

    // Joiner：等待 Sponsor 的 Offer
    WaitingOffer {
        pairing_session_id: SessionId,
        expires_at: DateTime<Utc>,
    },

    // Joiner：收到 Offer，等待用户输入 passphrase
    WaitingPassphrase {
        pairing_session_id: SessionId,
        space_id: SpaceId,
        keyslot_blob: Vec<u8>,
        challenge_nonce: [u8; 32],
        expires_at: DateTime<Utc>,
    },

    // Joiner：已发送 proof，等待结果
    WaitingResult {
        pairing_session_id: SessionId,
        space_id: SpaceId,
        challenge_nonce: [u8; 32],
        sent_at: DateTime<Utc>,
    },

    // Sponsor：已发出 Offer，等待 proof
    WaitingProof {
        pairing_session_id: SessionId,
        space_id: SpaceId,
        challenge_nonce: [u8; 32],
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
pub enum CancelReason {
    UserCancelled,
    Timeout,
    SessionClosed,
}
