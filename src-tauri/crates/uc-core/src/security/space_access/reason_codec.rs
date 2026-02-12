use super::state::DenyReason;

pub const DENY_REASON_EXPIRED: &str = "expired";
pub const DENY_REASON_INVALID_PROOF: &str = "invalid_proof";
pub const DENY_REASON_SPACE_MISMATCH: &str = "space_mismatch";
pub const DENY_REASON_SESSION_MISMATCH: &str = "session_mismatch";
pub const DENY_REASON_INTERNAL_ERROR: &str = "internal_error";

pub fn deny_reason_to_code(reason: &DenyReason) -> &'static str {
    match reason {
        DenyReason::Expired => DENY_REASON_EXPIRED,
        DenyReason::InvalidProof => DENY_REASON_INVALID_PROOF,
        DenyReason::SpaceMismatch => DENY_REASON_SPACE_MISMATCH,
        DenyReason::SessionMismatch => DENY_REASON_SESSION_MISMATCH,
        DenyReason::InternalError => DENY_REASON_INTERNAL_ERROR,
    }
}

pub fn deny_reason_from_code(code: &str) -> Option<DenyReason> {
    match code {
        DENY_REASON_EXPIRED => Some(DenyReason::Expired),
        DENY_REASON_INVALID_PROOF => Some(DenyReason::InvalidProof),
        DENY_REASON_SPACE_MISMATCH => Some(DenyReason::SpaceMismatch),
        DENY_REASON_SESSION_MISMATCH => Some(DenyReason::SessionMismatch),
        DENY_REASON_INTERNAL_ERROR => Some(DenyReason::InternalError),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deny_reason_round_trip_all_variants() {
        let variants = [
            DenyReason::Expired,
            DenyReason::InvalidProof,
            DenyReason::SpaceMismatch,
            DenyReason::SessionMismatch,
            DenyReason::InternalError,
        ];

        for reason in variants {
            let code = deny_reason_to_code(&reason);
            let decoded = deny_reason_from_code(code);
            assert_eq!(decoded, Some(reason));
        }
    }

    #[test]
    fn deny_reason_from_code_returns_none_for_unknown_code() {
        assert_eq!(deny_reason_from_code("unknown_reason"), None);
    }
}
