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

    /// Update the entry active time.
    /// 更新条目的活跃时间。
    async fn touch_entry(&self, _entry_id: &EntryId, _active_time_ms: i64) -> Result<bool> {
        Ok(false)
    }

    /// Delete a clipboard entry.
    /// 删除剪贴板条目。
    ///
    /// # Arguments
    /// * `entry_id` - The entry ID to delete
    ///
    /// # Errors
    /// Returns error if database operation fails
    async fn delete_entry(&self, entry_id: &EntryId) -> Result<()>;
}
