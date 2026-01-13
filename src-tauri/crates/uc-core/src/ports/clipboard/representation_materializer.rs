use crate::clipboard::{ObservedClipboardRepresentation, PersistedClipboardRepresentation};

#[async_trait::async_trait]
pub trait ClipboardRepresentationMaterializerPort: Send + Sync {
    async fn materialize(
        &self,
        observed: &ObservedClipboardRepresentation,
    ) -> anyhow::Result<PersistedClipboardRepresentation>;
}
