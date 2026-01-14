use anyhow::Result;
use std::sync::Arc;
use uc_core::ids::EntryId;
use uc_core::ports::{
    ClipboardEntryRepositoryPort,
    ClipboardSelectionRepositoryPort,
    ClipboardEventWriterPort,
};

/// Use case for deleting clipboard entries with all associated data.
/// 删除剪贴板条目及其所有关联数据的用例。
pub struct DeleteClipboardEntry {
    entry_repo: Arc<dyn ClipboardEntryRepositoryPort>,
    selection_repo: Arc<dyn ClipboardSelectionRepositoryPort>,
    event_writer: Arc<dyn ClipboardEventWriterPort>,
}

impl DeleteClipboardEntry {
    /// Create a new use case instance from trait objects.
    /// 从 trait 对象创建新的用例实例。
    pub fn from_ports(
        entry_repo: Arc<dyn ClipboardEntryRepositoryPort>,
        selection_repo: Arc<dyn ClipboardSelectionRepositoryPort>,
        event_writer: Arc<dyn ClipboardEventWriterPort>,
    ) -> Self {
        Self {
            entry_repo,
            selection_repo,
            event_writer,
        }
    }

    /// Execute the deletion workflow.
    /// 执行删除工作流。
    ///
    /// # Deletion Order / 删除顺序
    /// 1. Check if entry exists (returns NotFound if missing)
    /// 2. Delete clipboard_selection (depends on entry)
    /// 3. Delete clipboard_event + clipboard_snapshot_representation (via event_id)
    /// 4. Delete clipboard_entry (last, after dependencies removed)
    ///
    /// # Arguments / 参数
    /// * `entry_id` - The entry ID to delete
    ///
    /// # Returns / 返回值
    /// * `Ok(())` - Successfully deleted
    /// * `Err(NotFound)` - Entry does not exist
    /// * `Err(_)` - Database operation failed
    ///
    /// # Errors / 错误
    /// Returns error if:
    /// - Entry does not exist
    /// - Any database operation fails
    pub async fn execute(&self, entry_id: &EntryId) -> Result<()> {
        // 1. Verify entry exists
        let entry = self.entry_repo
            .get_entry(entry_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Clipboard entry not found: {}", entry_id))?;

        // 2. Delete selection (depends on entry)
        self.selection_repo
            .delete_selection(entry_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete selection: {}", e))?;

        // 3. Delete event and representations (via event_id)
        self.event_writer
            .delete_event_and_representations(&entry.event_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete event: {}", e))?;

        // 4. Delete entry (last, after dependencies removed)
        self.entry_repo
            .delete_entry(entry_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete entry: {}", e))?;

        Ok(())
    }
}
