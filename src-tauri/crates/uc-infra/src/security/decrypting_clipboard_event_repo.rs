//! Decrypting clipboard event repository decorator.
//!
//! Wraps ClipboardEventRepositoryPort and decrypts ObservedClipboardRepresentation.bytes on read.

use std::sync::Arc;
use anyhow::{Context, Result};
use async_trait::async_trait;
use tracing::debug;

use uc_core::{
    clipboard::ObservedClipboardRepresentation,
    ids::EventId,
    ports::{
        ClipboardEventRepositoryPort,
        EncryptionPort, EncryptionSessionPort,
    },
    security::model::EncryptedBlob,
};

/// Decorator that decrypts ObservedClipboardRepresentation.bytes on read.
pub struct DecryptingClipboardEventRepository {
    inner: Arc<dyn ClipboardEventRepositoryPort>,
    encryption: Arc<dyn EncryptionPort>,
    session: Arc<dyn EncryptionSessionPort>,
}

impl DecryptingClipboardEventRepository {
    pub fn new(
        inner: Arc<dyn ClipboardEventRepositoryPort>,
        encryption: Arc<dyn EncryptionPort>,
        session: Arc<dyn EncryptionSessionPort>,
    ) -> Self {
        Self { inner, encryption, session }
    }

    /// Generate AAD for representation bytes decryption.
    fn aad_for_inline(event_id: &EventId, rep_id: &str) -> Vec<u8> {
        format!("uc:inline:v1|{}|{}", event_id.as_ref(), rep_id).into_bytes()
    }
}

#[async_trait]
impl ClipboardEventRepositoryPort for DecryptingClipboardEventRepository {
    async fn get_representation(
        &self,
        event_id: &EventId,
        representation_id: &str,
    ) -> Result<ObservedClipboardRepresentation> {
        // Get from inner
        let mut observed = self.inner.get_representation(event_id, representation_id).await?;

        // Decrypt bytes if present
        if !observed.bytes.is_empty() {
            // Try to deserialize as encrypted blob
            match serde_json::from_slice::<EncryptedBlob>(&observed.bytes) {
                Ok(encrypted_blob) => {
                    // Get master key
                    let master_key = self.session.get_master_key().await
                        .context("encryption session not ready - cannot decrypt")?;

                    // Decrypt
                    let aad = Self::aad_for_inline(event_id, representation_id);
                    let plaintext = self.encryption
                        .decrypt_blob(&master_key, &encrypted_blob, &aad)
                        .await
                        .context("failed to decrypt representation bytes")?;

                    debug!("Decrypted representation bytes for {} ({} bytes)",
                        representation_id, plaintext.len());

                    observed.bytes = plaintext;
                }
                Err(_) => {
                    // Not encrypted blob format - this could be:
                    // 1. Old unencrypted data (hard fail as per spec)
                    // 2. Corrupted data
                    anyhow::bail!(
                        "representation {} bytes are not in encrypted format - \
                         data may be from before encryption was enabled or corrupted",
                        representation_id
                    );
                }
            }
        }

        Ok(observed)
    }
}

#[cfg(test)]
mod tests {
    // Tests will be added after mock infrastructure is available
}
