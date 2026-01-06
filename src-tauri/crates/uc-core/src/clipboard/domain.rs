use super::decision::ClipboardContentActionDecision;
use super::event::ClipboardContentActionEvent;
use crate::clipboard::decision::RejectReason;
use crate::ports::ClipboardHistoryPort;

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
            ClipboardContentActionEvent::UserRequested {
                content_hash,
                action,
            } => match self.history.get_snapshot_decision(&content_hash).await {
                Ok(Some(snapshot)) => ClipboardContentActionDecision::Allow { snapshot },
                Ok(None) => ClipboardContentActionDecision::Reject {
                    reason: RejectReason::NotFound,
                },
                Err(_) => CopyFromHistoryDecision::Reject {
                    reason: RejectReason::PolicyDenied,
                },
            },
        }
    }
}
