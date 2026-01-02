//! Unified encryption module - combines key derivation and encryption

use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::encryption::Encryptor;

/// Unified encryption system
pub struct UnifiedEncryption {
    master_key: Arc<RwLock<Option<[u8; 32]>>>,
}

impl UnifiedEncryption {
    pub fn new() -> Self {
        Self {
            master_key: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn initialize_from_password(&self, password: &str) -> Result<()> {
        let key = derive_key_from_password(password);
        *self.master_key.write().await = Some(key);
        Ok(())
    }

    pub async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let key = self.get_key().await?;
        let encryptor = Encryptor::from_key(&key);
        encryptor.encrypt(data)
    }

    pub async fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let key = self.get_key().await?;
        let encryptor = Encryptor::from_key(&key);
        encryptor.decrypt(data)
    }

    pub async fn is_ready(&self) -> bool {
        self.master_key.read().await.is_some()
    }

    async fn get_key(&self) -> Result<[u8; 32]> {
        self.master_key
            .read()
            .await
            .ok_or_else(|| anyhow!("Encryption not initialized"))
    }

    pub async fn clear(&self) {
        *self.master_key.write().await = None;
    }
}

/// Derive a 32-byte encryption key from the user's password using SHA-256
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
    async fn test_unified_encryption() {
        let encryption = UnifiedEncryption::new();
        encryption.initialize_from_password("test_password").await.unwrap();
        assert!(encryption.is_ready().await);

        let plaintext = b"Hello, World!";
        let ciphertext = encryption.encrypt(plaintext).await.unwrap();
        assert_ne!(ciphertext, plaintext.to_vec());

        let decrypted = encryption.decrypt(&ciphertext).await.unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_key_determinism() {
        let key1 = derive_key_from_password("test_password");
        let key2 = derive_key_from_password("test_password");
        assert_eq!(key1, key2);
    }

    #[tokio::test]
    async fn test_clear() {
        let encryption = UnifiedEncryption::new();
        encryption.initialize_from_password("test_password").await.unwrap();
        assert!(encryption.is_ready().await);

        encryption.clear().await;
        assert!(!encryption.is_ready().await);
    }
}
