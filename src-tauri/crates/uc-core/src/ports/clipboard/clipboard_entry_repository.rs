use anyhow::Result;

use crate::{
    ids::{EntryId, EventId},
    BlobId, EntrySelection,
};

pub trait ClipboardEntryRepositoryPort: Send + Sync {
    fn get_selection(&self, entry_id: &EntryId) -> Result<EntrySelection>;

    fn get_event_id_by_entry_id(&self, entry_id: &EntryId) -> Result<EventId>;

    /// 把 PendingBlob -> MaterializedBlob
    fn update_selection_to_blob(
        &self,
        entry_id: &EntryId,
        blob_id: &BlobId,
        mime: Option<String>,
    ) -> Result<()>;
}
