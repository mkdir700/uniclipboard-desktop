use crate::{clipboard::ClipboardSelectionDecision, ids::EntryId};
use anyhow::Result;

#[async_trait::async_trait]
pub trait ClipboardSelectionRepositoryPort: Send + Sync {
    async fn get_selection(&self, entry_id: &EntryId)
        -> Result<Option<ClipboardSelectionDecision>>;
}
