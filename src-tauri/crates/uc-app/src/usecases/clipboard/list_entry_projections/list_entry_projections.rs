//! Use case for listing clipboard entry projections
//! 列出剪贴板条目投影的用例

use anyhow::Result;
use std::sync::Arc;
use uc_core::ports::{
    ClipboardEntryRepositoryPort, ClipboardRepresentationRepositoryPort,
    ClipboardSelectionRepositoryPort,
};

/// DTO for clipboard entry projection (returned to command layer)
/// 剪贴板条目投影 DTO（返回给命令层）
#[derive(Debug, Clone, PartialEq)]
pub struct EntryProjectionDto {
    pub id: String,
    pub preview: String,
    pub has_detail: bool,
    pub size_bytes: i64,
    pub captured_at: i64,
    pub content_type: String,
    // TODO: is_encrypted, is_favorited to be implemented later
    pub is_encrypted: bool,
    pub is_favorited: bool,
    pub updated_at: i64,
    pub active_time: i64,
}

/// Error type for list projections use case
#[derive(Debug, thiserror::Error)]
pub enum ListProjectionsError {
    #[error("Invalid limit: {0}")]
    InvalidLimit(String),

    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error("Selection not found for entry {0}")]
    SelectionNotFound(String),

    #[error("Representation not found: {0}")]
    RepresentationNotFound(String),
}

/// Use case for listing clipboard entry projections
pub struct ListClipboardEntryProjections {
    entry_repo: Arc<dyn ClipboardEntryRepositoryPort>,
    selection_repo: Arc<dyn ClipboardSelectionRepositoryPort>,
    representation_repo: Arc<dyn ClipboardRepresentationRepositoryPort>,
    max_limit: usize,
}

impl ListClipboardEntryProjections {
    /// Create a new use case instance
    pub fn new(
        entry_repo: Arc<dyn ClipboardEntryRepositoryPort>,
        selection_repo: Arc<dyn ClipboardSelectionRepositoryPort>,
        representation_repo: Arc<dyn ClipboardRepresentationRepositoryPort>,
    ) -> Self {
        Self {
            entry_repo,
            selection_repo,
            representation_repo,
            max_limit: 1000,
        }
    }

    /// Execute the use case
    pub async fn execute(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<EntryProjectionDto>, ListProjectionsError> {
        // Validate limit
        if limit == 0 {
            return Err(ListProjectionsError::InvalidLimit(format!(
                "Must be at least 1, got {}",
                limit
            )));
        }

        if limit > self.max_limit {
            return Err(ListProjectionsError::InvalidLimit(format!(
                "Must be at most {}, got {}",
                self.max_limit, limit
            )));
        }

        // Query entries from repository
        let entries = self
            .entry_repo
            .list_entries(limit, offset)
            .await
            .map_err(|e| ListProjectionsError::RepositoryError(e.to_string()))?;

        let mut projections = Vec::with_capacity(entries.len());

        for entry in entries {
            let entry_id_str = entry.entry_id.inner().clone();
            let event_id_str = entry.event_id.inner().clone();
            let captured_at = entry.created_at_ms;

            // Get selection for this entry
            let selection = self
                .selection_repo
                .get_selection(&entry.entry_id)
                .await
                .map_err(|e| {
                    ListProjectionsError::RepositoryError(format!(
                        "Failed to get selection for {}: {}",
                        entry_id_str, e
                    ))
                })?
                .ok_or_else(|| ListProjectionsError::SelectionNotFound(entry_id_str.clone()))?;

            // Get preview representation
            let preview_rep_id = selection.selection.preview_rep_id.inner().clone();
            let representation = self
                .representation_repo
                .get_representation(&entry.event_id, &selection.selection.preview_rep_id)
                .await
                .map_err(|e| {
                    ListProjectionsError::RepositoryError(format!(
                        "Failed to get representation for {}/{}: {}",
                        event_id_str, preview_rep_id, e
                    ))
                })?
                .ok_or_else(|| {
                    ListProjectionsError::RepresentationNotFound(format!(
                        "{}/{}",
                        event_id_str, preview_rep_id
                    ))
                })?;

            // Build preview text
            let preview = if let Some(data) = representation.inline_data.as_ref() {
                String::from_utf8_lossy(data).trim().to_string()
            } else {
                format!("Image ({} bytes)", representation.size_bytes)
            };

            // Get content type from representation
            let content_type = representation
                .mime_type
                .as_ref()
                .map(|mt| mt.as_str().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            // Check if has detail (blob exists)
            let has_detail = representation.blob_id.is_some();

            projections.push(EntryProjectionDto {
                id: entry_id_str,
                preview,
                has_detail,
                size_bytes: entry.total_size,
                captured_at,
                content_type,
                is_encrypted: false, // TODO: implement later
                is_favorited: false, // TODO: implement later
                updated_at: captured_at,
                active_time: captured_at,
            });
        }

        Ok(projections)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uc_core::clipboard::{ClipboardEntry, PersistedClipboardRepresentation};
    use uc_core::ids::{EntryId, EventId, RepresentationId};
    use uc_core::ClipboardSelectionDecision;

    // Mock repositories for testing
    struct MockEntryRepository {
        entries: Vec<ClipboardEntry>,
    }

    struct MockSelectionRepository {
        selections: std::collections::HashMap<String, uc_core::ClipboardSelectionDecision>,
    }

    struct MockRepresentationRepository {
        representations:
            std::collections::HashMap<(String, String), uc_core::PersistedClipboardRepresentation>,
    }

    #[async_trait::async_trait]
    impl ClipboardEntryRepositoryPort for MockEntryRepository {
        async fn save_entry_and_selection(
            &self,
            _entry: &ClipboardEntry,
            _selection: &ClipboardSelectionDecision,
        ) -> Result<()> {
            unimplemented!()
        }

        async fn get_entry(&self, _entry_id: &EntryId) -> Result<Option<ClipboardEntry>> {
            unimplemented!()
        }

        async fn list_entries(&self, limit: usize, offset: usize) -> Result<Vec<ClipboardEntry>> {
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

    #[async_trait::async_trait]
    impl ClipboardSelectionRepositoryPort for MockSelectionRepository {
        async fn get_selection(
            &self,
            entry_id: &EntryId,
        ) -> Result<Option<uc_core::ClipboardSelectionDecision>> {
            Ok(self.selections.get(entry_id.inner()).cloned())
        }

        async fn delete_selection(&self, _entry_id: &EntryId) -> Result<()> {
            unimplemented!()
        }
    }

    #[async_trait::async_trait]
    impl ClipboardRepresentationRepositoryPort for MockRepresentationRepository {
        async fn get_representation(
            &self,
            event_id: &EventId,
            rep_id: &RepresentationId,
        ) -> Result<Option<PersistedClipboardRepresentation>> {
            Ok(self
                .representations
                .get(&(event_id.inner().clone(), rep_id.inner().clone()))
                .cloned())
        }

        async fn update_blob_id(
            &self,
            _representation_id: &RepresentationId,
            _blob_id: &uc_core::BlobId,
        ) -> Result<()> {
            unimplemented!()
        }

        async fn update_blob_id_if_none(
            &self,
            _representation_id: &RepresentationId,
            _blob_id: &uc_core::BlobId,
        ) -> Result<bool> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_validates_limit_zero() {
        let entry_repo = Arc::new(MockEntryRepository { entries: vec![] });
        let selection_repo = Arc::new(MockSelectionRepository {
            selections: std::collections::HashMap::new(),
        });
        let representation_repo = Arc::new(MockRepresentationRepository {
            representations: std::collections::HashMap::new(),
        });

        let use_case =
            ListClipboardEntryProjections::new(entry_repo, selection_repo, representation_repo);

        let result = use_case.execute(0, 0).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ListProjectionsError::InvalidLimit(_)));
    }

    #[tokio::test]
    async fn test_validates_limit_exceeds_max() {
        let entry_repo = Arc::new(MockEntryRepository { entries: vec![] });
        let selection_repo = Arc::new(MockSelectionRepository {
            selections: std::collections::HashMap::new(),
        });
        let representation_repo = Arc::new(MockRepresentationRepository {
            representations: std::collections::HashMap::new(),
        });

        let use_case =
            ListClipboardEntryProjections::new(entry_repo, selection_repo, representation_repo);

        let result = use_case.execute(2000, 0).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ListProjectionsError::InvalidLimit(_)));
    }
}
