//! Decrypting clipboard representation repository decorator.
//!
//! Wraps ClipboardRepresentationRepositoryPort and decrypts inline_data on read.

use std::sync::Arc;
use anyhow::{Context, Result};
use async_trait::async_trait;
use tracing::debug;

use uc_core::{
    clipboard::PersistedClipboardRepresentation,
    ids::{EventId, RepresentationId},
    ports::{ClipboardRepresentationRepositoryPort, EncryptionPort, EncryptionSessionPort},
    security::model::EncryptedBlob,
    BlobId,
};

/// Decorator that decrypts representation inline_data on read.
pub struct DecryptingClipboardRepresentationRepository {
    inner: Arc<dyn ClipboardRepresentationRepositoryPort>,
    encryption: Arc<dyn EncryptionPort>,
    session: Arc<dyn EncryptionSessionPort>,
}

impl DecryptingClipboardRepresentationRepository {
    pub fn new(
        inner: Arc<dyn ClipboardRepresentationRepositoryPort>,
        encryption: Arc<dyn EncryptionPort>,
        session: Arc<dyn EncryptionSessionPort>,
    ) -> Self {
        Self { inner, encryption, session }
    }

    /// Generate AAD for inline data decryption.
    fn aad_for_inline(event_id: &EventId, rep_id: &RepresentationId) -> Vec<u8> {
        format!("uc:inline:v1|{}|{}", event_id.as_ref(), rep_id.as_ref()).into_bytes()
    }
}

#[async_trait]
impl ClipboardRepresentationRepositoryPort for DecryptingClipboardRepresentationRepository {
    async fn get_representation(
        &self,
        event_id: &EventId,
        representation_id: &RepresentationId,
    ) -> Result<Option<PersistedClipboardRepresentation>> {
        // Get from inner
        let rep_opt = self.inner.get_representation(event_id, representation_id).await?;

        let Some(rep) = rep_opt else {
            return Ok(None);
        };

        // Decrypt inline_data if present
        let decrypted_inline_data = if let Some(ref encrypted_bytes) = rep.inline_data {
            // Deserialize encrypted blob
            let encrypted_blob: EncryptedBlob = serde_json::from_slice(encrypted_bytes)
                .context("failed to deserialize encrypted inline_data - data may be corrupted")?;

            // Get master key
            let master_key = self.session.get_master_key().await
                .context("encryption session not ready - cannot decrypt")?;

            // Decrypt
            let aad = Self::aad_for_inline(event_id, representation_id);
            let plaintext = self.encryption
                .decrypt_blob(&master_key, &encrypted_blob, &aad)
                .await
                .context("failed to decrypt inline_data")?;

            debug!("Decrypted inline_data for rep {} ({} bytes)",
                representation_id.as_ref(), plaintext.len());

            Some(plaintext)
        } else {
            None
        };

        // Return representation with decrypted data
        Ok(Some(PersistedClipboardRepresentation::new(
            rep.id,
            rep.format_id,
            rep.mime_type,
            rep.size_bytes,
            decrypted_inline_data,
            rep.blob_id,
        )))
    }

    async fn update_blob_id(
        &self,
        representation_id: &RepresentationId,
        blob_id: &BlobId,
    ) -> Result<()> {
        // No encryption needed for blob_id update - just delegate
        self.inner.update_blob_id(representation_id, blob_id).await
    }
}

#[cfg(test)]
mod tests {
    // Tests will be added after mock infrastructure is available
}
