use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use uc_core::{ids::SpaceId, network::SessionId, security::space_access::state::DenyReason};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SpaceAccessMsg {
    // Sponsor -> Joiner
    Offer(SpaceAccessOffer),

    // Joiner -> Sponsor
    Proof(SpaceAccessProof),

    // Sponsor -> Joiner
    Result(SpaceAccessResult),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceAccessOffer {
    pub pairing_session_id: SessionId,
    pub space_id: SpaceId,
    pub keyslot_blob: Vec<u8>, // 由 Sponsor 导出给 Joiner 的 keyslot（密封）
    pub challenge_nonce: [u8; 32], // K（随机）
    pub expires_at: DateTime<Utc>,
    pub version: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceAccessProof {
    pub pairing_session_id: SessionId,
    pub space_id: SpaceId,
    pub challenge_nonce: [u8; 32], // 回显，防串包
    pub proof: Vec<u8>,            // 例如: AEAD(K, key=space_key, aad=transcript)
    pub client_nonce: [u8; 32],    // Joiner 生成，增加双向唯一性
    pub version: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpaceAccessResult {
    Granted {
        pairing_session_id: SessionId,
        space_id: SpaceId,
    },
    Denied {
        pairing_session_id: SessionId,
        space_id: SpaceId,
        reason: DenyReason,
    },
}
