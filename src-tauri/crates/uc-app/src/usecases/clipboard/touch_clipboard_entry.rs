use anyhow::Result;
use std::sync::Arc;
use std::time::SystemTime;

use uc_core::ids::EntryId;
use uc_core::ports::ClipboardEntryRepositoryPort;

/// Update clipboard entry active time.
///
/// 更新剪贴板条目的活跃时间。
pub struct TouchClipboardEntryUseCase {
    entry_repo: Arc<dyn ClipboardEntryRepositoryPort>,
}

impl TouchClipboardEntryUseCase {
    pub fn new(entry_repo: Arc<dyn ClipboardEntryRepositoryPort>) -> Self {
        Self { entry_repo }
    }

    pub async fn execute(&self, entry_id: &EntryId) -> Result<bool> {
        let now_ms = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| anyhow::anyhow!("Failed to get system time: {}", e))?
            .as_millis() as i64;

        self.entry_repo.touch_entry(entry_id, now_ms).await
    }
}
