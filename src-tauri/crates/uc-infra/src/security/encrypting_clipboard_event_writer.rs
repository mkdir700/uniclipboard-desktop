//! Encrypting clipboard event writer decorator.
//!
//! Wraps ClipboardEventWriterPort and encrypts inline_data before storage.

use std::sync::Arc;
use anyhow::{Context, Result};
use async_trait::async_trait;
use tracing::debug;

use uc_core::{
    clipboard::{ClipboardEvent, PersistedClipboardRepresentation},
    ids::{EventId, RepresentationId},
    ports::{ClipboardEventWriterPort, EncryptionPort, EncryptionSessionPort},
    security::model::EncryptionAlgo,
};

/// Decorator that encrypts representation inline_data before storage.
pub struct EncryptingClipboardEventWriter {
    inner: Arc<dyn ClipboardEventWriterPort>,
    encryption: Arc<dyn EncryptionPort>,
    session: Arc<dyn EncryptionSessionPort>,
}

impl EncryptingClipboardEventWriter {
    pub fn new(
        inner: Arc<dyn ClipboardEventWriterPort>,
        encryption: Arc<dyn EncryptionPort>,
        session: Arc<dyn EncryptionSessionPort>,
    ) -> Self {
        Self { inner, encryption, session }
    }

    /// Generate AAD for inline data encryption.
    fn aad_for_inline(event_id: &EventId, rep_id: &RepresentationId) -> Vec<u8> {
        format!("uc:inline:v1|{}|{}", event_id.as_ref(), rep_id.as_ref()).into_bytes()
    }
}

#[async_trait]
impl ClipboardEventWriterPort for EncryptingClipboardEventWriter {
    async fn insert_event(
        &self,
        event: &ClipboardEvent,
        representations: &Vec<PersistedClipboardRepresentation>,
    ) -> Result<()> {
        // Get master key from session
        let master_key = self.session.get_master_key().await
            .context("encryption session not ready - cannot encrypt clipboard data")?;

        // Encrypt inline_data for each representation
        let mut encrypted_reps = Vec::with_capacity(representations.len());

        for rep in representations {
            let encrypted_inline_data = if let Some(ref plaintext) = rep.inline_data {
                // Encrypt the inline data
                let aad = Self::aad_for_inline(&event.event_id, &rep.id);
                let encrypted_blob = self.encryption
                    .encrypt_blob(&master_key, plaintext, &aad, EncryptionAlgo::XChaCha20Poly1305)
                    .await
                    .context("failed to encrypt inline_data")?;

                // Serialize to bytes
                let encrypted_bytes = serde_json::to_vec(&encrypted_blob)
                    .context("failed to serialize encrypted inline_data")?;

                debug!("Encrypted inline_data for rep {} ({} bytes -> {} bytes)",
                    rep.id.as_ref(), plaintext.len(), encrypted_bytes.len());

                Some(encrypted_bytes)
            } else {
                None
            };

            // Create new representation with encrypted inline_data
            encrypted_reps.push(PersistedClipboardRepresentation::new(
                rep.id.clone(),
                rep.format_id.clone(),
                rep.mime_type.clone(),
                rep.size_bytes,
                encrypted_inline_data,
                rep.blob_id.clone(),
            ));
        }

        // Delegate to inner with encrypted representations
        self.inner.insert_event(event, &encrypted_reps).await
    }

    async fn delete_event_and_representations(&self, event_id: &EventId) -> Result<()> {
        // Deletion doesn't need encryption - just delegate
        self.inner.delete_event_and_representations(event_id).await
    }
}

#[cfg(test)]
mod tests {
    // Tests will be added after mock infrastructure is available
}
