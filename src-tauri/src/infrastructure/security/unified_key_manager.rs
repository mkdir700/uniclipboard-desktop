//! Unified key manager for encryption across P2P and WebDAV
//!
//! This module provides a centralized key management system that derives
//! a master encryption key from the user's encryption password using Argon2id.
//! The same master key is used for both P2P and WebDAV encryption.

use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::password::PasswordManager;

/// Unified key manager that handles master key derivation from user password
pub struct UnifiedKeyManager {
    /// Master encryption key (32 bytes for AES-256-GCM)
    master_key: Arc<RwLock<Option<[u8; 32]>>>,
    /// Path to salt file for Argon2id KDF
    salt_file: PathBuf,
}

impl UnifiedKeyManager {
    /// Create a new UnifiedKeyManager
    pub fn new(salt_file: PathBuf) -> Self {
        Self {
            master_key: Arc::new(RwLock::new(None)),
            salt_file,
        }
    }

    /// Create a new UnifiedKeyManager using PasswordManager's salt file
    pub fn from_password_manager() -> Result<Self> {
        let salt_file = PasswordManager::get_salt_file_path();
        Ok(Self::new(salt_file))
    }

    /// Initialize the master key from user encryption password
    ///
    /// Uses Argon2id key derivation function with the following parameters:
    /// - Output length: 32 bytes (256 bits) for AES-256-GCM
    /// - Salt: loaded from or generated to salt file
    /// - Memory cost: 64 MiB (65536 KiB)
    /// - Time cost: 3 iterations
    /// - Parallelism: 4 lanes
    pub async fn initialize_from_password(&self, password: &str) -> Result<()> {
        let salt = self.load_or_generate_salt().await?;
        let master_key = argon2id_kdf(password, &salt)?;
        let mut key_guard = self.master_key.write().await;
        *key_guard = Some(master_key);
        Ok(())
    }

    /// Get the master key for encryption/decryption operations
    ///
    /// Returns an error if the key has not been initialized via initialize_from_password()
    pub async fn get_key(&self) -> Result<[u8; 32]> {
        let key_guard = self.master_key.read().await;
        key_guard.ok_or_else(|| {
            anyhow!("Master key not initialized. Call initialize_from_password() first.")
        })
    }

    /// Check if the master key has been initialized
    pub async fn is_initialized(&self) -> bool {
        let key_guard = self.master_key.read().await;
        key_guard.is_some()
    }

    /// Clear the master key from memory (e.g., after password change)
    pub async fn clear_key(&self) {
        let mut key_guard = self.master_key.write().await;
        *key_guard = None;
    }

    /// Load existing salt or generate a new one
    async fn load_or_generate_salt(&self) -> Result<Vec<u8>> {
        if self.salt_file.exists() {
            let salt = tokio::fs::read(&self.salt_file).await?;
            if salt.len() >= 16 {
                Ok(salt[..16].to_vec()) // Use first 16 bytes as salt
            } else {
                // Invalid salt file, generate new
                self.generate_and_save_salt().await
            }
        } else {
            self.generate_and_save_salt().await
        }
    }

    /// Generate a new salt and save it to the salt file
    async fn generate_and_save_salt(&self) -> Result<Vec<u8>> {
        use rand::Rng;

        // Generate salt before any await operations
        let salt: [u8; 16] = {
            let mut rng = rand::rng();
            rng.random()
        };

        // Ensure parent directory exists
        if let Some(parent) = self.salt_file.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&self.salt_file, &salt).await?;

        // Set file permissions on Unix (user read/write only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = tokio::fs::metadata(&self.salt_file).await?.permissions();
            perms.set_mode(0o600);
            tokio::fs::set_permissions(&self.salt_file, perms).await?;
        }

        Ok(salt.to_vec())
    }
}

/// Argon2id key derivation function
///
/// Derives a 32-byte encryption key from the user's password using Argon2id.
///
/// # Arguments
/// * `password` - User's encryption password
/// * `salt` - Salt for KDF (should be 16 bytes, cryptographically random)
///
/// # Returns
/// A 32-byte master key suitable for AES-256-GCM encryption
///
/// # Parameters
/// - Memory cost: 64 MiB (65536 KiB)
/// - Time cost: 3 iterations
/// - Parallelism: 4 lanes
/// - Output length: 32 bytes
pub fn argon2id_kdf(password: &str, salt: &[u8]) -> Result<[u8; 32]> {
    use argon2::{Algorithm, Argon2, Params, Version};

    // Argon2id parameters (as per OWASP recommendations)
    let params = Params::new(65536, 3, 4, None) // m=64 MiB, t=3, p=4
        .map_err(|e| anyhow!("Failed to create Argon2 params: {}", e))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| anyhow!("Failed to derive key: {}", e))?;

    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_key_manager_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let salt_file = temp_dir.path().join(".test_salt");
        let manager = UnifiedKeyManager::new(salt_file);

        assert!(!manager.is_initialized().await);

        manager
            .initialize_from_password("test_password")
            .await
            .unwrap();

        assert!(manager.is_initialized().await);

        let key = manager.get_key().await.unwrap();
        assert_eq!(key.len(), 32);
    }

    #[tokio::test]
    async fn test_key_determinism() {
        let temp_dir = TempDir::new().unwrap();
        let salt_file = temp_dir.path().join(".test_salt");
        let manager = UnifiedKeyManager::new(salt_file.clone());

        // Initialize with password
        manager
            .initialize_from_password("test_password")
            .await
            .unwrap();

        let key1 = manager.get_key().await.unwrap();

        // Clear and reinitialize with same password
        manager.clear_key().await;
        manager
            .initialize_from_password("test_password")
            .await
            .unwrap();

        let key2 = manager.get_key().await.unwrap();

        // Keys should be the same (same password, same salt)
        assert_eq!(key1, key2);
    }

    #[tokio::test]
    async fn test_different_passwords_different_keys() {
        let temp_dir = TempDir::new().unwrap();
        let salt_file = temp_dir.path().join(".test_salt");
        let manager = UnifiedKeyManager::new(salt_file);

        manager.initialize_from_password("password1").await.unwrap();
        let key1 = manager.get_key().await.unwrap();

        manager.clear_key().await;
        manager.initialize_from_password("password2").await.unwrap();
        let key2 = manager.get_key().await.unwrap();

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_argon2id_kdf() {
        let salt = vec![1u8; 16];
        let key1 = argon2id_kdf("password", &salt).unwrap();
        let key2 = argon2id_kdf("password", &salt).unwrap();

        assert_eq!(key1, key2);
        assert_eq!(key1.len(), 32);
    }

    #[test]
    fn test_argon2id_kdf_different_passwords() {
        let salt = vec![1u8; 16];
        let key1 = argon2id_kdf("password1", &salt).unwrap();
        let key2 = argon2id_kdf("password2", &salt).unwrap();

        assert_ne!(key1, key2);
    }
}
