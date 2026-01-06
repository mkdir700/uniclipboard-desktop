use super::snapshot::ClipboardDecisionSnapshot;

#[derive(Debug, Clone)]
pub enum ClipboardContentActionDecision {
    Allow { snapshot: ClipboardDecisionSnapshot },
    Reject { reason: RejectReason },
}

#[derive(Debug, Clone)]
pub enum RejectReason {
    NotFound,
    Expired,
    Sensitive,
    PolicyDenied,
}
