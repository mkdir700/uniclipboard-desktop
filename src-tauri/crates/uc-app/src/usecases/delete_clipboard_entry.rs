use anyhow::Result;
use std::sync::Arc;
use tracing::info;
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
    /// Constructs a `DeleteClipboardEntry` use case from repository and event-writer ports.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// // Assume `entry_repo`, `selection_repo`, and `event_writer` implement the required ports.
    /// let entry_repo = Arc::new(MockEntryRepo::new());
    /// let selection_repo = Arc::new(MockSelectionRepo::new());
    /// let event_writer = Arc::new(MockEventWriter::new());
    ///
    /// let use_case = DeleteClipboardEntry::from_ports(entry_repo, selection_repo, event_writer);
    /// ```
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

    /// Deletes a clipboard entry and its associated selection, event, and snapshot representations in the required order.
    ///
    /// Deletion order:
    /// 1. Verify the entry exists (returns an error if missing).
    /// 2. Delete the clipboard selection associated with the entry.
    /// 3. Delete the event and its snapshot representations using the entry's `event_id`.
    /// 4. Delete the clipboard entry itself.
    ///
    /// # Arguments
    /// * `entry_id` - ID of the clipboard entry to delete.
    ///
    /// # Returns
    /// `Ok(())` on success; an error if the entry is not found or if any repository/writer operation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::sync::Arc;
    /// # use tokio_test::block_on;
    /// # block_on(async {
    /// // Assume `uc` is a DeleteClipboardEntry instance created with valid ports
    /// // and `entry_id` is a valid EntryId.
    /// // uc.execute(&entry_id).await.unwrap();
    /// # });
    /// ```
    #[tracing::instrument(
        name = "usecase.delete_clipboard_entry.execute",
        skip(self),
        fields(entry_id = %entry_id)
    )]
    pub async fn execute(&self, entry_id: &EntryId) -> Result<()> {
        info!(entry_id = %entry_id, "Starting clipboard entry deletion");

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

        info!(entry_id = %entry_id, "Deleted clipboard entry successfully");
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
        /// Mock implementation of `get_entry` used by tests.
        ///
        /// Returns the configured `entry` wrapped in `Ok(Some(_))` when `should_fail_get` is `false`,
        /// returns `Ok(None)` if no entry is configured, and returns an `Err` when `should_fail_get` is `true`.
        ///
        /// # Examples
        ///
        /// ```
        /// // Construct a mock repository that will return a clipboard entry.
        /// let entry = ClipboardEntry { /* fields omitted */ };
        /// let repo = MockEntryRepo { entry: Some(entry.clone()), should_fail_get: false };
        /// let fetched = futures::executor::block_on(repo.get_entry(&entry.id)).unwrap();
        /// assert_eq!(fetched, Some(entry));
        /// ```
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

        /// Persists a clipboard entry together with its selection decision to the underlying stores.
        
        ///
        
        /// This saves both the provided `ClipboardEntry` and its associated `ClipboardSelectionDecision`.
        
        ///
        
        /// # Arguments
        
        ///
        
        /// * `entry` - The clipboard entry to persist.
        
        /// * `selection` - The selection decision associated with the entry.
        
        ///
        
        /// # Returns
        
        ///
        
        /// `Ok(())` on success, or an error if persistence fails.
        
        ///
        
        /// # Examples
        
        ///
        
        /// ```no_run
        
        /// # use std::sync::Arc;
        
        /// # async fn _example() -> Result<(), Box<dyn std::error::Error>> {
        
        /// // assuming `repo` is an implementation that provides this async method:
        
        /// // repo.save_entry_and_selection(&entry, &selection).await?;
        
        /// # Ok(())
        
        /// # }
        
        /// ```
        async fn save_entry_and_selection(
            &self,
            _entry: &ClipboardEntry,
            _selection: &uc_core::clipboard::ClipboardSelectionDecision,
        ) -> Result<()> {
            unimplemented!("Not used in tests")
        }

        /// Returns a page of clipboard entries using pagination parameters.
        ///
        /// # Parameters
        ///
        /// - `limit`: Maximum number of entries to return.
        /// - `offset`: Number of entries to skip before collecting results.
        ///
        /// # Returns
        ///
        /// A vector of `ClipboardEntry` containing up to `limit` entries starting at `offset`.
        ///
        /// # Examples
        ///
        /// ```
        /// # use futures::executor::block_on;
        /// # // `usecase` and `ClipboardEntry` would be available in real usage.
        /// # async fn _demo(usecase: &impl std::fmt::Debug) {}
        /// // let entries = block_on(usecase.list_entries(10, 0)).unwrap();
        /// // assert!(entries.len() <= 10);
        /// ```
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
        /// Retrieves the clipboard selection decision for the specified entry.
        ///
        /// Returns `Some(ClipboardSelectionDecision)` if a selection exists for the given entry, `None` otherwise.
        ///
        /// # Examples
        ///
        /// ```
        /// # async fn example<S: ClipboardSelectionRepositoryPort>(repo: &S, entry_id: &EntryId) {
        /// let decision = repo.get_selection(entry_id).await.unwrap();
        /// if let Some(d) = decision {
        ///     // use d
        /// } else {
        ///     // no selection for this entry
        /// }
        /// # }
        /// ```
        async fn get_selection(&self, _entry_id: &EntryId) -> Result<Option<uc_core::clipboard::ClipboardSelectionDecision>> {
            unimplemented!("Not used in tests")
        }

        /// Mock implementation of deleting a selection used in tests.
        ///
        /// Records that a deletion was attempted and, if configured, returns an error to simulate failure.
        ///
        /// # Examples
        ///
        /// ```
        /// // setup
        /// let mock = MockSelectionRepo { delete_called: std::sync::atomic::AtomicBool::new(false), should_fail_delete: false };
        /// // call (inside an async context)
        /// futures::executor::block_on(async {
        ///     mock.delete_selection(&EntryId::new()).await.unwrap();
        ///     assert!(mock.delete_called.load(std::sync::atomic::Ordering::SeqCst));
        /// });
        /// ```
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
        /// Placeholder implementation of `insert_event` for the mock event writer that panics if invoked.
        ///
        /// This mock method is not used in tests and will panic with the message `"Not used in tests"` when called.
        ///
        /// # Examples
        ///
        /// ```should_panic
        /// // In tests the mock's `insert_event` is not used; calling it will panic.
        /// // let mock = MockEventWriter::new();
        /// // futures::executor::block_on(mock.insert_event(&event, &representations));
        /// panic!("Not used in tests");
        /// ```
        async fn insert_event(
            &self,
            _event: &uc_core::clipboard::ClipboardEvent,
            _representations: &Vec<uc_core::clipboard::PersistedClipboardRepresentation>,
        ) -> Result<()> {
            unimplemented!("Not used in tests")
        }

        /// Simulates deletion of an event and its representations for testing.
        ///
        /// This mock marks that a deletion was attempted and either succeeds or returns an error
        /// based on the mock's configuration.
        ///
        /// # Parameters
        ///
        /// - `event_id`: Identifier of the event to delete (may be unused by the mock).
        ///
        /// # Returns
        ///
        /// `Ok(())` on success, `Err` with a descriptive message if the mock is configured to fail.
        ///
        /// # Examples
        ///
        /// ```rust,ignore
        /// // Construct a mock configured to succeed and verify deletion is recorded.
        /// let mock = MockEventWriter { delete_called: std::sync::atomic::AtomicBool::new(false), should_fail_delete: false };
        /// let event_id = EventId::new(); // example placeholder
        /// tokio::runtime::Runtime::new().unwrap().block_on(async {
        ///     let res = mock.delete_event_and_representations(&event_id).await;
        ///     assert!(res.is_ok());
        ///     assert!(mock.delete_called.load(std::sync::atomic::Ordering::SeqCst));
        /// });
        /// ```
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
