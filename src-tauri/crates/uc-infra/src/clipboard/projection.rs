use anyhow::Result;
use uc_app::ports::ClipboardEntryProjectionQueryPort;
use uc_app::ClipboardEntryProjection;
use uc_core::ids::EntryId;
use uc_core::ports::ClipboardSelectionRepositoryPort;

pub struct ClipboardProjectionReader {
    selection_repository: Box<dyn ClipboardSelectionRepositoryPort>,
}

#[async_trait::async_trait]
impl ClipboardEntryProjectionQueryPort for ClipboardProjectionReader {
    async fn get_projection(&self, entry_id: &EntryId) -> Result<ClipboardEntryProjection> {
        let selection = self.selection_repository.get_selection(entry_id).await?;
        todo!()
    }

    async fn list_projections(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<ClipboardEntryProjection>> {
        todo!()
    }
}
