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
mod blob_materializer;
pub mod blob_repository;
mod blob_store;
mod blob_writer;
pub mod clipboard;
mod clipboard_change_handler;
mod clipboard_event;
mod clock;
pub mod device_identity;
pub mod device_repository;
pub mod errors;
mod hash;
pub mod network;
pub mod onboarding;
pub mod security;
pub mod settings;
pub mod ui_port;
pub mod watcher_control;

pub use blob_materializer::BlobMaterializerPort;
pub use blob_repository::BlobRepositoryPort;
pub use blob_store::BlobStorePort;
pub use blob_writer::BlobWriterPort;
pub use clipboard_event::*;
pub use clock::*;
pub use hash::*;

pub use app_dirs::AppDirsPort;
pub use app_runtime::AppRuntimePort;
pub use autostart::AutostartPort;
pub use clipboard::*;
pub use clipboard_change_handler::ClipboardChangeHandler;
pub use device_identity::DeviceIdentityPort;
pub use device_repository::DeviceRepositoryPort;
pub use errors::{AppDirsError, DeviceRepositoryError};
pub use network::NetworkPort;
pub use onboarding::OnboardingStatePort;
pub use security::encryption::EncryptionPort;
pub use security::encryption_session::EncryptionSessionPort;
pub use security::key_material::KeyMaterialPort;
pub use security::keyring::KeyringPort;
pub use settings::{SettingsMigrationPort, SettingsPort};
pub use ui_port::UiPort;
pub use watcher_control::{WatcherControlError, WatcherControlPort};
