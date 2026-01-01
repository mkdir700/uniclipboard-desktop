use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// P2P protocol messages for UniClipboard
/// Based on decentpaste protocol with UniClipboard-specific adaptations
#[derive(Clone, Serialize, Deserialize)]
pub enum ProtocolMessage {
    Pairing(PairingMessage),
    Clipboard(ClipboardMessage),
    Heartbeat(HeartbeatMessage),
    /// Announces device name to all peers on the network.
    /// Used when device name is changed in settings.
    DeviceAnnounce(DeviceAnnounceMessage),
}

/// Pairing protocol messages for secure device pairing with PIN verification
#[derive(Clone, Serialize, Deserialize)]
pub enum PairingMessage {
    Request(PairingRequest),
    Challenge(PairingChallenge),
    Response(PairingResponse),
    Confirm(PairingConfirm),
}

/// Initial pairing request sent by initiator
#[derive(Clone, Serialize, Deserialize)]
pub struct PairingRequest {
    pub session_id: String,
    pub device_name: String,
    pub device_id: String,
    pub public_key: Vec<u8>, // X25519 public key for ECDH
}

/// Pairing challenge sent by responder with PIN
#[derive(Clone, Serialize, Deserialize)]
pub struct PairingChallenge {
    pub session_id: String,
    pub pin: String,
    pub device_name: String, // Responder's device name
    pub public_key: Vec<u8>, // Responder's X25519 public key for ECDH
}

/// Pairing response from initiator after PIN verification
#[derive(Clone, Serialize, Deserialize)]
pub struct PairingResponse {
    pub session_id: String,
    pub pin_hash: Vec<u8>,
    pub accepted: bool,
}

/// Final pairing confirmation message
#[derive(Clone, Serialize, Deserialize)]
pub struct PairingConfirm {
    pub session_id: String,
    pub success: bool,
    pub shared_secret: Option<Vec<u8>>, // Encrypted shared secret
    pub error: Option<String>,
    /// Sender's device name (the device sending this confirm message)
    pub sender_device_name: String,
}

/// Clipboard content broadcast via GossipSub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardMessage {
    pub id: String,
    pub content_hash: String,
    pub encrypted_content: Vec<u8>,
    pub timestamp: DateTime<Utc>,
    pub origin_device_id: String,
    pub origin_device_name: String,
}

/// Heartbeat message for connection liveness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatMessage {
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
}

/// Device name announcement broadcast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceAnnounceMessage {
    pub peer_id: String,
    pub device_name: String,
    pub timestamp: DateTime<Utc>,
}

impl ProtocolMessage {
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

// Custom Debug implementations to redact sensitive fields

impl std::fmt::Debug for ProtocolMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pairing(msg) => f.debug_tuple("Pairing").field(msg).finish(),
            Self::Clipboard(msg) => f.debug_tuple("Clipboard").field(msg).finish(),
            Self::Heartbeat(msg) => f.debug_tuple("Heartbeat").field(msg).finish(),
            Self::DeviceAnnounce(msg) => f.debug_tuple("DeviceAnnounce").field(msg).finish(),
        }
    }
}

impl std::fmt::Debug for PairingMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request(msg) => f.debug_tuple("Request").field(msg).finish(),
            Self::Challenge(msg) => f.debug_tuple("Challenge").field(msg).finish(),
            Self::Response(msg) => f.debug_tuple("Response").field(msg).finish(),
            Self::Confirm(msg) => f.debug_tuple("Confirm").field(msg).finish(),
        }
    }
}

impl std::fmt::Debug for PairingRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PairingRequest")
            .field("session_id", &self.session_id)
            .field("device_name", &self.device_name)
            .field("device_id", &self.device_id)
            .field("public_key", &"[REDACTED]")
            .finish()
    }
}

impl std::fmt::Debug for PairingChallenge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PairingChallenge")
            .field("session_id", &self.session_id)
            .field("pin", &"[REDACTED]")
            .field("device_name", &self.device_name)
            .field("public_key", &"[REDACTED]")
            .finish()
    }
}

impl std::fmt::Debug for PairingResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PairingResponse")
            .field("session_id", &self.session_id)
            .field("pin_hash", &"[REDACTED]")
            .field("accepted", &self.accepted)
            .finish()
    }
}

impl std::fmt::Debug for PairingConfirm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PairingConfirm")
            .field("session_id", &self.session_id)
            .field("success", &self.success)
            .field("shared_secret", &"[REDACTED]")
            .field("error", &self.error)
            .field("sender_device_name", &self.sender_device_name)
            .finish()
    }
}
