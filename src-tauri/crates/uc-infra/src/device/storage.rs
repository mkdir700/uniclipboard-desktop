//! Private storage implementation for device identity.
//!
//! This module handles the low-level file I/O for persisting the device ID.
//! It is not part of the public API of the device module.

use anyhow::{Context, Result};
use std::path::PathBuf;
use uc_core::device::DeviceId;

const DEVICE_ID_FILE: &str = "device_id.txt";

/// Load device ID from disk, returning None if file doesn't exist.
///
/// This is a private implementation detail of the device module.
pub(crate) fn load_from_disk(config_dir: &PathBuf) -> Result<Option<DeviceId>> {
    let path = config_dir.join(DEVICE_ID_FILE);

    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("read device_id file failed: {}", path.display()))?;

    let id_str = content.trim();
    if id_str.is_empty() {
        return Ok(None);
    }

    // Validate UUID format
    uuid::Uuid::parse_str(id_str)
        .with_context(|| format!("invalid device_id UUID in file: {}", path.display()))?;

    Ok(Some(DeviceId::new(id_str.to_string())))
}

/// Save device ID to disk, creating parent directory if needed.
///
/// This is a private implementation detail of the device module.
pub(crate) fn save_to_disk(config_dir: &PathBuf, id: &DeviceId) -> Result<()> {
    // Ensure parent directory exists
    std::fs::create_dir_all(config_dir)
        .with_context(|| format!("create config dir failed: {}", config_dir.display()))?;

    let path = config_dir.join(DEVICE_ID_FILE);

    // Try atomic write using temp file + rename first
    // If rename fails (e.g., cross-device link in CI environments), fall back to direct write
    let tmp_path = path.with_extension("txt.tmp");
    std::fs::write(&tmp_path, id.as_str())
        .with_context(|| format!("write temp device_id failed: {}", tmp_path.display()))?;

    match std::fs::rename(&tmp_path, &path) {
        Ok(_) => Ok(()),
        Err(rename_err) => {
            // Rename failed - likely cross-device link or permission issue
            // Fall back to direct write (non-atomic but better than failing)
            std::fs::write(&path, id.as_str()).with_context(|| {
                format!(
                    "direct write device_id failed after rename error ({}): {}",
                    rename_err,
                    path.display()
                )
            })?;
            // Clean up temp file if it still exists
            let _ = std::fs::remove_file(&tmp_path);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_temp_dir() -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("uc-device-storage-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn load_from_nonexistent_dir_returns_none() {
        let dir = std::env::temp_dir().join("nonexistent-uc-device-test");
        let result = load_from_disk(&dir).expect("load_from_disk should succeed");
        assert!(
            result.is_none(),
            "should return None for nonexistent directory"
        );
    }

    #[test]
    fn load_from_missing_file_returns_none() {
        let dir = make_temp_dir();
        let result = load_from_disk(&dir).expect("load_from_disk should succeed");
        assert!(
            result.is_none(),
            "should return None when file doesn't exist"
        );
        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn save_then_load_round_trip() {
        let dir = make_temp_dir();
        let id = DeviceId::new(uuid::Uuid::new_v4().to_string());

        save_to_disk(&dir, &id).expect("save should succeed");
        let loaded = load_from_disk(&dir).expect("load should succeed");

        assert!(loaded.is_some(), "should load saved device_id");
        assert_eq!(
            loaded.unwrap().as_str(),
            id.as_str(),
            "loaded ID should match saved ID"
        );

        let path = dir.join(DEVICE_ID_FILE);
        assert!(path.exists(), "device_id file should exist");

        // Cleanup
        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn load_validates_uuid_format() {
        let dir = make_temp_dir();
        let path = dir.join(DEVICE_ID_FILE);

        // Write invalid UUID
        std::fs::write(&path, "not-a-uuid").expect("write invalid UUID");

        let result = load_from_disk(&dir);
        assert!(result.is_err(), "should fail on invalid UUID format");

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn save_creates_parent_dir() {
        let dir = std::env::temp_dir()
            .join("uc-device-create-dir-test")
            .join("nested");
        let id = DeviceId::new(uuid::Uuid::new_v4().to_string());

        save_to_disk(&dir, &id).expect("save should create parent dirs");

        let loaded = load_from_disk(&dir).expect("load should succeed");
        assert!(loaded.is_some(), "should load after creating parent dirs");

        std::fs::remove_dir_all(std::env::temp_dir().join("uc-device-create-dir-test"))
            .expect("cleanup temp dir");
    }

    #[test]
    fn atomic_write_prevents_corruption() {
        let dir = make_temp_dir();
        let id1 = DeviceId::new(uuid::Uuid::new_v4().to_string());
        let id2 = DeviceId::new(uuid::Uuid::new_v4().to_string());

        save_to_disk(&dir, &id1).expect("first save should succeed");
        save_to_disk(&dir, &id2).expect("second save should succeed");

        let loaded = load_from_disk(&dir).expect("load should succeed");
        assert_eq!(
            loaded.unwrap().as_str(),
            id2.as_str(),
            "should have second ID"
        );

        let tmp_path = dir.join("device_id.txt.tmp");
        assert!(!tmp_path.exists(), "temp file should be cleaned up");

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }
}
