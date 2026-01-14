use anyhow::Result;
use std::sync::Arc;
use uc_core::clipboard::ClipboardEntry;
use uc_core::ports::ClipboardEntryRepositoryPort;

/// Use case for listing clipboard entries with pagination
/// 列出剪贴板条目的用例（分页）
pub struct ListClipboardEntries {
    entry_repo: Arc<dyn ClipboardEntryRepositoryPort>,
    max_limit: usize,
}

impl ListClipboardEntries {
    /// Create a new use case instance from a trait object
    /// 从 trait 对象创建新的用例实例
    pub fn from_arc(entry_repo: Arc<dyn ClipboardEntryRepositoryPort>) -> Self {
        Self {
            entry_repo,
            max_limit: 1000, // Business rule: maximum 1000 entries per query
        }
    }

    /// Execute the query
    /// 执行查询
    ///
    /// # Arguments
    /// * `limit` - Maximum number of entries to return (1 to max_limit)
    /// * `offset` - Number of entries to skip
    ///
    /// # Returns
    /// Vector of clipboard entries
    ///
    /// # Errors
    /// Returns error if:
    /// - Limit is 0 or exceeds max_limit
    /// - Repository query fails
    pub async fn execute(&self, limit: usize, offset: usize) -> Result<Vec<ClipboardEntry>> {
        // Validate limit
        if limit == 0 {
            return Err(anyhow::anyhow!(
                "Invalid limit: {}. Must be at least 1",
                limit
            ));
        }

        if limit > self.max_limit {
            return Err(anyhow::anyhow!(
                "Invalid limit: {}. Must be at most {}",
                limit,
                self.max_limit
            ));
        }

        // Query repository
        self.entry_repo
            .list_entries(limit, offset)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to query clipboard entries: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uc_core::ids::{EntryId, EventId};

    // Mock repository for testing
    struct MockClipboardEntryRepository {
        entries: Vec<ClipboardEntry>,
        should_fail: bool,
    }

    #[async_trait::async_trait]
    impl ClipboardEntryRepositoryPort for MockClipboardEntryRepository {
        async fn save_entry_and_selection(
            &self,
            _entry: &ClipboardEntry,
            _selection: &uc_core::ClipboardSelectionDecision,
        ) -> Result<()> {
            unimplemented!()
        }

        async fn get_entry(&self, _entry_id: &EntryId) -> Result<Option<ClipboardEntry>> {
            unimplemented!()
        }

        async fn list_entries(&self, limit: usize, offset: usize) -> Result<Vec<ClipboardEntry>> {
            if self.should_fail {
                return Err(anyhow::anyhow!("Mock repository error"));
            }
            Ok(self
                .entries
                .iter()
                .skip(offset)
                .take(limit)
                .cloned()
                .collect())
        }

        async fn delete_entry(&self, _entry_id: &EntryId) -> Result<()> {
            unimplemented!()
        }
    }

    fn create_test_entry(id_str: &str, timestamp: i64) -> ClipboardEntry {
        ClipboardEntry::new(
            EntryId::from_str(id_str),
            EventId::from_str(id_str),
            timestamp,
            Some(format!("Entry {}", id_str)),
            100 * id_str.len() as i64,
        )
    }

    #[tokio::test]
    async fn test_execute_returns_entries() {
        let entries = vec![
            create_test_entry("entry-1", 1000),
            create_test_entry("entry-2", 2000),
            create_test_entry("entry-3", 3000),
        ];

        let repo = MockClipboardEntryRepository {
            entries,
            should_fail: false,
        };

        let repo_arc: Arc<dyn ClipboardEntryRepositoryPort> = Arc::new(repo);
        let use_case = ListClipboardEntries::from_arc(repo_arc);
        let result = use_case.execute(10, 0).await.unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].entry_id.inner(), "entry-1");
    }

    #[tokio::test]
    async fn test_execute_respects_limit() {
        let entries = vec![
            create_test_entry("entry-1", 1000),
            create_test_entry("entry-2", 2000),
            create_test_entry("entry-3", 3000),
        ];

        let repo = MockClipboardEntryRepository {
            entries,
            should_fail: false,
        };

        let repo_arc: Arc<dyn ClipboardEntryRepositoryPort> = Arc::new(repo);
        let use_case = ListClipboardEntries::from_arc(repo_arc);
        let result = use_case.execute(2, 0).await.unwrap();

        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_execute_respects_offset() {
        let entries = vec![
            create_test_entry("entry-1", 1000),
            create_test_entry("entry-2", 2000),
            create_test_entry("entry-3", 3000),
        ];

        let repo = MockClipboardEntryRepository {
            entries,
            should_fail: false,
        };

        let repo_arc: Arc<dyn ClipboardEntryRepositoryPort> = Arc::new(repo);
        let use_case = ListClipboardEntries::from_arc(repo_arc);
        let result = use_case.execute(10, 1).await.unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].entry_id.inner(), "entry-2");
    }

    #[tokio::test]
    async fn test_execute_rejects_zero_limit() {
        let repo = MockClipboardEntryRepository {
            entries: vec![],
            should_fail: false,
        };

        let repo_arc: Arc<dyn ClipboardEntryRepositoryPort> = Arc::new(repo);
        let use_case = ListClipboardEntries::from_arc(repo_arc);
        let result = use_case.execute(0, 0).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid limit"));
    }

    #[tokio::test]
    async fn test_execute_rejects_excessive_limit() {
        let repo = MockClipboardEntryRepository {
            entries: vec![],
            should_fail: false,
        };

        let repo_arc: Arc<dyn ClipboardEntryRepositoryPort> = Arc::new(repo);
        let use_case = ListClipboardEntries::from_arc(repo_arc);
        let result = use_case.execute(2000, 0).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Must be at most"));
    }

    #[tokio::test]
    async fn test_execute_propagates_repository_errors() {
        let repo = MockClipboardEntryRepository {
            entries: vec![],
            should_fail: true,
        };

        let repo_arc: Arc<dyn ClipboardEntryRepositoryPort> = Arc::new(repo);
        let use_case = ListClipboardEntries::from_arc(repo_arc);
        let result = use_case.execute(10, 0).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to query"));
    }
}
