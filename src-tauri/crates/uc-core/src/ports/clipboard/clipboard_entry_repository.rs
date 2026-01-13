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

    /// List clipboard entries with pagination
    /// 列出剪贴板条目（分页）
    async fn list_entries(&self, limit: usize, offset: usize) -> Result<Vec<ClipboardEntry>>;
}
