//! Storage port - abstracts configuration and persistence
//!
//! This port defines the interface for storage operations including
//! configuration management, device registration, and paired device storage.

use async_trait::async_trait;
use anyhow::Result;
use crate::device::Device;
use crate::pairing::PairedPeer;
use crate::config::AppConfig;

/// Storage port - abstracts configuration and persistence
///
/// This trait provides a clean abstraction over storage operations,
/// allowing use cases to manage configuration and device data without
/// depending on concrete storage implementations (SQLite, file system, etc.).
#[async_trait]
pub trait StoragePort: Send + Sync {
    // === Configuration ===

    /// Get application configuration
    async fn get_config(&self) -> Result<AppConfig>;

    /// Save application configuration
    async fn save_config(&self, config: &AppConfig) -> Result<()>;

    // === Device management ===

    /// Get current device
    ///
    /// Returns the device record for this application instance,
    /// or None if no device has been registered yet.
    async fn get_current_device(&self) -> Result<Option<Device>>;

    /// Register/update current device
    ///
    /// Creates or updates the device record for this application instance.
    async fn register_device(&self, device: &Device) -> Result<()>;

    // === Paired devices ===

    /// Get all paired devices
    ///
    /// Returns a list of all devices that have been successfully paired
    /// with this device.
    async fn get_paired_devices(&self) -> Result<Vec<PairedPeer>>;

    /// Save paired device
    ///
    /// Creates or updates a paired device record after successful pairing.
    async fn save_paired_device(&self, peer: &PairedPeer) -> Result<()>;

    /// Remove paired device
    ///
    /// Removes a device from the paired devices list, effectively unpairing it.
    async fn remove_paired_device(&self, peer_id: &str) -> Result<()>;

    // === Encryption key ===

    /// Get encryption key material
    ///
    /// Returns the master encryption key, or None if encryption has not been set up.
    async fn get_encryption_key(&self) -> Result<Option<Vec<u8>>>;

    /// Save encryption key material
    ///
    /// Stores the master encryption key for clipboard content encryption.
    async fn save_encryption_key(&self, key: &[u8]) -> Result<()>;
}
