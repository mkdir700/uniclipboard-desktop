use anyhow::Result;

use crate::clipboard::{ClipboardEntry, ClipboardSelectionDecision};
use crate::ids::EntryId;

#[async_trait::async_trait]
pub trait ClipboardEntryWriterPort: Send + Sync {
    async fn insert_entry(
        &self,
        entry: &ClipboardEntry,
        selection: &ClipboardSelectionDecision,
    ) -> Result<()>;
}

#[async_trait::async_trait]
pub trait ClipboardEntryReaderPort: Send + Sync {
    async fn get_entry(&self, id: &EntryId) -> Result<Option<ClipboardEntry>>;
}
