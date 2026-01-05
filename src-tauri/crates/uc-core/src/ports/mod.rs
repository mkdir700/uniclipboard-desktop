//! Port interfaces for the application layer
//!
//! Ports define the contract between the application logic (use cases)
//! and infrastructure implementations. This follows Hexagonal Architecture
//! principles, allowing the core business logic to remain independent of
//! external dependencies.

pub mod clipboard;
pub mod device_repository;
pub mod errors;
pub mod network;
pub mod storage;
pub mod clipboard_repository;

pub use clipboard::ClipboardPort;
pub use errors::DeviceRepositoryError;
pub use device_repository::DeviceRepositoryPort;
pub use network::NetworkPort;
pub use storage::StoragePort;
