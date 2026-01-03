//! P2P-related data models
//!
//! Pure data structures for P2P networking without infrastructure dependencies.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A peer discovered via mDNS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPeer {
    /// libp2p peer ID
    pub peer_id: String,
    /// Device name (if available)
    pub device_name: Option<String>,
    /// 6-digit device ID (from Identify agent_version)
    pub device_id: Option<String>,
    /// Known multiaddresses
    pub addresses: Vec<String>,
    /// When this peer was discovered
    pub discovered_at: DateTime<Utc>,
    /// Whether this peer is already paired
    pub is_paired: bool,
}

impl DiscoveredPeer {
    /// Create a new discovered peer
    pub fn new(peer_id: String) -> Self {
        Self {
            peer_id,
            device_name: None,
            device_id: None,
            addresses: Vec::new(),
            discovered_at: Utc::now(),
            is_paired: false,
        }
    }

    /// Get display name for this peer
    pub fn display_name(&self) -> String {
        self.device_name
            .clone()
            .or_else(|| self.device_id.clone())
            .unwrap_or_else(|| self.short_peer_id())
    }

    /// Get short peer ID (first 8 characters)
    pub fn short_peer_id(&self) -> String {
        self.peer_id.chars().take(8).collect()
    }
}

/// A paired peer (stored in database)
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairedPeer {
    /// libp2p peer ID
    pub peer_id: String,
    /// Device name
    pub device_name: String,
    /// Shared secret for encrypted communication
    pub shared_secret: Vec<u8>,
    /// When pairing was completed
    pub paired_at: DateTime<Utc>,
    /// Last time this peer was seen
    pub last_seen: Option<DateTime<Utc>>,
    /// Last known multiaddresses
    pub last_known_addresses: Vec<String>,
}

impl PairedPeer {
    /// Create a new paired peer
    pub fn new(
        peer_id: String,
        device_name: String,
        shared_secret: Vec<u8>,
    ) -> Self {
        Self {
            peer_id,
            device_name,
            shared_secret,
            paired_at: Utc::now(),
            last_seen: None,
            last_known_addresses: Vec::new(),
        }
    }

    /// Get short peer ID (first 8 characters)
    pub fn short_peer_id(&self) -> String {
        self.peer_id.chars().take(8).collect()
    }
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

/// A peer we have an active connection with
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectedPeer {
    /// libp2p peer ID
    pub peer_id: String,
    /// Device name
    pub device_name: String,
    /// When the connection was established
    pub connected_at: DateTime<Utc>,
}

impl ConnectedPeer {
    /// Create a new connected peer
    pub fn new(peer_id: String, device_name: String) -> Self {
        Self {
            peer_id,
            device_name,
            connected_at: Utc::now(),
        }
    }

    /// Get short peer ID (first 8 characters)
    pub fn short_peer_id(&self) -> String {
        self.peer_id.chars().take(8).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovered_peer_new() {
        let peer = DiscoveredPeer::new("12D3KooW...".to_string());

        assert_eq!(peer.peer_id, "12D3KooW...");
        assert!(peer.device_name.is_none());
        assert!(peer.device_id.is_none());
        assert!(peer.addresses.is_empty());
        assert!(!peer.is_paired);
    }

    #[test]
    fn test_discovered_peer_display_name() {
        let mut peer = DiscoveredPeer::new("12D3KooW...".to_string());

        // Default: short peer ID
        assert_eq!(peer.display_name(), "12D3KooW");

        // device_id takes priority
        peer.device_id = Some("ABC123".to_string());
        assert_eq!(peer.display_name(), "ABC123");

        // device_name takes highest priority
        peer.device_name = Some("My Device".to_string());
        assert_eq!(peer.display_name(), "My Device");
    }

    #[test]
    fn test_paired_peer_new() {
        let peer = PairedPeer::new(
            "12D3KooW...".to_string(),
            "Test Device".to_string(),
            vec![1, 2, 3, 4],
        );

        assert_eq!(peer.peer_id, "12D3KooW...");
        assert_eq!(peer.device_name, "Test Device");
        assert_eq!(peer.shared_secret, vec![1, 2, 3, 4]);
        assert!(peer.last_seen.is_none());
        assert!(peer.last_known_addresses.is_empty());
    }

    #[test]
    fn test_paired_peer_debug_redacts_secret() {
        let peer = PairedPeer::new(
            "12D3KooW...".to_string(),
            "Test Device".to_string(),
            vec![1, 2, 3, 4],
        );

        let debug_str = format!("{:?}", peer);
        assert!(debug_str.contains("[REDACTED]"));
        assert!(!debug_str.contains("[1, 2, 3, 4]"));
    }

    #[test]
    fn test_connected_peer_new() {
        let peer = ConnectedPeer::new("12D3KooW...".to_string(), "Test Device".to_string());

        assert_eq!(peer.peer_id, "12D3KooW...");
        assert_eq!(peer.device_name, "Test Device");
    }

    #[test]
    fn test_peer_serialization() {
        let peer = DiscoveredPeer {
            peer_id: "12D3KooW...".to_string(),
            device_name: Some("Test Device".to_string()),
            device_id: Some("ABC123".to_string()),
            addresses: vec!["/ip4/192.168.1.100/tcp/12345".to_string()],
            discovered_at: Utc::now(),
            is_paired: true,
        };

        let json = serde_json::to_string(&peer).unwrap();
        let deserialized: DiscoveredPeer = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.peer_id, "12D3KooW...");
        assert_eq!(deserialized.device_name, Some("Test Device".to_string()));
        assert_eq!(deserialized.device_id, Some("ABC123".to_string()));
        assert_eq!(deserialized.addresses.len(), 1);
        assert!(deserialized.is_paired);
    }

    #[test]
    fn test_short_peer_id() {
        let peer = DiscoveredPeer::new("12D3KooWABCDEF".to_string());
        assert_eq!(peer.short_peer_id(), "12D3KooW");

        let paired = PairedPeer::new("12D3KooWABCDEF".to_string(), "Test".to_string(), vec![]);
        assert_eq!(paired.short_peer_id(), "12D3KooW");

        let connected = ConnectedPeer::new("12D3KooWABCDEF".to_string(), "Test".to_string());
        assert_eq!(connected.short_peer_id(), "12D3KooW");
    }
}
