//! Clipboard representation materializer with owned config
//! 带有拥有所有权的配置的剪贴板表示物化器

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

use uc_core::clipboard::{ObservedClipboardRepresentation, PersistedClipboardRepresentation};
use uc_core::ports::clipboard::ClipboardRepresentationMaterializerPort;
use crate::config::clipboard_storage_config::ClipboardStorageConfig;

/// Clipboard representation materializer with owned config
/// 带有拥有所有权的配置的剪贴板表示物化器
pub struct ClipboardRepresentationMaterializer {
    config: Arc<ClipboardStorageConfig>,
}

impl ClipboardRepresentationMaterializer {
    /// Create a new materializer with the given config
    /// 使用给定配置创建新物化器
    pub fn new(config: Arc<ClipboardStorageConfig>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ClipboardRepresentationMaterializerPort for ClipboardRepresentationMaterializer {
    async fn materialize(
        &self,
        observed: &ObservedClipboardRepresentation,
    ) -> Result<PersistedClipboardRepresentation> {
        let inline_threshold_bytes = self.config.inline_threshold_bytes;
        let size_bytes = observed.bytes.len() as i64;

        // Decision: inline or blob?
        // 决策：内联还是 blob？
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
            None, // blob_id will be set later by blob materializer
        ))
    }
}
