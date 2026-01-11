use anyhow::Result;

use crate::models::ClipboardEntryProjection;
use uc_core::ids::EntryId;

#[async_trait::async_trait]
pub trait ClipboardEntryProjectionQueryPort {
    async fn list_projections(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<ClipboardEntryProjection>>;

    async fn get_projection(&self, entry_id: &EntryId) -> Result<ClipboardEntryProjection>;
}
