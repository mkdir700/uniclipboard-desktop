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
    /// Creates a new UnifiedEncryption with no master key set.
    ///
    /// The returned instance is ready for initialization via `initialize_from_password`.
    ///
    /// # Examples
    ///
    /// ```
    /// let enc = UnifiedEncryption::new();
    /// ```
    pub fn new() -> Self {
        Self {
            master_key: Arc::new(RwLock::new(None)),
        }
    }

    /// Derives a 32-byte master key from the given password and stores it, replacing any existing key.
    ///
    /// The derived key is written into the internal, thread-safe storage so subsequent calls to
    /// encryption/decryption methods will use the new key.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success.
    ///
    /// # Examples
    ///
    /// ```
    /// # use anyhow::Result;
    /// # use tokio::runtime::Runtime;
    /// # fn run() -> Result<()> {
    /// let rt = Runtime::new()?;
    /// rt.block_on(async {
    ///     let enc = UnifiedEncryption::new();
    ///     enc.initialize_from_password("correct horse battery staple").await.unwrap();
    ///     assert!(enc.is_ready().await);
    ///     Ok::<(), anyhow::Error>(())
    /// })?;
    /// # Ok(()) }
    /// # run().unwrap();
    /// ```
    pub async fn initialize_from_password(&self, password: &str) -> Result<()> {
        let key = derive_key_from_password(password);
        *self.master_key.write().await = Some(key);
        Ok(())
    }

    /// Encrypts the provided bytes with the current master key.
    ///
    /// Uses the stored 32-byte master key to produce ciphertext from `data`.
    ///
    /// # Returns
    ///
    /// The resulting ciphertext as a `Vec<u8>`.
    ///
    /// # Errors
    ///
    /// Returns an error if the encryption has not been initialized (no key stored) or if encryption fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// let ue = UnifiedEncryption::new();
    /// ue.initialize_from_password("s3cr3t").await.unwrap();
    /// let ciphertext = ue.encrypt(b"hello world").await.unwrap();
    /// assert!(ciphertext != b"hello world");
    /// # });
    /// ```
    pub async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let key = self.get_key().await?;
        let encryptor = Encryptor::from_key(&key);
        encryptor.encrypt(data)
    }

    /// Decrypts ciphertext using the currently stored master key.
    ///
    /// Returns the decrypted plaintext bytes on success. Returns an error if no master key is initialized or if decryption fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use futures::executor::block_on;
    ///
    /// let enc = UnifiedEncryption::new();
    /// block_on(enc.initialize_from_password("password")).unwrap();
    /// let ciphertext = block_on(enc.encrypt(b"secret")).unwrap();
    /// let plaintext = block_on(enc.decrypt(&ciphertext)).unwrap();
    /// assert_eq!(plaintext, b"secret");
    /// ```
    pub async fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let key = self.get_key().await?;
        let encryptor = Encryptor::from_key(&key);
        encryptor.decrypt(data)
    }

    /// Checks whether a master encryption key is set.
    ///
    /// # Examples
    ///
    /// ```
    /// #[tokio::test]
    /// async fn is_ready_example() {
    ///     let ue = UnifiedEncryption::new();
    ///     assert!(!ue.is_ready().await);
    /// }
    /// ```
    ///
    /// # Returns
    ///
    /// `true` if a 32-byte master key is present, `false` otherwise.
    pub async fn is_ready(&self) -> bool {
        self.master_key.read().await.is_some()
    }

    /// Returns the stored 32-byte master encryption key if one has been initialized.
    ///
    /// # Examples
    ///
    /// ```
    /// # use anyhow::Result;
    /// # use tokio::runtime::Runtime;
    /// # use crate::infrastructure::security::{UnifiedEncryption, derive_key_from_password};
    /// let rt = Runtime::new().unwrap();
    /// rt.block_on(async {
    ///     let ue = UnifiedEncryption::new();
    ///     ue.initialize_from_password("password123").await.unwrap();
    ///     let key = ue.get_key().await.unwrap();
    ///     assert_eq!(key, derive_key_from_password("password123"));
    /// });
    /// ```
    ///
    /// # Returns
    ///
    /// `Ok([u8; 32])` with the stored master key if initialized, `Err` if no key is stored (`Encryption not initialized`).
    async fn get_key(&self) -> Result<[u8; 32]> {
        self.master_key
            .read()
            .await
            .ok_or_else(|| anyhow!("Encryption not initialized"))
    }

    /// Clears the stored master encryption key.
    ///
    /// After calling this method the internal master key is removed and subsequent
    /// operations that require a key will behave as if encryption has not been initialized.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::runtime::Runtime;
    /// let rt = Runtime::new().unwrap();
    /// rt.block_on(async {
    ///     let enc = UnifiedEncryption::new();
    ///     enc.initialize_from_password("pass").await.unwrap();
    ///     assert!(enc.is_ready().await);
    ///     enc.clear().await;
    ///     assert!(!enc.is_ready().await);
    /// });
    /// ```
    pub async fn clear(&self) {
        *self.master_key.write().await = None;
    }
}

/// Derives a 32-byte key from the provided password.
///
/// Returns a 32-byte array suitable for use as a symmetric encryption key.
///
/// # Examples
///
/// ```
/// let key = derive_key_from_password("correct horse battery staple");
/// assert_eq!(key.len(), 32);
/// ```
pub fn derive_key_from_password(password: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    result.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that UnifiedEncryption can be initialized from a password and performs correct encryption and decryption.
    ///
    /// # Examples
    ///
    /// ```
    /// #[tokio::test]
    /// async fn example_unified_encryption() {
    ///     let encryption = UnifiedEncryption::new();
    ///     encryption.initialize_from_password("test_password").await.unwrap();
    ///     assert!(encryption.is_ready().await);
    ///
    ///     let plaintext = b"Hello, World!";
    ///     let ciphertext = encryption.encrypt(plaintext).await.unwrap();
    ///     assert_ne!(ciphertext, plaintext.to_vec());
    ///
    ///     let decrypted = encryption.decrypt(&ciphertext).await.unwrap();
    ///     assert_eq!(decrypted, plaintext);
    /// }
    /// ```
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