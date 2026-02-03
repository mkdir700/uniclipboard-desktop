use chrono::{DateTime, Utc};

use crate::{ids::SpaceId, SessionId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpaceAccessAction {
    // ===== Sponsor intents =====

    // 请求准备一个 Offer（由 infra / crypto 构造）
    RequestOfferPreparation {
        pairing_session_id: SessionId,
        space_id: SpaceId,
        expires_at: DateTime<Utc>,
    },

    // 请求将已准备好的 Offer 发送出去
    SendOffer,

    // ===== Joiner intents =====

    // 用户已提交 passphrase，请求派生空间密钥
    RequestSpaceKeyDerivation {
        space_id: SpaceId,
    },

    // 请求发送 Joiner 的证明
    SendProof,

    // ===== Result intents =====

    // 请求发送最终裁决（Granted / Denied）
    SendResult,

    // ===== Persistence intents =====

    // Joiner 授权成功，需要持久化结果
    PersistJoinerAccess {
        space_id: SpaceId,
    },

    // Sponsor 授权成功，需要记录配对设备
    PersistSponsorAccess {
        space_id: SpaceId,
    },

    // ===== Housekeeping =====
    StartTimer {
        ttl_secs: u64,
    },
    StopTimer,
}
