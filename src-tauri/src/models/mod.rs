//! Pure data models - No infrastructure dependencies
//!
//! This module contains domain models that are free from infrastructure concerns.
//! These models should NOT depend on:
//! - Database models (DbDevice, DbClipboardRecord, etc.)
//! - Infrastructure services (storage, network, clipboard)
//! - Application services
//!
//! These models CAN depend on:
//! - Standard library types
//! - Serde for serialization
//! - Chrono for timestamps
//! - External pure data crates

pub mod clipboard;
pub mod device;
pub mod network;
pub mod p2p;

// Re-export commonly used types
pub use clipboard::{ClipboardItem, ClipboardMetadata, ClipboardStats, CodeItem, FileItem, ImageItem, LinkItem, TextItem};
pub use device::{Device, DeviceStatus, Platform};
pub use network::{ConnectionRequestDecision, ConnectionRequestMessage, ConnectionResponseMessage, ManualConnectionRequest, ManualConnectionResponse, NetworkInterface};
pub use p2p::{ConnectedPeer, DiscoveredPeer, PairedPeer};
