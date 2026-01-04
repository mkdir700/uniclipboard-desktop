//! Storage port adapter - bridges storage infrastructure to StoragePort

use async_trait::async_trait;
use uc_core::config::AppConfig;
use uc_core::device::Device;
use uc_core::pairing::PairedPeer;
use uc_core::ports::StoragePort;

/// Adapter that wraps storage implementations to provide StoragePort
///
/// TODO: This is a placeholder implementation. In later phases, we will implement
/// the actual adapter that wraps the existing storage (Setting, DeviceManager, PeerStorage).
pub struct FileStorageAdapter {
    _private: (),
}

impl FileStorageAdapter {
    /// Create a new FileStorageAdapter
    ///
    /// TODO: This will accept the actual storage implementations in later phases
    pub fn new() -> Self {
        Self { _private: () }
    }
}

#[async_trait]
impl StoragePort for FileStorageAdapter {
    // === Configuration ===

    async fn get_config(&self) -> anyhow::Result<AppConfig> {
        // TODO: Implement - read from Setting
        Ok(AppConfig::default())
    }

    async fn save_config(&self, _config: &AppConfig) -> anyhow::Result<()> {
        // TODO: Implement - save to Setting
        Ok(())
    }

    // === Device management ===

    async fn get_current_device(&self) -> anyhow::Result<Option<Device>> {
        // TODO: Implement - read from DeviceManager
        Ok(None)
    }

    async fn register_device(&self, _device: &Device) -> anyhow::Result<()> {
        // TODO: Implement - save to DeviceManager
        Ok(())
    }

    // === Paired devices ===

    async fn get_paired_devices(&self) -> anyhow::Result<Vec<PairedPeer>> {
        // TODO: Implement - read from PeerStorage
        Ok(Vec::new())
    }

    async fn save_paired_device(&self, _peer: &PairedPeer) -> anyhow::Result<()> {
        // TODO: Implement - save to PeerStorage
        Ok(())
    }

    async fn remove_paired_device(&self, _peer_id: &str) -> anyhow::Result<()> {
        // TODO: Implement - remove from PeerStorage
        Ok(())
    }

    // === Encryption key ===

    async fn get_encryption_key(&self) -> anyhow::Result<Option<Vec<u8>>> {
        // TODO: Implement - read from password manager
        Ok(None)
    }

    async fn save_encryption_key(&self, _key: &[u8]) -> anyhow::Result<()> {
        // TODO: Implement - save to password manager
        Ok(())
    }
}
