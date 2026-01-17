//! Encrypted blob store decorator.
//!
//! Wraps an inner BlobStorePort and transparently encrypts/decrypts
//! blob data using the session's MasterKey.

use std::path::PathBuf;
use std::sync::Arc;
use anyhow::{Context, Result};
use async_trait::async_trait;
use tracing::debug;

use uc_core::{
    ports::{BlobStorePort, EncryptionPort, EncryptionSessionPort},
    security::model::EncryptionAlgo,
    BlobId,
};

/// Decorator that encrypts/decrypts blob data transparently.
pub struct EncryptedBlobStore {
    inner: Arc<dyn BlobStorePort>,
    encryption: Arc<dyn EncryptionPort>,
    session: Arc<dyn EncryptionSessionPort>,
}

impl EncryptedBlobStore {
    pub fn new(
        inner: Arc<dyn BlobStorePort>,
        encryption: Arc<dyn EncryptionPort>,
        session: Arc<dyn EncryptionSessionPort>,
    ) -> Self {
        Self { inner, encryption, session }
    }

    /// Generate AAD for blob encryption.
    fn aad_for_blob(blob_id: &BlobId) -> Vec<u8> {
        format!("uc:blob:v1|{}", blob_id.as_ref()).into_bytes()
    }
}

#[async_trait]
impl BlobStorePort for EncryptedBlobStore {
    async fn put(&self, blob_id: &BlobId, data: &[u8]) -> Result<PathBuf> {
        // 1. Get master key from session
        let master_key = self.session.get_master_key().await
            .context("encryption session not ready - cannot encrypt blob")?;

        // 2. Encrypt the data
        let aad = Self::aad_for_blob(blob_id);
        let encrypted_blob = self.encryption
            .encrypt_blob(&master_key, data, &aad, EncryptionAlgo::XChaCha20Poly1305)
            .await
            .context("failed to encrypt blob data")?;

        // 3. Serialize encrypted blob to bytes
        let encrypted_bytes = serde_json::to_vec(&encrypted_blob)
            .context("failed to serialize encrypted blob")?;

        debug!("Encrypted blob {} ({} bytes plaintext -> {} bytes ciphertext)",
            blob_id.as_ref(), data.len(), encrypted_bytes.len());

        // 4. Store encrypted bytes
        self.inner.put(blob_id, &encrypted_bytes).await
    }

    async fn get(&self, blob_id: &BlobId) -> Result<Vec<u8>> {
        // 1. Get encrypted bytes from inner store
        let encrypted_bytes = self.inner.get(blob_id).await
            .context("failed to read encrypted blob from storage")?;

        // 2. Deserialize encrypted blob
        let encrypted_blob: uc_core::security::model::EncryptedBlob = serde_json::from_slice(&encrypted_bytes)
            .context("failed to deserialize encrypted blob - data may be corrupted or unencrypted")?;

        // 3. Get master key from session
        let master_key = self.session.get_master_key().await
            .context("encryption session not ready - cannot decrypt blob")?;

        // 4. Decrypt the data
        let aad = Self::aad_for_blob(blob_id);
        let plaintext = self.encryption
            .decrypt_blob(&master_key, &encrypted_blob, &aad)
            .await
            .context("failed to decrypt blob - key mismatch or data corrupted")?;

        debug!("Decrypted blob {} ({} bytes)", blob_id.as_ref(), plaintext.len());

        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    // Tests will be added after mock infrastructure is available
}
