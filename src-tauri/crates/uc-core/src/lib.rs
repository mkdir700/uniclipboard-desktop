//! # uc-core
//!
//! Core domain models and business logic for UniClipboard.
//!
//! This crate contains pure business logic without any infrastructure dependencies.

// Public module exports
pub mod clipboard;
pub mod decision;
pub mod device;
pub mod ids;
pub mod network;
pub mod pairing;
pub mod ports;
pub mod security;
pub mod settings;
pub mod sync;
pub mod system;

// Re-export commonly used types at the crate root
// pub use system::;
pub use device::{Device, DeviceId, DeviceName, DeviceStatus, Platform};
pub use ids::{PeerId, SessionId};
pub use network::{NetworkEvent, NetworkStatus, ProtocolMessage};
pub use pairing::{PairedPeer, PairingState};
pub use sync::SyncState;
