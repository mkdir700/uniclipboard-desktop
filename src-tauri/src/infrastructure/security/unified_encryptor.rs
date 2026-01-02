//! Unified encryptor using master key from UnifiedKeyManager
//!
//! This module provides a centralized encryption interface that works with
//! the UnifiedKeyManager to encrypt/decrypt data for both P2P and WebDAV.

use anyhow::Result;
use std::sync::Arc;

use super::encryption::Encryptor;
use super::unified_key_manager::UnifiedKeyManager;

/// Unified encryptor that uses the master key from UnifiedKeyManager
pub struct UnifiedEncryptor {
    key_manager: Arc<UnifiedKeyManager>,
}

impl UnifiedEncryptor {
    /// Create a new UnifiedEncryptor
    pub fn new(key_manager: Arc<UnifiedKeyManager>) -> Self {
        Self { key_manager }
    }

    /// Encrypt data using the master key
    ///
    /// # Arguments
    /// * `data` - Plaintext data to encrypt
    ///
    /// # Returns
    /// Ciphertext with nonce prepended (12 bytes nonce + ciphertext)
    ///
    /// # Errors
    /// Returns an error if:
    /// - Master key is not initialized
    /// - Encryption operation fails
    pub async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let key = self.key_manager.get_key().await?;
        let encryptor = Encryptor::from_key(&key);
        encryptor.encrypt(data)
    }

    /// Decrypt data using the master key
    ///
    /// # Arguments
    /// * `data` - Ciphertext with nonce prepended
    ///
    /// # Returns
    /// Decrypted plaintext
    ///
    /// # Errors
    /// Returns an error if:
    /// - Master key is not initialized
    /// - Ciphertext format is invalid
    /// - Decryption operation fails (authentication failed)
    pub async fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let key = self.key_manager.get_key().await?;
        let encryptor = Encryptor::from_key(&key);
        encryptor.decrypt(data)
    }

    /// Check if the encryptor is ready (master key initialized)
    pub async fn is_ready(&self) -> bool {
        self.key_manager.is_initialized().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unified_encryptor() {
        let key_manager = Arc::new(UnifiedKeyManager::new());
        let encryptor = UnifiedEncryptor::new(key_manager.clone());

        // Initialize with password
        key_manager
            .initialize_from_password("test_password")
            .await
            .unwrap();

        assert!(encryptor.is_ready().await);

        // Test encryption
        let plaintext = b"Hello, World!";
        let ciphertext = encryptor.encrypt(plaintext).await.unwrap();

        // Ciphertext should be different from plaintext
        assert_ne!(ciphertext, plaintext.to_vec());
        // Ciphertext should include nonce (12 bytes) + tag (16 bytes)
        assert!(ciphertext.len() > plaintext.len());

        // Test decryption
        let decrypted = encryptor.decrypt(&ciphertext).await.unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[tokio::test]
    async fn test_encrypt_without_initialization() {
        let key_manager = Arc::new(UnifiedKeyManager::new());
        let encryptor = UnifiedEncryptor::new(key_manager);

        let plaintext = b"Hello, World!";
        let result = encryptor.encrypt(plaintext).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Master key not initialized"));
    }

    #[tokio::test]
    async fn test_decrypt_invalid_ciphertext() {
        let key_manager = Arc::new(UnifiedKeyManager::new());
        let encryptor = UnifiedEncryptor::new(key_manager.clone());

        key_manager
            .initialize_from_password("test_password")
            .await
            .unwrap();

        // Invalid ciphertext (too short)
        let invalid_ciphertext = vec![0u8; 8];
        let result = encryptor.decrypt(&invalid_ciphertext).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_encrypt_decrypt_different_managers_same_password() {
        // Simulate two devices using the same password

        // Create first manager and encrypt
        let key_manager1 = Arc::new(UnifiedKeyManager::new());
        let encryptor1 = UnifiedEncryptor::new(key_manager1.clone());
        key_manager1
            .initialize_from_password("test_password")
            .await
            .unwrap();

        let plaintext = b"Shared secret";
        let ciphertext = encryptor1.encrypt(plaintext).await.unwrap();

        // Create second manager with same password and decrypt
        let key_manager2 = Arc::new(UnifiedKeyManager::new());
        let encryptor2 = UnifiedEncryptor::new(key_manager2.clone());
        key_manager2
            .initialize_from_password("test_password")
            .await
            .unwrap();

        let decrypted = encryptor2.decrypt(&ciphertext).await.unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[tokio::test]
    async fn test_different_passwords_cannot_decrypt() {
        let key_manager1 = Arc::new(UnifiedKeyManager::new());
        let encryptor1 = UnifiedEncryptor::new(key_manager1.clone());
        key_manager1
            .initialize_from_password("password1")
            .await
            .unwrap();

        let plaintext = b"Secret message";
        let ciphertext = encryptor1.encrypt(plaintext).await.unwrap();

        // Try to decrypt with different password
        let key_manager2 = Arc::new(UnifiedKeyManager::new());
        let encryptor2 = UnifiedEncryptor::new(key_manager2.clone());
        key_manager2
            .initialize_from_password("password2")
            .await
            .unwrap();

        let result = encryptor2.decrypt(&ciphertext).await;
        assert!(result.is_err());
    }
}
