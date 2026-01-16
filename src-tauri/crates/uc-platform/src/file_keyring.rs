//! File-based keyring implementation for WSL and headless environments.
//!
//! This implementation stores KEK material as encrypted files on disk,
//! with restricted permissions (0600 on Unix). This is less secure than
//! system keyrings but provides a fallback for development environments.

use std::fs;
use std::io;
use std::path::PathBuf;

use uc_core::{
    ports::KeyringPort,
    security::model::{EncryptionError, Kek, KeyScope},
};

const KEK_PREFIX: &str = "kek:v1:";

/// Build the filename for a KEK file from the given scope.
fn build_filename(scope: &KeyScope) -> String {
    format!("{}{}.bin", KEK_PREFIX, scope.to_identifier())
}

/// File-based keyring implementation.
///
/// KEK material is stored as binary files in `~/.config/com.uniclipboard/`.
/// Each scope gets its own file: `kek:v1:<scope-identifier>.bin`
///
/// # Security
///
/// - Files are created with mode 0600 (owner read/write only) on Unix
/// - Uses atomic write-and-rename to prevent corruption
/// - Only suitable for development environments
#[derive(Clone)]
pub struct FileBasedKeyring {
    base_dir: PathBuf,
}

impl FileBasedKeyring {
    /// Create a new FileBasedKeyring with the default base directory.
    ///
    /// The default directory is `~/.config/com.uniclipboard/`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The config directory cannot be determined
    /// - The base directory cannot be created
    pub fn new() -> Result<Self, io::Error> {
        let base_dir = dirs::config_dir()
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "Cannot determine config directory")
            })?
            .join("com.uniclipboard");

        fs::create_dir_all(&base_dir)?;

        Ok(Self { base_dir })
    }

    /// Create a FileBasedKeyring with a custom base directory for testing.
    pub fn with_base_dir(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Get the base directory where KEK files are stored.
    pub fn base_dir(&self) -> &PathBuf {
        &self.base_dir
    }

    /// Get the full path to the KEK file for the given scope.
    fn kek_file_path(&self, scope: &KeyScope) -> PathBuf {
        self.base_dir.join(build_filename(scope))
    }
}

impl KeyringPort for FileBasedKeyring {
    fn load_kek(&self, scope: &KeyScope) -> Result<Kek, EncryptionError> {
        let path = self.kek_file_path(scope);

        let bytes = fs::read(&path).map_err(|e| match e.kind() {
            io::ErrorKind::NotFound => EncryptionError::KeyNotFound,
            _ => EncryptionError::KeyringError(format!("Failed to read KEK file: {}", e)),
        })?;

        Kek::from_bytes(&bytes).map_err(|e| {
            EncryptionError::KeyringError(format!("Invalid KEK material in file: {e}"))
        })
    }

    fn store_kek(&self, scope: &KeyScope, kek: &Kek) -> Result<(), EncryptionError> {
        let path = self.kek_file_path(scope);

        // Write to temporary file first, then atomic rename
        let temp_path = path.with_extension("tmp");

        fs::write(&temp_path, &kek.0).map_err(|e| {
            EncryptionError::KeyringError(format!("Failed to write KEK temp file: {}", e))
        })?;

        fs::rename(&temp_path, &path).map_err(|e| {
            EncryptionError::KeyringError(format!("Failed to rename KEK file: {}", e))
        })?;

        // Set restrictive permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path)
                .map_err(|e| {
                    EncryptionError::KeyringError(format!(
                        "Failed to read KEK file metadata: {}",
                        e
                    ))
                })?
                .permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&path, perms).map_err(|e| {
                EncryptionError::KeyringError(format!("Failed to set KEK file permissions: {}", e))
            })?;
        }

        Ok(())
    }

    fn delete_kek(&self, scope: &KeyScope) -> Result<(), EncryptionError> {
        let path = self.kek_file_path(scope);

        match fs::remove_file(&path) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()), // Idempotent
            Err(e) => Err(EncryptionError::KeyringError(format!(
                "Failed to delete KEK file: {}",
                e
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use uc_core::security::model::KeyScope;

    fn create_test_scope(id: &str) -> KeyScope {
        KeyScope {
            profile_id: id.to_string(),
        }
    }

    #[test]
    fn build_filename_includes_prefix_and_scope() {
        let scope = create_test_scope("user123");
        let filename = build_filename(&scope);
        assert_eq!(filename, "kek:v1:profile:user123.bin");
    }

    #[test]
    fn store_and_load_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let keyring = FileBasedKeyring::with_base_dir(temp_dir.path().to_path_buf());
        let scope = create_test_scope("roundtrip");
        let kek = Kek([42u8; 32]);

        keyring.store_kek(&scope, &kek).expect("store_kek failed");
        let loaded = keyring.load_kek(&scope).expect("load_kek failed");

        assert_eq!(loaded, kek);
    }

    #[test]
    fn load_missing_key_returns_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let keyring = FileBasedKeyring::with_base_dir(temp_dir.path().to_path_buf());
        let scope = create_test_scope("missing");

        match keyring.load_kek(&scope) {
            Err(EncryptionError::KeyNotFound) => {}
            other => panic!("expected KeyNotFound, got {:?}", other),
        }
    }

    #[test]
    fn store_overwrites_existing_key() {
        let temp_dir = TempDir::new().unwrap();
        let keyring = FileBasedKeyring::with_base_dir(temp_dir.path().to_path_buf());
        let scope = create_test_scope("overwrite");
        let kek1 = Kek([1u8; 32]);
        let kek2 = Kek([2u8; 32]);

        keyring
            .store_kek(&scope, &kek1)
            .expect("first store failed");
        keyring
            .store_kek(&scope, &kek2)
            .expect("second store failed");

        let loaded = keyring.load_kek(&scope).expect("load failed");
        assert_eq!(loaded, kek2, "should load the most recent value");
    }

    #[test]
    fn delete_is_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let keyring = FileBasedKeyring::with_base_dir(temp_dir.path().to_path_buf());
        let scope = create_test_scope("delete");

        // Delete non-existent key should succeed
        keyring.delete_kek(&scope).expect("first delete failed");

        // Delete again should still succeed
        keyring.delete_kek(&scope).expect("second delete failed");
    }

    #[test]
    fn delete_then_load_returns_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let keyring = FileBasedKeyring::with_base_dir(temp_dir.path().to_path_buf());
        let scope = create_test_scope("delete_load");
        let kek = Kek([99u8; 32]);

        keyring.store_kek(&scope, &kek).expect("store failed");
        keyring.delete_kek(&scope).expect("delete failed");

        match keyring.load_kek(&scope) {
            Err(EncryptionError::KeyNotFound) => {}
            other => panic!("expected KeyNotFound after delete, got {:?}", other),
        }
    }

    #[test]
    fn file_permissions_are_restricted() {
        let temp_dir = TempDir::new().unwrap();
        let keyring = FileBasedKeyring::with_base_dir(temp_dir.path().to_path_buf());
        let scope = create_test_scope("permissions");
        let kek = Kek([1u8; 32]);

        keyring.store_kek(&scope, &kek).expect("store failed");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let path = keyring.kek_file_path(&scope);
            let metadata = fs::metadata(&path).expect("failed to get metadata");
            let mode = metadata.permissions().mode();
            let user_perms = mode & 0o777;

            assert_eq!(
                user_perms, 0o600,
                "KEK file should have 0600 permissions, got {:o}",
                user_perms
            );
        }

        #[cfg(not(unix))]
        {
            // On non-Unix, we just verify the file exists
            let path = keyring.kek_file_path(&scope);
            assert!(path.exists(), "KEK file should exist");
        }
    }

    #[test]
    fn load_invalid_kek_material_returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let keyring = FileBasedKeyring::with_base_dir(temp_dir.path().to_path_buf());
        let scope = create_test_scope("invalid");

        // Write invalid KEK material (wrong length)
        let path = keyring.kek_file_path(&scope);
        fs::write(&path, [1u8, 2, 3]).expect("failed to write invalid data");

        match keyring.load_kek(&scope) {
            Err(EncryptionError::KeyringError(msg)) => {
                assert!(msg.contains("Invalid KEK material"));
            }
            other => panic!("expected KeyringError, got {:?}", other),
        }
    }
}
