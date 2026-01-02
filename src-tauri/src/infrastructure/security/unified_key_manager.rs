//! Unified key manager for encryption across P2P and WebDAV
//!
//! This module provides a centralized key management system that derives
//! a master encryption key from the user's encryption password using SHA-256.
//! The same master key is used for both P2P and WebDAV encryption.
//!
//! Key Derivation:
//! - Uses SHA-256 to directly hash the user's password into a 32-byte key
//! - This ensures all devices with the same password derive the same key
//! - No salt is needed, making the system simpler and more reliable

use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Unified key manager that handles master key derivation from user password
pub struct UnifiedKeyManager {
    /// Master encryption key (32 bytes for AES-256-GCM)
    master_key: Arc<RwLock<Option<[u8; 32]>>>,
}

impl UnifiedKeyManager {
    /// Create a new UnifiedKeyManager
    pub fn new() -> Self {
        Self {
            master_key: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize the master key from user encryption password
    ///
    /// Uses SHA-256 to directly hash the password into a 32-byte key.
    /// All devices with the same password will derive the same key.
    pub async fn initialize_from_password(&self, password: &str) -> Result<()> {
        let master_key = derive_key_from_password(password);
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
}

/// Derive a 32-byte encryption key from the user's password using SHA-256
///
/// # Arguments
/// * `password` - User's encryption password
///
/// # Returns
/// A 32-byte master key suitable for AES-256-GCM encryption
///
/// # Deterministic Behavior
/// This function is deterministic: the same password will always produce
/// the same key. This is intentional to ensure all devices with the same
/// password can decrypt each other's data.
pub fn derive_key_from_password(password: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    result.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_key_manager_initialization() {
        let manager = UnifiedKeyManager::new();

        assert!(!manager.is_initialized().await);

        manager
            .initialize_from_password("test_password")
            .await
            .unwrap();

        assert!(manager.is_initialized().await);

        let key = manager.get_key().await.unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_key_determinism() {
        let key1 = derive_key_from_password("test_password");
        let key2 = derive_key_from_password("test_password");

        // Same password should produce the same key
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_different_passwords_different_keys() {
        let key1 = derive_key_from_password("password1");
        let key2 = derive_key_from_password("password2");

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_same_password_same_key_across_instances() {
        // Simulate two devices using the same password
        let device_a_key = derive_key_from_password("shared_password");
        let device_b_key = derive_key_from_password("shared_password");

        assert_eq!(device_a_key, device_b_key);
    }

    #[test]
    fn test_derive_key_from_password_output_length() {
        let key = derive_key_from_password("any_password");
        assert_eq!(key.len(), 32);
    }

    #[tokio::test]
    async fn test_clear_key() {
        let manager = UnifiedKeyManager::new();

        manager
            .initialize_from_password("test_password")
            .await
            .unwrap();

        assert!(manager.is_initialized().await);

        manager.clear_key().await;
        assert!(!manager.is_initialized().await);
    }
}
