//! Local device identity implementation.
//!
//! This module provides a filesystem-based persistence layer for the device identity.
//! The device ID is stored as a plain text UUID in the application data directory.
//!
//! ## Architecture Notes
//!
//! - **No Repository pattern needed**: DeviceId is a singleton, not a collection
//! - **Port in core, implementation in infra**: `DeviceIdentityPort` is defined in uc-core
//! - **Fail-fast on init**: If we can't load/create the ID, the app should not start
//! - **Immutable once created**: DeviceId never changes for the lifetime of the installation

mod storage;

use anyhow::Result;
use std::path::PathBuf;
use uc_core::device::DeviceId;
use uc_core::ports::DeviceIdentityPort;

/// Local filesystem-backed device identity.
///
/// This struct implements `DeviceIdentityPort` by storing the device ID
/// as a plain text file in the application data directory.
pub struct LocalDeviceIdentity {
    device_id: DeviceId,
}

impl LocalDeviceIdentity {
    /// Load existing device ID or create a new one.
    ///
    /// This is the primary entry point for obtaining the device identity.
    /// It will:
    /// 1. Try to load from disk
    /// 2. If not found, generate a new UUID v4 and persist it
    /// 3. Fail-fast on any I/O error (app should not start without valid identity)
    pub fn load_or_create(config_dir: PathBuf) -> Result<Self> {
        if let Some(id) = storage::load_from_disk(&config_dir)? {
            Ok(Self { device_id: id })
        } else {
            let id = DeviceId::new(uuid::Uuid::new_v4().to_string());
            storage::save_to_disk(&config_dir, &id)?;
            Ok(Self { device_id: id })
        }
    }
}

impl DeviceIdentityPort for LocalDeviceIdentity {
    fn current_device_id(&self) -> DeviceId {
        self.device_id.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("uc-device-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn load_or_create_creates_new_id_when_missing() {
        let dir = make_temp_dir();
        let identity = LocalDeviceIdentity::load_or_create(dir.clone())
            .expect("load_or_create should succeed");

        // Verify it's a valid UUID
        uuid::Uuid::parse_str(identity.device_id.as_str()).expect("device_id should be valid UUID");

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn load_or_create_loads_existing_id() {
        let dir = make_temp_dir();

        // Create first identity
        let identity1 = LocalDeviceIdentity::load_or_create(dir.clone())
            .expect("first load_or_create should succeed");
        let id1 = identity1.device_id.as_str().to_string();

        // Load again - should get same ID
        let identity2 = LocalDeviceIdentity::load_or_create(dir.clone())
            .expect("second load_or_create should succeed");
        let id2 = identity2.device_id.as_str();

        assert_eq!(id1, id2, "device_id should be the same after reload");

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn port_returns_cloned_device_id() {
        let dir = make_temp_dir();
        let identity = LocalDeviceIdentity::load_or_create(dir.clone())
            .expect("load_or_create should succeed");

        let id1 = identity.current_device_id();
        let id2 = identity.current_device_id();

        assert_eq!(
            id1, id2,
            "current_device_id should return consistent values"
        );

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }
}
