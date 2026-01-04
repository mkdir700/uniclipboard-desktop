use super::{SyncEvent, SyncState};
use crate::decision::DomainDecision;

pub struct SyncDomain {
    state: SyncState,
    last_hash: Option<String>,
}

impl SyncDomain {
    pub fn new() -> Self {
        Self {
            state: SyncState::Idle,
            last_hash: None,
        }
    }

    pub fn apply(&mut self, event: SyncEvent) -> DomainDecision {
        match (&self.state, event) {
            // 本地变化：开始同步
            (SyncState::Idle, SyncEvent::LocalClipboardChanged { payload }) => {
                self.state = SyncState::Sending;
                self.last_hash = Some(payload.content_hash());

                DomainDecision::BroadcastClipboard { payload }
            }

            // 远端变化
            (
                SyncState::Idle,
                SyncEvent::RemoteClipboardReceived {
                    payload,
                    content_hash,
                    origin,
                },
            ) => {
                // 重复内容
                if self.last_hash.as_deref() == Some(&content_hash) {
                    return DomainDecision::Ignore;
                }

                self.state = SyncState::Receiving;
                self.last_hash = Some(content_hash);

                DomainDecision::ApplyRemoteClipboard { payload, origin }
            }

            // 同步中又来了新内容 → 冲突
            (
                SyncState::Receiving,
                SyncEvent::RemoteClipboardReceived {
                    payload, origin, ..
                },
            ) => {
                self.state = SyncState::Conflict;

                DomainDecision::EnterConflict { payload, origin }
            }

            _ => DomainDecision::Ignore,
        }
    }
}
