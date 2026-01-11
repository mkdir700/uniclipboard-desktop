use crate::clipboard::{ObservedClipboardRepresentation, PersistedClipboardRepresentation};

#[async_trait::async_trait]
pub trait ClipboardRepresentationMaterializerPort {
    async fn materialize(
        &self,
        observed: &ObservedClipboardRepresentation,
    ) -> anyhow::Result<PersistedClipboardRepresentation>;
}
