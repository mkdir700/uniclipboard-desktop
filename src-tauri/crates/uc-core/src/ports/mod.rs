//! Port interfaces for the application layer
//!
//! Ports define the contract between the application logic (use cases)
//! and infrastructure implementations. This follows Hexagonal Architecture
//! principles, allowing the core business logic to remain independent of
//! external dependencies.

pub mod blob;
pub mod clipboard;
pub mod clipboard_repository;
pub mod device_repository;
pub mod errors;
pub mod network;
pub mod storage;

pub use blob::meta::BlobMeta;
pub use blob::port::BlobStorePort;
pub use clipboard::ClipboardPort;
pub use device_repository::DeviceRepositoryPort;
pub use errors::DeviceRepositoryError;
pub use network::NetworkPort;
pub use storage::StoragePort;
