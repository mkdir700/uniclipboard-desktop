use crate::{ids::EventId, ObservedClipboardRepresentation};
use anyhow::Result;

#[async_trait::async_trait]
pub trait ClipboardEventRepositoryPort: Send + Sync {
    async fn get_representation(
        &self,
        id: &EventId,
        representation_id: &str,
    ) -> Result<ObservedClipboardRepresentation>;
}
