//! Blob Writer Port
//!
//! This port writes raw bytes to blob store with deduplication.
//!
//! **Semantic:** "write_if_absent" = atomic write-if-absent with deduplication

use crate::{Blob, ContentHash};

#[async_trait::async_trait]
pub trait BlobWriterPort: Send + Sync {
    /// Write bytes to blob store if content_id doesn't already exist.
    ///
    /// # Atomic semantics
    /// - If `content_id` already exists → return existing `Blob`
    /// - If `content_id` doesn't exist → write and return new `Blob`
    ///
    /// # Idempotence guarantee
    /// - Multiple concurrent calls with same content_id produce same `Blob`
    /// - Data is written only once per content_id
    ///
    /// # Parameters
    /// - `content_id`: Hash-based identifier for deduplication (keyed hash)
    /// - `encrypted_bytes`: Encrypted payload to persist
    async fn write_if_absent(
        &self,
        content_id: &ContentHash,
        encrypted_bytes: &[u8],
    ) -> anyhow::Result<Blob>;

    /// Legacy write method (deprecated, use write_if_absent).
    ///
    /// Default implementation calls `write_if_absent`.
    #[deprecated(note = "Use write_if_absent for atomic semantics")]
    async fn write(&self, data: &[u8], content_hash: &ContentHash) -> anyhow::Result<Blob> {
        self.write_if_absent(content_hash, data).await
    }
}
