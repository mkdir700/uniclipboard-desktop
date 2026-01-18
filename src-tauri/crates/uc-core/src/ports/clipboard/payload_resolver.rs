//! Clipboard Payload Resolver Port
//!
//! This port resolves persisted representations into directly usable payloads.
//!
//! **Semantic:** "resolve" = on-demand loading with lazy blob write

use crate::clipboard::PersistedClipboardRepresentation;
use crate::BlobId;

/// Result of resolving a clipboard representation into a usable payload
#[derive(Debug, Clone)]
pub enum ResolvedClipboardPayload {
    /// Inline data available (small content or preview)
    Inline { mime: String, bytes: Vec<u8> },

    /// Reference to blob storage (large content)
    BlobRef { mime: String, blob_id: BlobId },
}

#[async_trait::async_trait]
pub trait ClipboardPayloadResolverPort: Send + Sync {
    /// Resolve a persisted clipboard representation into a usable payload.
    ///
    /// # Resolution rules
    /// 1. **Prefer inline**: If `inline_data` available and complete → return `Inline`
    /// 2. **Has blob**: If `blob_id` exists → return `BlobRef`
    /// 3. **Lazy write**: Otherwise:
    ///    - Load raw bytes (from inline_data or temp storage)
    ///    - Calculate `ContentHash`
    ///    - Call `BlobWriterPort::write()` to persist
    ///    - Write back `representation.blob_id` (idempotent)
    ///    - Return `BlobRef`
    ///
    /// # Idempotence guarantee
    /// - Multiple resolves of same rep yield identical `blob_id`
    /// - Concurrent resolve: `update_blob_id` only takes effect when `None`
    async fn resolve(
        &self,
        representation: &PersistedClipboardRepresentation,
    ) -> anyhow::Result<ResolvedClipboardPayload>;
}
