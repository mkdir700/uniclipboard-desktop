//! Clipboard selection repository implementation
//! 剪贴板选择仓库实现

use anyhow::Result;
use async_trait::async_trait;

use uc_core::clipboard::ClipboardSelectionDecision;
use uc_core::ids::EntryId;
use uc_core::ports::clipboard::ClipboardSelectionRepositoryPort;

/// In-memory clipboard selection repository (placeholder)
///
/// NOTE: This is a placeholder implementation that returns None for all queries.
/// The actual database implementation will be added when the database schema
/// for clipboard_selection is finalized.
///
/// 注意：这是一个占位符实现，对所有查询返回 None。
/// 当 clipboard_selection 的数据库模式确定后，将添加实际的数据库实现。
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
        // TODO: Implement actual database query when schema is ready
        // 当模式准备好时实现实际的数据库查询
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_selection_returns_none() {
        let repo = InMemoryClipboardSelectionRepository::new();
        let entry_id = EntryId::from("test-entry".to_string());

        let result = repo.get_selection(&entry_id).await.unwrap();

        assert!(result.is_none(), "Placeholder should return None");
    }
}
