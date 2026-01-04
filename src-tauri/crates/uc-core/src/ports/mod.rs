//! Port interfaces for the application layer
//!
//! Ports define the contract between the application logic (use cases)
//! and infrastructure implementations. This follows Hexagonal Architecture
//! principles, allowing the core business logic to remain independent of
//! external dependencies.

pub mod clipboard;
pub mod network;
pub mod storage;

pub use clipboard::ClipboardPort;
pub use network::NetworkPort;
pub use storage::StoragePort;
