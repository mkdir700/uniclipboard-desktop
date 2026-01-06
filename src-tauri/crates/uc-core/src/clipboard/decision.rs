use super::snapshot::ClipboardDecisionSnapshot;

#[derive(Debug, Clone)]
pub enum ClipboardContentActionDecision {
    Allow,
    Reject { reason: RejectReason },
}

#[derive(Debug, Clone)]
pub enum RejectReason {
    NotFound,
    Expired,
    Sensitive,
    PolicyDenied,
    InternalError,
}
