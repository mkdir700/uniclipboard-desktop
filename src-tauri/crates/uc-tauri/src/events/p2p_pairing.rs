use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct P2PPairingVerificationEvent {
    pub session_id: String,
    pub kind: P2PPairingVerificationKind,
    pub peer_id: Option<String>,
    pub device_name: Option<String>,
    pub code: Option<String>,
    pub local_fingerprint: Option<String>,
    pub peer_fingerprint: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum P2PPairingVerificationKind {
    Request,
    Verification,
    Complete,
    Failed,
}

impl P2PPairingVerificationEvent {
    pub fn request(session_id: &str, peer_id: String, device_name: Option<String>) -> Self {
        Self {
            session_id: session_id.to_string(),
            kind: P2PPairingVerificationKind::Request,
            peer_id: Some(peer_id),
            device_name,
            code: None,
            local_fingerprint: None,
            peer_fingerprint: None,
            error: None,
        }
    }

    pub fn verification(
        session_id: &str,
        device_name: Option<String>,
        code: String,
        local_fingerprint: String,
        peer_fingerprint: String,
    ) -> Self {
        Self {
            session_id: session_id.to_string(),
            kind: P2PPairingVerificationKind::Verification,
            peer_id: None,
            device_name,
            code: Some(code),
            local_fingerprint: Some(local_fingerprint),
            peer_fingerprint: Some(peer_fingerprint),
            error: None,
        }
    }

    pub fn complete(session_id: &str, peer_id: String, device_name: Option<String>) -> Self {
        Self {
            session_id: session_id.to_string(),
            kind: P2PPairingVerificationKind::Complete,
            peer_id: Some(peer_id),
            device_name,
            code: None,
            local_fingerprint: None,
            peer_fingerprint: None,
            error: None,
        }
    }

    pub fn failed(session_id: &str, error: String) -> Self {
        Self {
            session_id: session_id.to_string(),
            kind: P2PPairingVerificationKind::Failed,
            peer_id: None,
            device_name: None,
            code: None,
            local_fingerprint: None,
            peer_fingerprint: None,
            error: Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verification_event_serializes_kind() {
        let payload = P2PPairingVerificationEvent::request(
            "session-1",
            "peer-1".to_string(),
            Some("Device".to_string()),
        );
        let value = serde_json::to_value(payload).expect("serialize event");
        assert_eq!(value["kind"], "request");
    }
}
