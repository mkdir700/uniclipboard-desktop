use crate::clipboard::PersistedClipboardRepresentation;
use crate::ids::{EventId, RepresentationId};
use crate::BlobId;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ClipboardRepresentationRepositoryPort: Send + Sync {
    async fn get_representation(
        &self,
        event_id: &EventId,
        representation_id: &RepresentationId,
    ) -> Result<Option<PersistedClipboardRepresentation>>;

    async fn update_blob_id(
        &self,
        representation_id: &RepresentationId,
        blob_id: &BlobId,
    ) -> Result<()>;

    /// Update blob_id for a representation, but only if it's currently None.
    ///
    /// # Returns
    /// - `true` if the update was applied (blob_id was None)
    /// - `false` if blob_id was already set (no-op)
    ///
    /// # Concurrency safety
    /// This should use compare-and-set semantics at the database level:
    /// ```sql
    /// UPDATE clipboard_snapshots_representations
    /// SET blob_id = ?
    /// WHERE id = ? AND blob_id IS NULL
    /// ```
    async fn update_blob_id_if_none(
        &self,
        representation_id: &RepresentationId,
        blob_id: &BlobId,
    ) -> Result<bool>;
}
