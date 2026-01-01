use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::protocol::{ClipboardMessage, PairingConfirm, PairingRequest, PairingResponse};

/// Network status for P2P connection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NetworkStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// A peer discovered via mDNS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPeer {
    pub peer_id: String,
    pub device_name: Option<String>,
    pub addresses: Vec<String>,
    pub discovered_at: DateTime<Utc>,
    pub is_paired: bool,
}

/// A peer we have an active connection with
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectedPeer {
    pub peer_id: String,
    pub device_name: String,
    pub connected_at: DateTime<Utc>,
}

/// Network events emitted by NetworkManager
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    // Discovery events
    PeerDiscovered(DiscoveredPeer),
    PeerLost(String), // peer_id
    /// A peer's device name was updated (via DeviceAnnounce message or Identify)
    PeerNameUpdated {
        peer_id: String,
        device_name: String,
    },

    // Connection events
    PeerConnected(ConnectedPeer),
    PeerDisconnected(String), // peer_id

    // Readiness events (protocol-agnostic)
    /// A peer is now ready to receive broadcast messages
    PeerReady {
        peer_id: String,
    },
    /// A peer is no longer ready to receive broadcast messages
    PeerNotReady {
        peer_id: String,
    },

    // Pairing events
    PairingRequestReceived {
        session_id: String,
        peer_id: String,
        request: PairingRequest,
    },
    PairingPinReady {
        session_id: String,
        pin: String,
        peer_device_name: String, // Responder's device name (for initiator to display)
        peer_public_key: Vec<u8>, // Responder's X25519 public key for ECDH
    },
    PairingResponseReceived {
        session_id: String,
        peer_id: String,
        response: PairingResponse,
    },
    PairingConfirmReceived {
        session_id: String,
        peer_id: String,
        confirm: PairingConfirm,
    },
    PairingComplete {
        session_id: String,
        peer_id: String,
        /// Peer device name (the other device's name, not this device's name)
        peer_device_name: String,
        shared_secret: Vec<u8>,
    },
    PairingFailed {
        session_id: String,
        error: String,
    },

    // Clipboard events
    ClipboardReceived(ClipboardMessage),
    ClipboardSent {
        id: String,
        peer_count: usize,
    },

    // Status events
    StatusChanged(NetworkStatus),
    #[allow(dead_code)]
    Error(String),
}
