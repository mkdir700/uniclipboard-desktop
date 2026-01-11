use crate::config::ClipboardStorageConfig;
use uc_core::{
    ports::clipboard::ClipboardRepresentationMaterializerPort, ObservedClipboardRepresentation,
    PersistedClipboardRepresentation,
};

/// Persisted representation of a clipboard payload.
///
/// Valid states:
/// 1. inline_data = Some, blob_id = None  -> inline payload
/// 2. inline_data = None, blob_id = Some  -> materialized blob
/// 3. inline_data = None, blob_id = None  -> lazy (metadata only)
/// 4. inline_data = Some, blob_id = Some  -> transitional / debugging
pub struct ClipboardRepresentationMaterializer<'a> {
    config: &'a ClipboardStorageConfig,
}

#[async_trait::async_trait]
impl ClipboardRepresentationMaterializerPort for ClipboardRepresentationMaterializer<'_> {
    async fn materialize(
        &self,
        observed: &ObservedClipboardRepresentation,
    ) -> anyhow::Result<PersistedClipboardRepresentation> {
        let inline_threshold_bytes = self.config.inline_threshold_bytes;
        let size_bytes = observed.bytes.len() as i64;
        let inline_data = if size_bytes <= inline_threshold_bytes {
            Some(observed.bytes.clone())
        } else {
            None
        };
        Ok(PersistedClipboardRepresentation::new(
            observed.id.clone(),
            observed.format_id.clone(),
            observed.mime.clone(),
            size_bytes,
            inline_data,
            None,
        ))
    }
}
