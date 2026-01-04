//! Network protocol types.

pub mod events;
pub mod protocol;

pub use events::{NetworkEvent, NetworkStatus, DiscoveredPeer, ConnectedPeer};
pub use protocol::{ProtocolMessage, PairingMessage, ClipboardMessage, HeartbeatMessage, DeviceAnnounceMessage};
