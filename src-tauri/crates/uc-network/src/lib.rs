//! # uc-network
//!
//! P2P networking layer for UniClipboard using libp2p.
//!
//! This crate provides a clean abstraction over libp2p for:
//! - Device discovery via mDNS
//! - Secure device pairing with PIN verification
//! - Clipboard content transfer via BlobStream
//! - Connection management and health monitoring

pub mod behaviour;
pub mod blob;
pub mod codec;
pub mod pairing;
pub mod pin_hash;
pub mod swarm;
pub mod transport;

// Re-export commonly used types from uc-core
pub use uc_core::network::{
    ClipboardMessage, DiscoveredPeer, ConnectedPeer, NetworkEvent, NetworkStatus,
    ProtocolMessage, PairingMessage, PairingRequest, PairingChallenge, PairingResponse,
    PairingConfirm, HeartbeatMessage, DeviceAnnounceMessage,
};

// Re-export uc-network specific types
pub use codec::{PairingRequest as ReqPairingRequest, PairingResponse as ReqPairingResponse};
pub use pairing::{PairingManager, PairingSession};
pub use pin_hash::{hash_pin, verify_pin};
pub use swarm::{NetworkCommand, NetworkManager};
