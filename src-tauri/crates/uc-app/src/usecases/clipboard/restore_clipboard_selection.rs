use anyhow::Result;
use std::sync::Arc;

use uc_core::{
    ids::EntryId,
    ports::{
        ClipboardEntryRepositoryPort, ClipboardRepresentationRepositoryPort,
        ClipboardSelectionRepositoryPort, SystemClipboardPort,
    },
};

/// Reconstructs a system clipboard state from a historical clipboard entry,
/// restoring the primary selected representation and, when possible,
/// additional compatible representations captured in the same event.
pub struct RestoreClipboardSelectionUseCase<C, L, S, R>
where
    C: ClipboardEntryRepositoryPort,
    L: SystemClipboardPort,
    S: ClipboardSelectionRepositoryPort,
    R: ClipboardRepresentationRepositoryPort,
{
    clipboard_repo: Arc<C>,
    local_clipboard: Arc<L>,
    selection_repo: Arc<S>,
    representation_repo: Arc<R>,
}

impl<C, L, S, R> RestoreClipboardSelectionUseCase<C, L, S, R>
where
    C: ClipboardEntryRepositoryPort,
    L: SystemClipboardPort,
    S: ClipboardSelectionRepositoryPort,
    R: ClipboardRepresentationRepositoryPort,
{
    /// Creates a new use case instance that copies clipboard entries from history to the system clipboard.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// // `repo` and `local` should implement the required ports: `ClipboardRepositoryPort` and `LocalClipboardPort`.
    /// let repo = Arc::new(/* impl of ClipboardRepositoryPort */);
    /// let local = Arc::new(/* impl of LocalClipboardPort */);
    /// let use_case = RestoreClipboardSelectionUseCase::new(repo, local);
    /// ```
    pub fn new(
        clipboard_repo: Arc<C>,
        local_clipboard: Arc<L>,
        selection_repo: Arc<S>,
        representation_repo: Arc<R>,
    ) -> Self {
        Self {
            clipboard_repo: clipboard_repo,
            local_clipboard: local_clipboard,
            selection_repo: selection_repo,
            representation_repo: representation_repo,
        }
    }

    pub async fn execute(&self, entry_id: &EntryId) -> Result<()> {
        let entry = self
            .clipboard_repo
            .get_entry(entry_id)
            .await?
            .ok_or(anyhow::anyhow!("Entry not found"))?;

        let selection_decision = self
            .selection_repo
            .get_selection(entry_id)
            .await?
            .ok_or(anyhow::anyhow!("Selection not found"))?;

        let representation_id = selection_decision.selection.paste_rep_id;
        let representation = self
            .representation_repo
            .get_representation(&entry.event_id, &representation_id)
            .await?;

        if representation.is_inline() {
            // 从 inline data 转为 snapshot
            todo!()
        } else if representation.is_blob() {
            todo!()
        } else {
            unreachable!()
        }

        let blob_id = representation.blob_id;

        Ok(())
    }
}
