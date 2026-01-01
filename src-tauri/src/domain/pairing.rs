use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairedPeer {
    pub peer_id: String,
    pub device_name: String,
    pub shared_secret: Vec<u8>,
    pub paired_at: DateTime<Utc>,
    pub last_seen: Option<DateTime<Utc>>,
    pub last_known_addresses: Vec<String>,
}

impl std::fmt::Debug for PairedPeer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PairedPeer")
            .field("peer_id", &self.peer_id)
            .field("device_name", &self.device_name)
            .field("shared_secret", &"[REDACTED]")
            .field("paired_at", &self.paired_at)
            .field("last_seen", &self.last_seen)
            .field("last_known_addresses", &self.last_known_addresses)
            .finish()
    }
}
