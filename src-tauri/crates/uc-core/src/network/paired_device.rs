use crate::PeerId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PairingState {
    Pending,
    Trusted,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairedDevice {
    pub peer_id: PeerId,
    pub pairing_state: PairingState,
    pub identity_fingerprint: String,
    pub paired_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paired_device_serialization() {
        let device = PairedDevice {
            peer_id: PeerId::from("12D3KooW..."),
            pairing_state: PairingState::Trusted,
            identity_fingerprint: "fp".to_string(),
            paired_at: Utc::now(),
            last_seen_at: None,
        };

        let json = serde_json::to_string(&device).unwrap();
        let restored: PairedDevice = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.pairing_state, PairingState::Trusted);
        assert_eq!(restored.identity_fingerprint, device.identity_fingerprint);
    }
}
