//! Port interfaces for the application layer
//!
//! Ports define the contract between the application logic (use cases)
//! and infrastructure implementations. This follows Hexagonal Architecture
//! principles, allowing the core business logic to remain independent of
//! external dependencies.
//!
//! ## Port Placement Guidelines
//!
//! Before adding a new port to `uc-core/ports`, ask yourself three questions:
//!
//! 1. **Does this port represent a business capability?**
//! 2. **Will it be depended upon by multiple use cases or domains?**
//! 3. **Is it implemented by the infrastructure or platform layer?**
//!
//! If all three answers are **yes**, place it in `uc-core/ports`.
//! Otherwise, place it in the relevant `domain` submodule.

pub mod app_dirs;
pub mod app_runtime;
pub mod autostart;
pub mod blob_repository;
mod blob_store;
mod blob_writer;
pub mod clipboard;
mod clipboard_change_handler;
mod clipboard_event;
mod clock;
pub mod connection_policy;
pub mod device_identity;
pub mod device_repository;
pub mod errors;
mod hash;
pub mod identity_store;
pub mod network;
pub mod network_control;
pub mod observability;
pub mod onboarding;
pub mod paired_device_repository;
pub mod security;
pub mod settings;
mod timer;
pub mod ui_port;
pub mod watcher_control;

pub use blob_repository::BlobRepositoryPort;
pub use blob_store::BlobStorePort;
pub use blob_writer::BlobWriterPort;
pub use clipboard_event::*;
pub use clock::*;
pub use connection_policy::{ConnectionPolicyResolverError, ConnectionPolicyResolverPort};
pub use hash::*;
pub use timer::TimerPort;

pub use app_dirs::AppDirsPort;
pub use app_runtime::AppRuntimePort;
pub use autostart::AutostartPort;
pub use clipboard::*;
pub use clipboard_change_handler::ClipboardChangeHandler;
pub use device_identity::DeviceIdentityPort;
pub use device_repository::DeviceRepositoryPort;
pub use errors::{AppDirsError, DeviceRepositoryError, PairedDeviceRepositoryError};
pub use identity_store::{IdentityStoreError, IdentityStorePort};
pub use network::NetworkPort;
pub use network_control::NetworkControlPort;
pub use observability::{extract_trace, OptionalTrace, TraceMetadata, TraceParseError};
pub use onboarding::OnboardingStatePort;
pub use paired_device_repository::PairedDeviceRepositoryPort;
pub use security::encryption::EncryptionPort;
pub use security::encryption_session::EncryptionSessionPort;
pub use security::key_material::KeyMaterialPort;
pub use security::secure_storage::{SecureStorageError, SecureStoragePort};
pub use settings::{SettingsMigrationPort, SettingsPort};
pub use ui_port::UiPort;
pub use watcher_control::{WatcherControlError, WatcherControlPort};
