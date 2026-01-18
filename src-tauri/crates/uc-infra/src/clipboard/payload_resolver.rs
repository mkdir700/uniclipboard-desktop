//! Clipboard Payload Resolver Implementation
//!
//! Resolves persisted clipboard representations into usable payloads.
//! Supports inline data, blob references, and lazy blob writing.

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info_span, Instrument};

use uc_core::clipboard::PersistedClipboardRepresentation;
use uc_core::ports::clipboard::ResolvedClipboardPayload;
use uc_core::ports::{
    BlobRepositoryPort, BlobWriterPort, ClipboardPayloadResolverPort,
    ClipboardRepresentationRepositoryPort, ContentHashPort,
};

/// Clipboard payload resolver implementation
pub struct ClipboardPayloadResolver {
    representation_repo: Arc<dyn ClipboardRepresentationRepositoryPort>,
    blob_writer: Arc<dyn BlobWriterPort>,
    blob_repo: Arc<dyn BlobRepositoryPort>,
    hasher: Arc<dyn ContentHashPort>,
}

impl ClipboardPayloadResolver {
    pub fn new(
        representation_repo: Arc<dyn ClipboardRepresentationRepositoryPort>,
        blob_writer: Arc<dyn BlobWriterPort>,
        blob_repo: Arc<dyn BlobRepositoryPort>,
        hasher: Arc<dyn ContentHashPort>,
    ) -> Self {
        Self {
            representation_repo,
            blob_writer,
            blob_repo,
            hasher,
        }
    }
}

#[async_trait]
impl ClipboardPayloadResolverPort for ClipboardPayloadResolver {
    async fn resolve(
        &self,
        representation: &PersistedClipboardRepresentation,
    ) -> Result<ResolvedClipboardPayload> {
        let span = info_span!(
            "infra.payload.resolve",
            representation_id = %representation.id,
            format_id = %representation.format_id,
        );
        async move {
            // Rule 1: Prefer inline data
            if let Some(inline_data) = &representation.inline_data {
                // Check if it's a placeholder (empty) or actual data
                if !inline_data.is_empty() {
                    debug!("Resolving from inline data");
                    let mime = representation
                        .mime_type
                        .clone()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| "application/octet-stream".to_string());
                    return Ok(ResolvedClipboardPayload::Inline {
                        mime,
                        bytes: inline_data.clone(),
                    });
                }
                // Empty inline_data means placeholder - continue to blob logic
            }

            // Rule 2: Has blob_id
            if let Some(blob_id) = &representation.blob_id {
                debug!("Resolving from existing blob reference");
                let mime = representation
                    .mime_type
                    .clone()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| "application/octet-stream".to_string());
                return Ok(ResolvedClipboardPayload::BlobRef {
                    mime,
                    blob_id: blob_id.clone(),
                });
            }

            // Rule 3: Lazy write - load bytes and persist to blob
            debug!("Lazy writing blob for representation");

            // Load raw bytes (from inline placeholder or other source)
            let raw_bytes = self.load_raw_bytes(representation).await?;

            // Calculate content hash
            let content_hash = self.hasher.hash_bytes(&raw_bytes)?;

            // Write to blob store (deduplicated)
            let blob = self.blob_writer.write(&raw_bytes, &content_hash).await?;

            // Update representation.blob_id (idempotent)
            let updated = self
                .representation_repo
                .update_blob_id_if_none(&representation.id, &blob.blob_id)
                .await?;

            if updated {
                debug!("Updated representation with new blob_id");
            } else {
                debug!("Representation already had blob_id (concurrent update)");
            }

            Ok(ResolvedClipboardPayload::BlobRef {
                mime: representation
                    .mime_type
                    .clone()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| "application/octet-stream".to_string()),
                blob_id: blob.blob_id,
            })
        }
        .instrument(span)
        .await
    }
}

impl ClipboardPayloadResolver {
    /// Load raw bytes for a representation.
    ///
    /// This is a helper for the lazy write case when we need to materialize
    /// the original data before writing to blob storage.
    async fn load_raw_bytes(
        &self,
        representation: &PersistedClipboardRepresentation,
    ) -> Result<Vec<u8>> {
        // For placeholder representations (empty inline_data, no blob_id),
        // we need to reload from the original source.
        //
        // TODO: This requires access to the original snapshot data.
        // For now, return an error as this needs to be implemented
        // with proper temp storage or snapshot caching.
        Err(anyhow::anyhow!(
            "Lazy blob writing not yet implemented for placeholders. \
             Representation {} has no retrievable data.",
            representation.id
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Add tests for inline data, blob reference, and error cases
}
