//! Network protocol types.

pub mod events;
pub mod protocol;

pub use events::{ConnectedPeer, DiscoveredPeer, NetworkEvent, NetworkStatus};
pub use protocol::{
    ClipboardMessage, DeviceAnnounceMessage, HeartbeatMessage, PairingChallenge, PairingConfirm,
    PairingMessage, PairingRequest, PairingResponse, ProtocolMessage,
};
