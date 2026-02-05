use serde::{Deserialize, Serialize};

use crate::security::model::KeySlotFile;

/// Pairing protocol messages for secure device pairing with PIN verification
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PairingMessage {
    Request(PairingRequest),
    Challenge(PairingChallenge),
    KeyslotOffer(PairingKeyslotOffer),
    ChallengeResponse(PairingChallengeResponse),
    Response(PairingResponse),
    Confirm(PairingConfirm),
    Reject(PairingReject),
    Cancel(PairingCancel),
    Busy(PairingBusy),
}

/// Initial pairing request sent by initiator
///
/// Note: This no longer includes public_key as we've removed ECDH key exchange.
/// All devices now use the same master key derived from the user's encryption password.
///
/// # Fields
/// - `device_id`: 6-digit stable device ID (from database devices.id)
/// - `peer_id`: libp2p PeerId (network layer, stable while identity is persisted)
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairingRequest {
    pub session_id: String,
    pub device_name: String,
    /// 6-digit stable device ID (from devices table)
    pub device_id: String,
    /// Target PeerId for validation. Responder checks this matches its own PeerId.
    /// Sender PeerId is passed via network layer events (e.g., PairingEvent::RecvRequest).
    pub peer_id: String,
    /// Stable identity public key (Ed25519)
    pub identity_pubkey: Vec<u8>,
    /// Random nonce for short-code transcript
    pub nonce: Vec<u8>,
}

/// Pairing challenge sent by responder with PIN
///
/// Note: This no longer includes public_key as we've removed ECDH key exchange.
/// All devices now use the same master key derived from the user's encryption password.
///
/// # Fields
/// - `device_id`: 6-digit stable device ID of the responder (from database devices.id)
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairingChallenge {
    pub session_id: String,
    pub pin: String,
    pub device_name: String, // Responder's device name
    /// 6-digit stable device ID of the responder (from devices table)
    pub device_id: String,
    /// Stable identity public key (Ed25519)
    pub identity_pubkey: Vec<u8>,
    /// Random nonce for short-code transcript
    pub nonce: Vec<u8>,
}

/// Keyslot offer sent by responder for join flow
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairingKeyslotOffer {
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keyslot_file: Option<KeySlotFile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub challenge: Option<Vec<u8>>,
}

/// Challenge response sent by initiator for join flow
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairingChallengeResponse {
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encrypted_challenge: Option<Vec<u8>>,
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
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairingConfirm {
    pub session_id: String,
    pub success: bool,
    pub error: Option<String>,
    /// Sender's device name (the device sending this confirm message)
    pub sender_device_name: String,
    /// Sender's 6-digit device ID (stable identifier from database)
    pub device_id: String,
}

/// Pairing rejection message
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairingReject {
    pub session_id: String,
    pub reason: Option<String>,
}

/// Pairing cancel message
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairingCancel {
    pub session_id: String,
    pub reason: Option<String>,
}

/// Pairing busy message
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairingBusy {
    pub session_id: String,
    pub reason: Option<String>,
}

impl PairingMessage {
    pub fn session_id(&self) -> &str {
        match self {
            PairingMessage::Request(msg) => &msg.session_id,
            PairingMessage::Challenge(msg) => &msg.session_id,
            PairingMessage::KeyslotOffer(msg) => &msg.session_id,
            PairingMessage::ChallengeResponse(msg) => &msg.session_id,
            PairingMessage::Response(msg) => &msg.session_id,
            PairingMessage::Confirm(msg) => &msg.session_id,
            PairingMessage::Reject(msg) => &msg.session_id,
            PairingMessage::Cancel(msg) => &msg.session_id,
            PairingMessage::Busy(msg) => &msg.session_id,
        }
    }
}

// Custom Debug implementations to redact sensitive fields

impl std::fmt::Debug for PairingMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request(msg) => f.debug_tuple("Request").field(msg).finish(),
            Self::Challenge(msg) => f.debug_tuple("Challenge").field(msg).finish(),
            Self::KeyslotOffer(msg) => f.debug_tuple("KeyslotOffer").field(msg).finish(),
            Self::ChallengeResponse(msg) => f.debug_tuple("ChallengeResponse").field(msg).finish(),
            Self::Response(msg) => f.debug_tuple("Response").field(msg).finish(),
            Self::Confirm(msg) => f.debug_tuple("Confirm").field(msg).finish(),
            Self::Reject(msg) => f.debug_tuple("Reject").field(msg).finish(),
            Self::Cancel(msg) => f.debug_tuple("Cancel").field(msg).finish(),
            Self::Busy(msg) => f.debug_tuple("Busy").field(msg).finish(),
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
            .field("identity_pubkey_len", &self.identity_pubkey.len())
            .field("nonce_len", &self.nonce.len())
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
            .field("identity_pubkey_len", &self.identity_pubkey.len())
            .field("nonce_len", &self.nonce.len())
            .finish()
    }
}

impl std::fmt::Debug for PairingKeyslotOffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let keyslot_present = self.keyslot_file.is_some();
        let challenge_len = self.challenge.as_ref().map(Vec::len);

        f.debug_struct("PairingKeyslotOffer")
            .field("session_id", &self.session_id)
            .field("keyslot_file_present", &keyslot_present)
            .field("challenge_len", &challenge_len)
            .finish()
    }
}

impl std::fmt::Debug for PairingChallengeResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let encrypted_len = self.encrypted_challenge.as_ref().map(Vec::len);

        f.debug_struct("PairingChallengeResponse")
            .field("session_id", &self.session_id)
            .field("encrypted_challenge_len", &encrypted_len)
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

impl std::fmt::Debug for PairingReject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PairingReject")
            .field("session_id", &self.session_id)
            .field("reason", &self.reason)
            .finish()
    }
}

impl std::fmt::Debug for PairingCancel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PairingCancel")
            .field("session_id", &self.session_id)
            .field("reason", &self.reason)
            .finish()
    }
}

impl std::fmt::Debug for PairingBusy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PairingBusy")
            .field("session_id", &self.session_id)
            .field("reason", &self.reason)
            .finish()
    }
}
