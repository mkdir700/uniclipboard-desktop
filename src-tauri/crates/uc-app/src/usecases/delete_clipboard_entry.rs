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

#[cfg(test)]
mod tests {
    use super::*;
    use uc_core::clipboard::ClipboardEntry;
    use uc_core::ids::{EntryId, EventId};
    use async_trait::async_trait;

    // Mock entry repository
    struct MockEntryRepo {
        entry: Option<ClipboardEntry>,
        should_fail_get: bool,
        should_fail_delete: bool,
        delete_called: std::sync::Arc<std::sync::atomic::AtomicBool>,
    }

    #[async_trait]
    impl ClipboardEntryRepositoryPort for MockEntryRepo {
        async fn get_entry(&self, _entry_id: &EntryId) -> Result<Option<ClipboardEntry>> {
            if self.should_fail_get {
                return Err(anyhow::anyhow!("Mock get_entry error"));
            }
            Ok(self.entry.clone())
        }

        async fn delete_entry(&self, _entry_id: &EntryId) -> Result<()> {
            self.delete_called.store(true, std::sync::atomic::Ordering::SeqCst);
            if self.should_fail_delete {
                return Err(anyhow::anyhow!("Mock delete_entry error"));
            }
            Ok(())
        }

        async fn save_entry_and_selection(
            &self,
            _entry: &ClipboardEntry,
            _selection: &uc_core::clipboard::ClipboardSelectionDecision,
        ) -> Result<()> {
            unimplemented!("Not used in tests")
        }

        async fn list_entries(&self, _limit: usize, _offset: usize) -> Result<Vec<ClipboardEntry>> {
            unimplemented!("Not used in tests")
        }
    }

    // Mock selection repository
    struct MockSelectionRepo {
        should_fail_delete: bool,
        delete_called: std::sync::Arc<std::sync::atomic::AtomicBool>,
    }

    #[async_trait]
    impl ClipboardSelectionRepositoryPort for MockSelectionRepo {
        async fn get_selection(&self, _entry_id: &EntryId) -> Result<Option<uc_core::clipboard::ClipboardSelectionDecision>> {
            unimplemented!("Not used in tests")
        }

        async fn delete_selection(&self, _entry_id: &EntryId) -> Result<()> {
            self.delete_called.store(true, std::sync::atomic::Ordering::SeqCst);
            if self.should_fail_delete {
                return Err(anyhow::anyhow!("Mock delete_selection error"));
            }
            Ok(())
        }
    }

    // Mock event writer
    struct MockEventWriter {
        should_fail_delete: bool,
        delete_called: std::sync::Arc<std::sync::atomic::AtomicBool>,
    }

    #[async_trait]
    impl ClipboardEventWriterPort for MockEventWriter {
        async fn insert_event(
            &self,
            _event: &uc_core::clipboard::ClipboardEvent,
            _representations: &Vec<uc_core::clipboard::PersistedClipboardRepresentation>,
        ) -> Result<()> {
            unimplemented!("Not used in tests")
        }

        async fn delete_event_and_representations(&self, _event_id: &EventId) -> Result<()> {
            self.delete_called.store(true, std::sync::atomic::Ordering::SeqCst);
            if self.should_fail_delete {
                return Err(anyhow::anyhow!("Mock delete_event_and_representations error"));
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_execute_deletes_all_related_data() {
        // Setup: Create mock repositories
        let delete_entry_called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let delete_selection_called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let delete_event_called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

        let entry_id = EntryId::from("test-entry".to_string());
        let event_id = EventId::from("test-event".to_string());

        let entry = ClipboardEntry::new(
            entry_id.clone(),
            event_id.clone(),
            1234567890,
            Some("Test Entry".to_string()),
            1024,
        );

        let entry_repo = MockEntryRepo {
            entry: Some(entry),
            should_fail_get: false,
            should_fail_delete: false,
            delete_called: delete_entry_called.clone(),
        };

        let selection_repo = MockSelectionRepo {
            should_fail_delete: false,
            delete_called: delete_selection_called.clone(),
        };

        let event_writer = MockEventWriter {
            should_fail_delete: false,
            delete_called: delete_event_called.clone(),
        };

        // Create use case with mocks
        let use_case = DeleteClipboardEntry::from_ports(
            Arc::new(entry_repo),
            Arc::new(selection_repo),
            Arc::new(event_writer),
        );

        // Execute deletion
        let result = use_case.execute(&entry_id).await;

        // Verify success
        assert!(result.is_ok(), "Deletion should succeed");

        // Verify all repositories were called in correct order
        assert!(delete_selection_called.load(std::sync::atomic::Ordering::SeqCst));
        assert!(delete_event_called.load(std::sync::atomic::Ordering::SeqCst));
        assert!(delete_entry_called.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_execute_returns_not_found_for_nonexistent_entry() {
        // Setup: Mock entry repo returns None
        let delete_entry_called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let delete_selection_called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let delete_event_called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

        let entry_id = EntryId::from("nonexistent".to_string());

        let entry_repo = MockEntryRepo {
            entry: None,
            should_fail_get: false,
            should_fail_delete: false,
            delete_called: delete_entry_called.clone(),
        };

        let selection_repo = MockSelectionRepo {
            should_fail_delete: false,
            delete_called: delete_selection_called.clone(),
        };

        let event_writer = MockEventWriter {
            should_fail_delete: false,
            delete_called: delete_event_called.clone(),
        };

        let use_case = DeleteClipboardEntry::from_ports(
            Arc::new(entry_repo),
            Arc::new(selection_repo),
            Arc::new(event_writer),
        );

        // Execute deletion
        let result = use_case.execute(&entry_id).await;

        // Assert error contains "not found"
        assert!(result.is_err(), "Should return error for nonexistent entry");
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not found"), "Error should contain 'not found': {}", err);

        // Verify delete methods were NOT called (entry didn't exist)
        assert!(!delete_selection_called.load(std::sync::atomic::Ordering::SeqCst));
        assert!(!delete_event_called.load(std::sync::atomic::Ordering::SeqCst));
        assert!(!delete_entry_called.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_execute_propagates_repository_errors() {
        // Setup: Mock returns error
        let delete_entry_called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let delete_selection_called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let delete_event_called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

        let entry_id = EntryId::from("test-entry".to_string());
        let event_id = EventId::from("test-event".to_string());

        let entry = ClipboardEntry::new(
            entry_id.clone(),
            event_id.clone(),
            1234567890,
            Some("Test Entry".to_string()),
            1024,
        );

        let entry_repo = MockEntryRepo {
            entry: Some(entry),
            should_fail_get: false,
            should_fail_delete: false,
            delete_called: delete_entry_called.clone(),
        };

        let selection_repo = MockSelectionRepo {
            should_fail_delete: true, // Will fail on delete_selection
            delete_called: delete_selection_called.clone(),
        };

        let event_writer = MockEventWriter {
            should_fail_delete: false,
            delete_called: delete_event_called.clone(),
        };

        let use_case = DeleteClipboardEntry::from_ports(
            Arc::new(entry_repo),
            Arc::new(selection_repo),
            Arc::new(event_writer),
        );

        // Execute deletion
        let result = use_case.execute(&entry_id).await;

        // Assert error is propagated
        assert!(result.is_err(), "Should return error when repo fails");
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Failed to delete selection"), "Error should indicate which operation failed: {}", err);
    }
}
