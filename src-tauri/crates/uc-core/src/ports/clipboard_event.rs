use crate::clipboard::ClipboardEvent;
use crate::ids::EventId;
use crate::persistence::{NewClipboardEvent, NewSnapshotRepresentation};
use anyhow::Result;

#[async_trait::async_trait]
pub trait ClipboardEventWriterPort: Send + Sync {
    async fn insert_event(
        &self,
        event: &NewClipboardEvent,
        representations: &Vec<NewSnapshotRepresentation>,
    ) -> Result<()>;
}

#[async_trait::async_trait]
pub trait ClipboardEventReaderPort: Send + Sync {
    async fn get_event(&self, id: &EventId) -> Result<Option<ClipboardEvent>>;
}
