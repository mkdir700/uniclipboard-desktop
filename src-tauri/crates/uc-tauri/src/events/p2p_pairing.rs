use serde::{Deserialize, Serialize};

const DEPRECATED_REASON: &str = "legacy event name; will be replaced by p2p-pairing-verification";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct P2PPairingRequestEvent {
    pub session_id: String,
    pub peer_id: String,
    pub device_name: Option<String>,
    pub deprecated: bool,
    pub deprecated_reason: String,
}

impl P2PPairingRequestEvent {
    pub fn deprecated(session_id: &str, peer_id: &str, device_name: Option<String>) -> Self {
        Self {
            session_id: session_id.to_string(),
            peer_id: peer_id.to_string(),
            device_name,
            deprecated: true,
            deprecated_reason: DEPRECATED_REASON.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct P2PPinReadyEvent {
    pub session_id: String,
    pub pin: String,
    pub peer_device_name: Option<String>,
    pub short_code: Option<String>,
    pub local_fingerprint: Option<String>,
    pub peer_fingerprint: Option<String>,
    pub deprecated: bool,
    pub deprecated_reason: String,
}

impl P2PPinReadyEvent {
    pub fn deprecated(
        session_id: &str,
        pin: String,
        peer_device_name: Option<String>,
        short_code: Option<String>,
        local_fingerprint: Option<String>,
        peer_fingerprint: Option<String>,
    ) -> Self {
        Self {
            session_id: session_id.to_string(),
            pin,
            peer_device_name,
            short_code,
            local_fingerprint,
            peer_fingerprint,
            deprecated: true,
            deprecated_reason: DEPRECATED_REASON.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct P2PPairingCompleteEvent {
    pub session_id: String,
    pub peer_id: String,
    pub device_name: Option<String>,
    pub deprecated: bool,
    pub deprecated_reason: String,
}

impl P2PPairingCompleteEvent {
    pub fn deprecated(session_id: &str, peer_id: &str, device_name: Option<String>) -> Self {
        Self {
            session_id: session_id.to_string(),
            peer_id: peer_id.to_string(),
            device_name,
            deprecated: true,
            deprecated_reason: DEPRECATED_REASON.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct P2PPairingFailedEvent {
    pub session_id: String,
    pub error: String,
    pub deprecated: bool,
    pub deprecated_reason: String,
}

impl P2PPairingFailedEvent {
    pub fn deprecated(session_id: &str, error: String) -> Self {
        Self {
            session_id: session_id.to_string(),
            error,
            deprecated: true,
            deprecated_reason: DEPRECATED_REASON.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairing_request_payload_includes_deprecation_marker() {
        let payload =
            P2PPairingRequestEvent::deprecated("session-1", "peer-1", Some("Device".to_string()));
        assert!(payload.deprecated);
        assert!(!payload.deprecated_reason.is_empty());
    }
}
