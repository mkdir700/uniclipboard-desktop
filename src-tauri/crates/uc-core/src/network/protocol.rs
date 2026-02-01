use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::security::model::KeySlotFile;

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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;

    fn sample_request(session_id: &str) -> PairingRequest {
        PairingRequest {
            session_id: session_id.to_string(),
            device_name: "Desk".to_string(),
            device_id: "123456".to_string(),
            peer_id: "peer-1".to_string(),
            identity_pubkey: vec![1, 2, 3],
            nonce: vec![4, 5, 6],
        }
    }

    fn sample_challenge(session_id: &str) -> PairingChallenge {
        PairingChallenge {
            session_id: session_id.to_string(),
            pin: "1234".to_string(),
            device_name: "Laptop".to_string(),
            device_id: "654321".to_string(),
            identity_pubkey: vec![7, 8],
            nonce: vec![9],
        }
    }

    fn sample_keyslot_file() -> crate::security::model::KeySlotFile {
        use crate::security::model::{
            EncryptedBlob, EncryptionAlgo, EncryptionFormatVersion, KdfAlgorithm, KdfParams,
            KdfParamsV1, KeyScope, KeySlotFile, KeySlotVersion,
        };

        KeySlotFile {
            version: KeySlotVersion::V1,
            scope: KeyScope {
                profile_id: "profile-1".to_string(),
            },
            kdf: KdfParams {
                alg: KdfAlgorithm::Argon2id,
                params: KdfParamsV1 {
                    mem_kib: 1024,
                    iters: 2,
                    parallelism: 1,
                },
            },
            salt: vec![1, 2, 3],
            wrapped_master_key: EncryptedBlob {
                version: EncryptionFormatVersion::V1,
                aead: EncryptionAlgo::XChaCha20Poly1305,
                nonce: vec![9, 8, 7],
                ciphertext: vec![6, 5, 4],
                aad_fingerprint: None,
            },
            created_at: None,
            updated_at: None,
        }
    }

    #[test]
    fn pairing_message_session_id_returns_inner_id() {
        let request = PairingMessage::Request(sample_request("s1"));
        let challenge = PairingMessage::Challenge(sample_challenge("s2"));
        let response = PairingMessage::Response(PairingResponse {
            session_id: "s3".to_string(),
            pin_hash: vec![1, 2, 3],
            accepted: true,
        });
        let confirm = PairingMessage::Confirm(PairingConfirm {
            session_id: "s4".to_string(),
            success: true,
            error: None,
            sender_device_name: "Phone".to_string(),
            device_id: "111111".to_string(),
        });

        assert_eq!(request.session_id(), "s1");
        assert_eq!(challenge.session_id(), "s2");
        assert_eq!(response.session_id(), "s3");
        assert_eq!(confirm.session_id(), "s4");
    }

    #[test]
    fn protocol_message_round_trip_device_announce() {
        let message = ProtocolMessage::DeviceAnnounce(DeviceAnnounceMessage {
            peer_id: "peer-9".to_string(),
            device_name: "Desk".to_string(),
            timestamp: Utc::now(),
        });

        let bytes = message.to_bytes().expect("serialize");
        let decoded = ProtocolMessage::from_bytes(&bytes).expect("deserialize");

        match decoded {
            ProtocolMessage::DeviceAnnounce(decoded_message) => {
                assert_eq!(decoded_message.peer_id, "peer-9");
                assert_eq!(decoded_message.device_name, "Desk");
            }
            _ => panic!("expected DeviceAnnounce message"),
        }
    }

    #[test]
    fn pairing_request_json_still_deserializes() {
        let legacy_json = json!({
            "Pairing": {
                "Request": {
                    "session_id": "s-legacy",
                    "device_name": "Desk",
                    "device_id": "123456",
                    "peer_id": "peer-1",
                    "identity_pubkey": [1, 2, 3],
                    "nonce": [4, 5, 6]
                }
            }
        });

        let bytes = serde_json::to_vec(&legacy_json).expect("serialize");
        let decoded = ProtocolMessage::from_bytes(&bytes).expect("deserialize");

        match decoded {
            ProtocolMessage::Pairing(PairingMessage::Request(request)) => {
                assert_eq!(request.session_id, "s-legacy");
                assert_eq!(request.device_id, "123456");
            }
            _ => panic!("expected pairing request"),
        }
    }

    #[test]
    fn pairing_message_deserializes_keyslot_offer_json() {
        let keyslot_file = sample_keyslot_file();
        let keyslot_value = serde_json::to_value(&keyslot_file).expect("serialize keyslot");
        let challenge: Vec<u8> = (0u8..32).collect();

        let json_value = json!({
            "Pairing": {
                "KeyslotOffer": {
                    "session_id": "s1",
                    "keyslot_file": keyslot_value,
                    "challenge": challenge
                }
            }
        });

        let bytes = serde_json::to_vec(&json_value).expect("serialize");
        let decoded = ProtocolMessage::from_bytes(&bytes).expect("deserialize");
        let round_trip = serde_json::to_value(&decoded).expect("serialize round trip");

        assert_eq!(round_trip, json_value);
    }

    #[test]
    fn pairing_message_deserializes_challenge_response_json() {
        let json_value = json!({
            "Pairing": {
                "ChallengeResponse": {
                    "session_id": "s2",
                    "encrypted_challenge": [9, 8, 7]
                }
            }
        });

        let bytes = serde_json::to_vec(&json_value).expect("serialize");
        let decoded = ProtocolMessage::from_bytes(&bytes).expect("deserialize");
        let round_trip = serde_json::to_value(&decoded).expect("serialize round trip");

        assert_eq!(round_trip, json_value);
    }

    #[test]
    fn debug_redacts_sensitive_fields() {
        let request = sample_request("s5");
        let request_debug = format!("{:?}", request);
        assert!(request_debug.contains("identity_pubkey_len"));
        assert!(request_debug.contains("nonce_len"));
        assert!(!request_debug.contains("identity_pubkey:"));

        let challenge = sample_challenge("s6");
        let challenge_debug = format!("{:?}", challenge);
        assert!(challenge_debug.contains("[REDACTED]"));
        assert!(!challenge_debug.contains("1234"));

        let response = PairingResponse {
            session_id: "s7".to_string(),
            pin_hash: vec![1, 2, 3],
            accepted: true,
        };
        let response_debug = format!("{:?}", response);
        assert!(response_debug.contains("[REDACTED]"));
        assert!(!response_debug.contains("1, 2, 3"));
    }

    #[test]
    fn pairing_message_session_id_handles_cancel_and_reject() {
        let reject = PairingMessage::Reject(PairingReject {
            session_id: "s1".to_string(),
            reason: Some("user".to_string()),
        });
        let cancel = PairingMessage::Cancel(PairingCancel {
            session_id: "s2".to_string(),
            reason: Some("timeout".to_string()),
        });
        let busy = PairingMessage::Busy(PairingBusy {
            session_id: "s3".to_string(),
            reason: Some("occupied".to_string()),
        });

        assert_eq!(reject.session_id(), "s1");
        assert_eq!(cancel.session_id(), "s2");
        assert_eq!(busy.session_id(), "s3");
    }
}
