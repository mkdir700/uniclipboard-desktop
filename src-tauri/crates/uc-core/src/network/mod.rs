//! Network protocol types.

pub mod connection_policy;
pub mod events;
pub mod paired_device;
pub mod protocol;
pub mod protocol_ids;

pub use connection_policy::{
    AllowedProtocols, ConnectionPolicy, ProtocolKind, ResolvedConnectionPolicy,
};
pub use events::{
    ConnectedPeer, DiscoveredPeer, NetworkEvent, NetworkStatus, ProtocolDenyReason,
    ProtocolDirection,
};
pub use paired_device::{PairedDevice, PairingState};
pub use protocol::{
    ClipboardMessage, DeviceAnnounceMessage, HeartbeatMessage, PairingChallenge, PairingConfirm,
    PairingMessage, PairingRequest, PairingResponse, ProtocolMessage,
};
pub use protocol_ids::ProtocolId;
