//! # uc-core
//!
//! Core domain models and business logic for UniClipboard.
//!
//! This crate contains pure business logic without any infrastructure dependencies.

// Public module exports
pub mod clipboard;
pub mod config;
pub mod decision;
pub mod device;
pub mod ids;
pub mod network;
pub mod pairing;
pub mod ports;
pub mod sync;

// Re-export commonly used types at the crate root
pub use config::AppConfig;
pub use device::{Device, DeviceStatus, Platform, DeviceId, DeviceName};
pub use ids::{PeerId, SessionId};
pub use network::{NetworkEvent, NetworkStatus, ProtocolMessage};
pub use pairing::{PairedPeer, PairingState};
pub use sync::SyncState;
