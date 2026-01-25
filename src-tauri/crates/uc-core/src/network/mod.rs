//! Network protocol types.

pub mod events;
pub mod paired_device;
pub mod protocol;

pub use events::{ConnectedPeer, DiscoveredPeer, NetworkEvent, NetworkStatus};
pub use paired_device::{PairedDevice, PairingState};
pub use protocol::{
    ClipboardMessage, DeviceAnnounceMessage, HeartbeatMessage, PairingChallenge, PairingConfirm,
    PairingMessage, PairingRequest, PairingResponse, ProtocolMessage,
};
