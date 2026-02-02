use serde::{Deserialize, Serialize};

use super::protocol::{SpaceAccessOffer, SpaceAccessProof, SpaceAccessResult};
use crate::{ids::SpaceId, network::SessionId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpaceAccessEvent {
    // 由 Setup/Orchestrator 触发：开始 JoinSpaceAccess（建立在 pairing 完成后的 session）
    StartAsJoiner {
        pairing_session_id: SessionId,
        ttl_secs: u64,
    },
    StartAsSponsor {
        pairing_session_id: SessionId,
        space_id: SpaceId,
        ttl_secs: u64,
    },

    // 网络回调：收到了对方消息
    ReceivedOffer(SpaceAccessOffer),
    ReceivedProof(SpaceAccessProof),
    ReceivedResult(SpaceAccessResult),

    // 用户输入：Joiner 提交 passphrase
    SubmitPassphrase {
        passphrase: String,
    },

    // 用户取消
    CancelByUser,

    // 定时器/系统事件
    Timeout,
    SessionClosed,
}
