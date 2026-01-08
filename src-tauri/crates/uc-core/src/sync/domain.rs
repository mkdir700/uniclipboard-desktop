use super::{SyncEvent, SyncState};
use crate::{clipboard::ContentHash, decision::DomainDecision};

pub struct SyncDomain {
    state: SyncState,
    last_hash: Option<ContentHash>,
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
            (SyncState::Idle, SyncEvent::LocalClipboardChanged { content }) => {
                self.state = SyncState::Sending;
                self.last_hash = Some(content.content_hash());

                DomainDecision::BroadcastClipboard { content: content }
            }

            // 远端变化
            (
                SyncState::Idle,
                SyncEvent::RemoteClipboardReceived {
                    content,
                    content_hash,
                    origin,
                },
            ) => {
                // 重复内容
                if self.last_hash == Some(content_hash.clone()) {
                    return DomainDecision::Ignore;
                }

                self.state = SyncState::Receiving;
                self.last_hash = Some(content_hash);

                DomainDecision::ApplyRemoteClipboard {
                    content: content,
                    origin,
                }
            }

            // 同步中又来了新内容 → 冲突
            (
                SyncState::Receiving,
                SyncEvent::RemoteClipboardReceived {
                    content, origin, ..
                },
            ) => {
                self.state = SyncState::Conflict;

                DomainDecision::EnterConflict {
                    content: content,
                    origin,
                }
            }

            _ => DomainDecision::Ignore,
        }
    }
}
