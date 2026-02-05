use serde::{Deserialize, Serialize};

use super::{ClipboardMessage, DeviceAnnounceMessage, HeartbeatMessage, PairingMessage};

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
