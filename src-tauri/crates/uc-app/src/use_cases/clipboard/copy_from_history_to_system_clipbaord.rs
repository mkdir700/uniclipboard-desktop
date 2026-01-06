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
    pub fn new(clipboard_repo: Arc<C>, local_clipboard: Arc<L>) -> Self {
        Self {
            clipboard_repo: clipboard_repo,
            local_clipboard: local_clipboard,
        }
    }

    pub async fn execute(&self, hash: &str) -> Result<()> {
        // 1. Write to system clipboard
        if let Some(content) = self.clipboard_repo.get_by_hash(hash).await? {
            self.local_clipboard.write(&content).await?;
        }

        // 2. TODO: 同步到其他设备，网络 infra 暂未实现
        Ok(())
    }
}
