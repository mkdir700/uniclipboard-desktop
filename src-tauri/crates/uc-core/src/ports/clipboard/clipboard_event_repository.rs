use crate::clipboard::{NewClipboardEvent, NewSnapshotRepresentation};
use anyhow::Result;

#[async_trait::async_trait]
pub trait ClipboardEventRepositoryPort: Send + Sync {
    async fn insert_event(
        &self,
        event: NewClipboardEvent,
        representations: Vec<NewSnapshotRepresentation>,
    ) -> Result<()>;
}
