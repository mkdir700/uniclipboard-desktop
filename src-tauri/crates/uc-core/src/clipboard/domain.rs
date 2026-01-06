use base64::alphabet::ParseAlphabetError;

use super::decision::ClipboardContentActionDecision;
use super::event::ClipboardContentActionEvent;
use crate::clipboard::decision::RejectReason;
use crate::clipboard::event::ClipboardContentAction;
use crate::ports::ClipboardHistoryPort;

/// Checks a permission boolean and returns the appropriate decision
fn check_permission(permission: bool) -> ClipboardContentActionDecision {
    if permission {
        ClipboardContentActionDecision::Allow
    } else {
        ClipboardContentActionDecision::Reject {
            reason: RejectReason::PolicyDenied,
        }
    }
}

pub struct ClipboardContentDecisionDomain<H>
where
    H: ClipboardHistoryPort,
{
    history: H,
}

impl<H> ClipboardContentDecisionDomain<H>
where
    H: ClipboardHistoryPort,
{
    pub fn new(history: H) -> Self {
        Self { history }
    }

    pub async fn apply(
        &self,
        event: ClipboardContentActionEvent,
    ) -> ClipboardContentActionDecision {
        match event {
            ClipboardContentActionEvent::UserRequested { content_hash, .. } => {
                match self.history.get_snapshot_decision(&content_hash).await {
                    Ok(Some(snapshot)) if snapshot.is_usable() => {
                        ClipboardContentActionDecision::Allow
                    }
                    Ok(Some(_)) | Ok(None) => ClipboardContentActionDecision::Reject {
                        reason: RejectReason::NotFound,
                    },
                    Err(_) => ClipboardContentActionDecision::Reject {
                        reason: RejectReason::InternalError,
                    },
                }
            }
        }
    }
}
