use anyhow::Result;

use crate::ids::EntryId;
use crate::persistence::{NewClipboardEntry, NewClipboardSelection};
use crate::ClipboardEntry;

#[async_trait::async_trait]
pub trait ClipboardEntryWriterPort: Send + Sync {
    async fn insert_entry(
        &self,
        entry: &NewClipboardEntry,
        selection: &NewClipboardSelection,
    ) -> Result<()>;
}

#[async_trait::async_trait]
pub trait ClipboardEntryReaderPort: Send + Sync {
    async fn get_entry(&self, id: &EntryId) -> Result<Option<ClipboardEntry>>;
}
