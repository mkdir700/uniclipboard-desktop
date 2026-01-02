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
///
/// Note: This no longer includes public_key as we've removed ECDH key exchange.
/// All devices now use the same master key derived from the user's encryption password.
///
/// # Fields
/// - `device_id`: 6-digit stable device ID (from database devices.id)
/// - `peer_id`: libp2p PeerId (network layer, changes on each restart)
#[derive(Clone, Serialize, Deserialize)]
pub struct PairingRequest {
    pub session_id: String,
    pub device_name: String,
    /// 6-digit stable device ID (from devices table)
    pub device_id: String,
    /// Current libp2p PeerId for this session (network layer)
    pub peer_id: String,
}

/// Pairing challenge sent by responder with PIN
///
/// Note: This no longer includes public_key as we've removed ECDH key exchange.
/// All devices now use the same master key derived from the user's encryption password.
///
/// # Fields
/// - `device_id`: 6-digit stable device ID of the responder (from database devices.id)
#[derive(Clone, Serialize, Deserialize)]
pub struct PairingChallenge {
    pub session_id: String,
    pub pin: String,
    pub device_name: String, // Responder's device name
    /// 6-digit stable device ID of the responder (from devices table)
    pub device_id: String,
}

/// Pairing response from initiator after PIN verification
///
/// # Security Requirements for `pin_hash`
///
/// The `pin_hash` field MUST contain a properly derived key using a secure
/// password hashing algorithm, NOT a simple cryptographic hash.
///
/// ## Algorithm: Argon2id
///
/// Use Argon2id (the hybrid version that provides protection against both
/// GPU/ASIC and side-channel attacks) with the following RECOMMENDED parameters:
///
/// - **Output length**: 32 bytes (256 bits)
/// - **Salt**: 16 bytes, cryptographically random per PIN
/// - **Memory cost**: 64 MiB (65536 KiB)
/// - **Time cost**: 3 iterations
/// - **Parallelism**: 4 lanes
///
/// ## Salt Storage Strategy
///
/// The salt MUST be encoded together with the hash to allow verification.
/// Use the following encoding format:
///
/// ```text
/// {version||salt||hash}
/// ```
///
/// Where:
/// - `version`: 1 byte (currently 0x01 for Argon2id)
/// - `salt`: 16 bytes
/// - `hash`: 32 bytes (Argon2id output)
///
/// Total: 49 bytes (1 + 16 + 32)
///
/// This allows the verifier to extract the salt and recompute the hash
/// for verification while supporting future algorithm upgrades via versioning.
///
/// ## Example Encoding (pseudocode)
///
/// ```rust,ignore
/// let salt = random_16_bytes();
/// let hash = argon2id_hash(pin, salt, params);
/// let encoded = [0x01, salt..., hash...]; // 49 bytes total
/// ```
#[derive(Clone, Serialize, Deserialize)]
pub struct PairingResponse {
    pub session_id: String,
    /// Argon2id-derived key encoded as {version(1)||salt(16)||hash(32)} = 49 bytes
    pub pin_hash: Vec<u8>,
    pub accepted: bool,
}

/// Final pairing confirmation message
///
/// Note: This no longer includes shared_secret as we've removed ECDH key exchange.
/// All devices now use the same master key derived from the user's encryption password.
#[derive(Clone, Serialize, Deserialize)]
pub struct PairingConfirm {
    pub session_id: String,
    pub success: bool,
    pub error: Option<String>,
    /// Sender's device name (the device sending this confirm message)
    pub sender_device_name: String,
    /// Sender's 6-digit device ID (stable identifier from database)
    pub device_id: String,
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
            .field("peer_id", &self.peer_id)
            .finish()
    }
}

impl std::fmt::Debug for PairingChallenge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PairingChallenge")
            .field("session_id", &self.session_id)
            .field("pin", &"[REDACTED]")
            .field("device_name", &self.device_name)
            .field("device_id", &self.device_id)
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
            .field("error", &self.error)
            .field("sender_device_name", &self.sender_device_name)
            .field("device_id", &self.device_id)
            .finish()
    }
}
