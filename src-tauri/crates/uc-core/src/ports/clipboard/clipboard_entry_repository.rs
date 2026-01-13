use anyhow::Result;

use crate::{clipboard::ClipboardEntry, ids::EntryId, ClipboardSelectionDecision};

#[async_trait::async_trait]
pub trait ClipboardEntryRepositoryPort: Send + Sync {
    async fn save_entry_and_selection(
        &self,
        entry: &ClipboardEntry,
        selection: &ClipboardSelectionDecision,
    ) -> Result<()>;
    async fn get_entry(&self, entry_id: &EntryId) -> Result<Option<ClipboardEntry>>;
}
