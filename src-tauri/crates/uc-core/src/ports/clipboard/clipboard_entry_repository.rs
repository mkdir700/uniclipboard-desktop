use anyhow::Result;

use crate::clipboard::{NewClipboardEntry, NewClipboardSelection};

#[async_trait::async_trait]
pub trait ClipboardEntryRepositoryPort: Send + Sync {
    async fn insert_entry(
        &self,
        entry: NewClipboardEntry,
        selection: NewClipboardSelection,
    ) -> Result<()>;
}
