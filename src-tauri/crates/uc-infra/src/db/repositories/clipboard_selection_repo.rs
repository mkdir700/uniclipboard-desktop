//! Clipboard selection repository implementation
//! 剪贴板选择仓库实现

use crate::db::mappers::clipboard_selection_mapper::ClipboardSelectionRowMapper;
use crate::db::models::clipboard_selection::ClipboardSelectionRow;
use crate::db::ports::{DbExecutor, RowMapper};
use crate::db::schema::clipboard_selection;
use anyhow::Result;
use async_trait::async_trait;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};

use uc_core::clipboard::ClipboardSelectionDecision;
use uc_core::ids::EntryId;
use uc_core::ports::clipboard::ClipboardSelectionRepositoryPort;

/// In-memory clipboard selection repository (placeholder)
///
/// NOTE: This is a test helper implementation that returns None for all queries.
/// Use DieselClipboardSelectionRepository for production code with actual database queries.
///
/// 注意：这是测试辅助实现，对所有查询返回 None。
/// 生产代码请使用 DieselClipboardSelectionRepository 进行实际数据库查询。
pub struct InMemoryClipboardSelectionRepository;

impl InMemoryClipboardSelectionRepository {
    pub fn new() -> Self {
        Self
    }
}

impl Default for InMemoryClipboardSelectionRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ClipboardSelectionRepositoryPort for InMemoryClipboardSelectionRepository {
    async fn get_selection(
        &self,
        _entry_id: &EntryId,
    ) -> Result<Option<ClipboardSelectionDecision>> {
        // Placeholder implementation - always return None
        // 占位符实现 - 始终返回 None
        Ok(None)
    }

    async fn delete_selection(&self, _entry_id: &EntryId) -> Result<()> {
        // Placeholder implementation - no-op
        // 占位符实现 - 无操作
        Ok(())
    }
}

/// Diesel-based clipboard selection repository
///
/// Implements ClipboardSelectionRepositoryPort using SQLite database through Diesel ORM.
///
/// Diesel 实现的剪贴板选择仓库
/// 使用 Diesel ORM 通过 SQLite 数据库实现 ClipboardSelectionRepositoryPort。
pub struct DieselClipboardSelectionRepository<E>
where
    E: DbExecutor,
{
    executor: E,
}

impl<E> DieselClipboardSelectionRepository<E>
where
    E: DbExecutor,
{
    pub fn new(executor: E) -> Self {
        Self { executor }
    }
}

#[async_trait]
impl<E> ClipboardSelectionRepositoryPort for DieselClipboardSelectionRepository<E>
where
    E: DbExecutor,
{
    async fn get_selection(
        &self,
        entry_id: &EntryId,
    ) -> Result<Option<ClipboardSelectionDecision>> {
        let entry_id_str = entry_id.to_string();

        let row: Option<ClipboardSelectionRow> = self
            .executor
            .run(|conn| {
                Ok(clipboard_selection::table
                    .filter(clipboard_selection::entry_id.eq(&entry_id_str))
                    .first::<ClipboardSelectionRow>(conn)
                    .optional()?)
            })
            .map_err(|e| {
                log::error!(
                    "Failed to query clipboard_selection for entry_id '{}': {}",
                    entry_id_str,
                    e
                );
                e
            })?;

        match row {
            Some(r) => {
                let mapper = ClipboardSelectionRowMapper;
                let decision = mapper.to_domain(&r)?;
                Ok(Some(decision))
            }
            None => Ok(None),
        }
    }

    async fn delete_selection(&self, entry_id: &EntryId) -> Result<()> {
        let entry_id_str = entry_id.to_string();
        self.executor.run(|conn| {
            diesel::delete(clipboard_selection::table)
                .filter(clipboard_selection::entry_id.eq(&entry_id_str))
                .execute(conn)?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // In-memory repository tests
    #[tokio::test]
    async fn test_in_memory_repo_returns_none() {
        let repo = InMemoryClipboardSelectionRepository::new();
        let entry_id = EntryId::from("test-entry".to_string());

        let result = repo.get_selection(&entry_id).await.unwrap();

        assert!(result.is_none(), "InMemory repo should return None");
    }

    // Diesel repository integration tests
    use std::sync::Arc;
    use uc_core::clipboard::{ClipboardSelection, SelectionPolicyVersion};
    use uc_core::ids::RepresentationId;

    /// In-memory test executor for testing repositories
    #[derive(Clone)]
    struct TestDbExecutor {
        pool: Arc<crate::db::pool::DbPool>,
    }

    impl TestDbExecutor {
        fn new() -> Self {
            let pool = Arc::new(
                crate::db::pool::init_db_pool(":memory:").expect("Failed to create test DB pool"),
            );
            Self { pool }
        }
    }

    impl DbExecutor for TestDbExecutor {
        fn run<T>(
            &self,
            f: impl FnOnce(&mut diesel::SqliteConnection) -> anyhow::Result<T>,
        ) -> anyhow::Result<T> {
            let mut conn = self.pool.get()?;
            f(&mut conn)
        }
    }

    /// Helper function to insert complete test data for clipboard selection
    ///
    /// Foreign key chain:
    /// 1. clipboard_event (no dependencies)
    /// 2. blob (no dependencies)
    /// 3. clipboard_snapshot_representation (depends on: clipboard_event, blob)
    /// 4. clipboard_entry (depends on: clipboard_event)
    /// 5. clipboard_selection (depends on: clipboard_entry, clipboard_snapshot_representation x3)
    fn insert_complete_test_data(
        executor: &TestDbExecutor,
        decision: &ClipboardSelectionDecision,
    ) -> anyhow::Result<()> {
        use crate::db::mappers::clipboard_selection_mapper::ClipboardSelectionRowMapper;
        use crate::db::ports::InsertMapper;
        use crate::db::schema::{
            blob, clipboard_entry, clipboard_event, clipboard_selection,
            clipboard_snapshot_representation,
        };

        let mapper = ClipboardSelectionRowMapper;
        let new_row = mapper.to_row(decision)?;

        executor.run(|conn| {
            // 1. Insert clipboard_event
            diesel::insert_into(clipboard_event::table)
                .values((
                    clipboard_event::event_id.eq("test-event-1"),
                    clipboard_event::captured_at_ms.eq(1704067200000i64),
                    clipboard_event::source_device.eq("test-device"),
                    clipboard_event::snapshot_hash.eq(
                        "blake3v1:testhash123456789012345678901234567890123456789012345678901234",
                    ),
                ))
                .execute(conn)?;

            // 2. Insert blob (for snapshot representations)
            diesel::insert_into(blob::table)
                .values((
                    blob::blob_id.eq("test-blob-1"),
                    blob::storage_path.eq("/test/path/blob1"),
                    blob::storage_backend.eq("filesystem"),
                    blob::size_bytes.eq(100i64),
                    blob::content_hash.eq("hash1"),
                    blob::created_at_ms.eq(1704067200000i64),
                ))
                .execute(conn)?;

            // 3. Insert clipboard_snapshot_representation (primary, preview, paste)
            // Primary representation
            diesel::insert_into(clipboard_snapshot_representation::table)
                .values((
                    clipboard_snapshot_representation::id
                        .eq(decision.selection.primary_rep_id.as_str()),
                    clipboard_snapshot_representation::event_id.eq("test-event-1"),
                    clipboard_snapshot_representation::format_id.eq("public.text"),
                    clipboard_snapshot_representation::mime_type.eq("text/plain"),
                    clipboard_snapshot_representation::size_bytes.eq(100i64),
                    clipboard_snapshot_representation::inline_data.eq(Some(vec![1, 2, 3])),
                    clipboard_snapshot_representation::blob_id.eq(Option::<String>::None),
                ))
                .execute(conn)?;

            // Preview representation
            diesel::insert_into(clipboard_snapshot_representation::table)
                .values((
                    clipboard_snapshot_representation::id
                        .eq(decision.selection.preview_rep_id.as_str()),
                    clipboard_snapshot_representation::event_id.eq("test-event-1"),
                    clipboard_snapshot_representation::format_id.eq("public.html"),
                    clipboard_snapshot_representation::mime_type.eq("text/html"),
                    clipboard_snapshot_representation::size_bytes.eq(200i64),
                    clipboard_snapshot_representation::inline_data.eq(Some(vec![4, 5, 6])),
                    clipboard_snapshot_representation::blob_id.eq(Option::<String>::None),
                ))
                .execute(conn)?;

            // Paste representation
            diesel::insert_into(clipboard_snapshot_representation::table)
                .values((
                    clipboard_snapshot_representation::id
                        .eq(decision.selection.paste_rep_id.as_str()),
                    clipboard_snapshot_representation::event_id.eq("test-event-1"),
                    clipboard_snapshot_representation::format_id.eq("public.utf8-plain-text"),
                    clipboard_snapshot_representation::mime_type.eq("text/plain"),
                    clipboard_snapshot_representation::size_bytes.eq(150i64),
                    clipboard_snapshot_representation::inline_data.eq(Some(vec![7, 8, 9])),
                    clipboard_snapshot_representation::blob_id.eq(Option::<String>::None),
                ))
                .execute(conn)?;

            // 4. Insert clipboard_entry
            diesel::insert_into(clipboard_entry::table)
                .values((
                    clipboard_entry::entry_id.eq(decision.entry_id.as_str()),
                    clipboard_entry::event_id.eq("test-event-1"),
                    clipboard_entry::created_at_ms.eq(1704067200000i64),
                    clipboard_entry::title.eq(Option::<String>::None),
                    clipboard_entry::total_size.eq(450i64),
                    clipboard_entry::pinned.eq(false),
                    clipboard_entry::deleted_at_ms.eq(Option::<i64>::None),
                ))
                .execute(conn)?;

            // 5. Insert clipboard_selection
            diesel::insert_into(clipboard_selection::table)
                .values(&new_row)
                .execute(conn)?;

            Ok(())
        })
    }

    #[tokio::test]
    async fn test_diesel_repo_found() {
        let executor = TestDbExecutor::new();
        let repo = DieselClipboardSelectionRepository::new(executor.clone());

        // Insert test data with all foreign keys
        let decision = ClipboardSelectionDecision::new(
            EntryId::from("test-entry-1".to_string()),
            ClipboardSelection {
                primary_rep_id: RepresentationId::from("rep-primary".to_string()),
                secondary_rep_ids: vec![
                    RepresentationId::from("rep-sec1".to_string()),
                    RepresentationId::from("rep-sec2".to_string()),
                ],
                preview_rep_id: RepresentationId::from("rep-preview".to_string()),
                paste_rep_id: RepresentationId::from("rep-paste".to_string()),
                policy_version: SelectionPolicyVersion::V1,
            },
        );

        insert_complete_test_data(&executor, &decision).expect("Failed to insert test data");

        // Query the data
        let result = repo
            .get_selection(&decision.entry_id)
            .await
            .expect("Failed to execute query");

        assert!(result.is_some(), "Should find the inserted selection");
        let found = result.unwrap();
        assert_eq!(found.entry_id.as_str(), "test-entry-1");
        assert_eq!(found.selection.primary_rep_id.as_str(), "rep-primary");
        assert_eq!(found.selection.secondary_rep_ids.len(), 2);
        assert_eq!(found.selection.secondary_rep_ids[0].as_str(), "rep-sec1");
        assert_eq!(found.selection.secondary_rep_ids[1].as_str(), "rep-sec2");
        assert_eq!(found.selection.preview_rep_id.as_str(), "rep-preview");
        assert_eq!(found.selection.paste_rep_id.as_str(), "rep-paste");
        assert_eq!(found.selection.policy_version, SelectionPolicyVersion::V1);
    }

    #[tokio::test]
    async fn test_diesel_repo_not_found() {
        let executor = TestDbExecutor::new();
        let repo = DieselClipboardSelectionRepository::new(executor);

        let result = repo
            .get_selection(&EntryId::from("nonexistent".to_string()))
            .await
            .expect("Failed to execute query");

        assert!(result.is_none(), "Non-existent entry should return None");
    }

    #[tokio::test]
    async fn test_diesel_repo_single_secondary_rep() {
        let executor = TestDbExecutor::new();
        let repo = DieselClipboardSelectionRepository::new(executor.clone());

        let decision = ClipboardSelectionDecision::new(
            EntryId::from("test-entry-single".to_string()),
            ClipboardSelection {
                primary_rep_id: RepresentationId::from("rep-primary".to_string()),
                secondary_rep_ids: vec![RepresentationId::from("rep-only".to_string())],
                preview_rep_id: RepresentationId::from("rep-preview".to_string()),
                paste_rep_id: RepresentationId::from("rep-paste".to_string()),
                policy_version: SelectionPolicyVersion::V1,
            },
        );

        insert_complete_test_data(&executor, &decision).expect("Failed to insert test data");

        let result = repo
            .get_selection(&decision.entry_id)
            .await
            .expect("Failed to execute query");

        assert!(result.is_some());
        let found = result.unwrap();
        assert_eq!(found.selection.secondary_rep_ids.len(), 1);
        assert_eq!(found.selection.secondary_rep_ids[0].as_str(), "rep-only");
    }
}
