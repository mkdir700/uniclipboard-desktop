//! Clipboard Payload Resolver Port
//!
//! This port resolves persisted representations into directly usable payloads.
//!
//! **Semantic:** "resolve" = read-only access with best-effort availability

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
    /// 1. **Inline**: If payload state is Inline and inline_data exists → return `Inline`
    /// 2. **BlobReady**: If payload state is BlobReady and blob_id exists → return `BlobRef`
    /// 3. **Staged/Processing/Failed**: Best-effort return of bytes from cache/spool
    ///    - If bytes are available, return `Inline`
    ///    - Otherwise return an error (data not currently available)
    /// 4. **Lost**: Return an unrecoverable error
    ///
    /// # Notes
    /// - Resolver must be read-only; no lazy blob writes here.
    /// - Background workers are responsible for materializing blobs.
    async fn resolve(
        &self,
        representation: &PersistedClipboardRepresentation,
    ) -> anyhow::Result<ResolvedClipboardPayload>;
}
