use crate::clipboard::PersistedClipboardRepresentation;
use crate::ids::{EventId, RepresentationId};
use crate::BlobId;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ClipboardRepresentationRepositoryPort {
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
}
