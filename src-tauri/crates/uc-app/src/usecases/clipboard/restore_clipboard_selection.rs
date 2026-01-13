use anyhow::Result;
use std::sync::Arc;

use uc_core::{
    clipboard::{ObservedClipboardRepresentation, SystemClipboardSnapshot},
    ids::EntryId,
    ports::{
        BlobStorePort, ClipboardEntryRepositoryPort, ClipboardRepresentationRepositoryPort,
        ClipboardSelectionRepositoryPort, SystemClipboardPort,
    },
};

/// Reconstructs a system clipboard state from a historical clipboard entry,
/// restoring the primary selected representation and, when possible,
/// additional compatible representations captured in the same event.
pub struct RestoreClipboardSelectionUseCase<C, L, S, R, B>
where
    C: ClipboardEntryRepositoryPort,
    L: SystemClipboardPort,
    S: ClipboardSelectionRepositoryPort,
    R: ClipboardRepresentationRepositoryPort,
    B: BlobStorePort,
{
    clipboard_repo: Arc<C>,
    local_clipboard: Arc<L>,
    selection_repo: Arc<S>,
    representation_repo: Arc<R>,
    blob_store: Arc<B>,
}

impl<C, L, S, R, B> RestoreClipboardSelectionUseCase<C, L, S, R, B>
where
    C: ClipboardEntryRepositoryPort,
    L: SystemClipboardPort,
    S: ClipboardSelectionRepositoryPort,
    R: ClipboardRepresentationRepositoryPort,
    B: BlobStorePort,
{
    /// Creates a new use case instance that copies clipboard entries from history to the system clipboard.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use uc_app::usecases::clipboard::restore_clipboard_selection::RestoreClipboardSelectionUseCase;
    /// use uc_core::ports::{BlobStorePort, ClipboardEntryRepositoryPort, ClipboardRepresentationRepositoryPort, ClipboardSelectionRepositoryPort, SystemClipboardPort};
    /// // All parameters must implement their respective ports
    /// // let use_case = RestoreClipboardSelectionUseCase::new(
    /// //     Arc::new(clipboard_repo),
    /// //     Arc::new(local_clipboard),
    /// //     Arc::new(selection_repo),
    /// //     Arc::new(representation_repo),
    /// //     Arc::new(blob_store),
    /// // );
    /// ```
    pub fn new(
        clipboard_repo: Arc<C>,
        local_clipboard: Arc<L>,
        selection_repo: Arc<S>,
        representation_repo: Arc<R>,
        blob_store: Arc<B>,
    ) -> Self {
        Self {
            clipboard_repo,
            local_clipboard,
            selection_repo,
            representation_repo,
            blob_store,
        }
    }

    pub async fn execute(&self, entry_id: &EntryId) -> Result<()> {
        // 1. 读取 Entry
        let entry = self
            .clipboard_repo
            .get_entry(entry_id)
            .await?
            .ok_or(anyhow::anyhow!("Entry not found"))?;

        // 2. 获取 Selection 决策
        let selection = self
            .selection_repo
            .get_selection(entry_id)
            .await?
            .ok_or(anyhow::anyhow!("Selection not found"))?;

        // 3. 收集要恢复的所有 representation IDs
        let mut rep_ids = vec![selection.selection.paste_rep_id];
        rep_ids.extend(selection.selection.secondary_rep_ids);

        // 4. 加载所有 representations 的数据
        let mut representations = Vec::new();
        for rep_id in rep_ids {
            let rep = self
                .representation_repo
                .get_representation(&entry.event_id, &rep_id)
                .await?
                .ok_or(anyhow::anyhow!(
                    "Representation {} not found for event {}",
                    rep_id,
                    entry.event_id
                ))?;

            // 加载字节数据
            let bytes = if let Some(inline_data) = rep.inline_data {
                inline_data
            } else if let Some(blob_id) = rep.blob_id {
                self.blob_store.get(&blob_id).await?
            } else {
                return Err(anyhow::anyhow!("Representation has no data: {}", rep_id));
            };

            representations.push(ObservedClipboardRepresentation {
                id: rep.id,
                format_id: rep.format_id,
                mime: rep.mime_type,
                bytes,
            });
        }

        // 5. 构造 Snapshot
        let snapshot = SystemClipboardSnapshot {
            ts_ms: chrono::Utc::now().timestamp_millis(),
            representations,
        };

        // 6. 写入系统剪贴板
        self.local_clipboard.write_snapshot(snapshot)?;

        Ok(())
    }
}
