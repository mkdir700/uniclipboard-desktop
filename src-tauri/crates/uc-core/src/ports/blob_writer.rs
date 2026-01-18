//! Blob Writer Port
//!
//! This port writes raw bytes to blob store with deduplication.
//!
//! **Semantic:** "write" = persistence with deduplication

use crate::{Blob, ContentHash};

#[async_trait::async_trait]
pub trait BlobWriterPort: Send + Sync {
    /// Write bytes to blob store with deduplication by content hash.
    ///
    /// # Idempotence guarantee
    /// - Data with identical `ContentHash` is written only once
    /// - Returns existing `Blob` or newly created one
    ///
    /// # Concurrency safety
    /// - Uses `ContentHash` as content-addressed key
    /// - `find_by_hash()` + `insert_blob()` combination provides natural deduplication
    async fn write(&self, data: &[u8], content_hash: &ContentHash) -> anyhow::Result<Blob>;
}
