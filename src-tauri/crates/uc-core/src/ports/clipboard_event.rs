use crate::clipboard::{ClipboardEvent, PersistedClipboardRepresentation};
use crate::ids::EventId;
use anyhow::Result;

#[async_trait::async_trait]
pub trait ClipboardEventWriterPort: Send + Sync {
    async fn insert_event(
        &self,
        event: &ClipboardEvent,
        representations: &Vec<PersistedClipboardRepresentation>,
    ) -> Result<()>;
}

#[async_trait::async_trait]
pub trait ClipboardEventReaderPort: Send + Sync {
    async fn get_event(&self, id: &EventId) -> Result<Option<ClipboardEvent>>;
}
