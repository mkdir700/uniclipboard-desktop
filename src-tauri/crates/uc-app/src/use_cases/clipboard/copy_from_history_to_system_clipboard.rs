use anyhow::Result;
use std::sync::Arc;

use uc_core::ports::{ClipboardRepositoryPort, LocalClipboardPort};

/// Copy a historical clipboard content back into the system clipboard
///
/// This use case represents a user intention to reuse a previously
/// recorded clipboard snapshot.
///
/// Responsibilities:
/// - Load clipboard content from storage
/// - Write content to system clipboard
pub struct CopyFromHistoryToSystemClipboard<C, L>
where
    C: ClipboardRepositoryPort,
    L: LocalClipboardPort,
{
    clipboard_repo: Arc<C>,
    local_clipboard: Arc<L>,
}

impl<C, L> CopyFromHistoryToSystemClipboard<C, L>
where
    C: ClipboardRepositoryPort,
    L: LocalClipboardPort,
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
    /// let use_case = CopyFromHistoryToSystemClipboard::new(repo, local);
    /// ```
    pub fn new(clipboard_repo: Arc<C>, local_clipboard: Arc<L>) -> Self {
        Self {
            clipboard_repo: clipboard_repo,
            local_clipboard: local_clipboard,
        }
    }

    /// Copies a historical clipboard entry identified by `hash` into the local system clipboard.
    ///
    /// If the repository contains content for the provided `hash`, that content is written to the local clipboard; if no entry exists for `hash`, the function performs no action.
    ///
    /// # Parameters
    ///
    /// - `hash`: Identifier of the clipboard entry to restore.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error propagated from the clipboard repository or local clipboard port.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assume `usecase` is an instance of `CopyFromHistoryToSystemClipboard`.
    /// // usecase.execute("some-hash").await?;
    /// ```
    pub async fn execute(&self, hash: &str) -> Result<()> {
        // 1. Write to system clipboard
        if let Some(content) = self.clipboard_repo.get_by_hash(hash).await? {
            self.local_clipboard.write(content).await?;
        }

        // 2. TODO: 同步到其他设备，网络 infra 暂未实现
        Ok(())
    }
}