//! Paired peer domain model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Information about a paired peer device
#[derive(Clone, Serialize, Deserialize)]
pub struct PairedPeer {
    /// libp2p PeerId
    pub peer_id: String,
    /// Human-readable device name
    pub device_name: String,
    /// Shared secret (no longer used with unified encryption)
    #[serde(default)]
    pub shared_secret: Vec<u8>,
    /// When pairing was completed
    pub paired_at: DateTime<Utc>,
    /// Last time this peer was seen
    pub last_seen: Option<DateTime<Utc>>,
    /// Known addresses for this peer
    pub last_known_addresses: Vec<String>,
}

impl fmt::Debug for PairedPeer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

impl PartialEq for PairedPeer {
    fn eq(&self, other: &Self) -> bool {
        self.peer_id == other.peer_id
    }
}

impl Eq for PairedPeer {}
