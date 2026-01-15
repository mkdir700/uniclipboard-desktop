use anyhow::Result;
use std::sync::Arc;

use uc_core::{
    ids::EntryId,
    ports::{
        BlobStorePort, ClipboardEntryRepositoryPort, ClipboardRepresentationRepositoryPort,
        ClipboardSelectionRepositoryPort,
    },
};

/// Get full clipboard entry detail
/// 获取剪贴板条目完整详情
pub struct GetEntryDetailUseCase {
    entry_repo: Arc<dyn ClipboardEntryRepositoryPort>,
    selection_repo: Arc<dyn ClipboardSelectionRepositoryPort>,
    representation_repo: Arc<dyn ClipboardRepresentationRepositoryPort>,
    blob_store: Arc<dyn BlobStorePort>,
}

/// Detail result from GetEntryDetailUseCase
/// GetEntryDetailUseCase 返回的详情结果
pub struct EntryDetailResult {
    pub id: String,
    pub content: String,
    pub size_bytes: i64,
    pub created_at_ms: i64,
}

impl GetEntryDetailUseCase {
    pub fn new(
        entry_repo: Arc<dyn ClipboardEntryRepositoryPort>,
        selection_repo: Arc<dyn ClipboardSelectionRepositoryPort>,
        representation_repo: Arc<dyn ClipboardRepresentationRepositoryPort>,
        blob_store: Arc<dyn BlobStorePort>,
    ) -> Self {
        Self {
            entry_repo,
            selection_repo,
            representation_repo,
            blob_store,
        }
    }

    pub async fn execute(&self, entry_id: &EntryId) -> Result<EntryDetailResult> {
        // Get entry
        let entry = self
            .entry_repo
            .get_entry(entry_id)
            .await?
            .ok_or(anyhow::anyhow!("Entry not found"))?;

        // Get selection
        let selection = self
            .selection_repo
            .get_selection(entry_id)
            .await?
            .ok_or(anyhow::anyhow!("Selection not found"))?;

        // Get preview representation
        let preview_rep = self
            .representation_repo
            .get_representation(&entry.event_id, &selection.selection.preview_rep_id)
            .await?
            .ok_or(anyhow::anyhow!("Preview representation not found"))?;

        // Determine if we need to read from blob
        let full_content = if let Some(blob_id) = preview_rep.blob_id {
            // Read from blob
            let blob_content = self.blob_store.get(&blob_id).await?;
            String::from_utf8_lossy(&blob_content).to_string()
        } else {
            // Use inline data
            String::from_utf8_lossy(
                preview_rep
                    .inline_data
                    .as_ref()
                    .ok_or(anyhow::anyhow!("No inline data"))?,
            )
            .to_string()
        };

        Ok(EntryDetailResult {
            id: entry.entry_id.to_string(),
            content: full_content,
            size_bytes: entry.total_size,
            created_at_ms: entry.created_at_ms,
        })
    }
}
