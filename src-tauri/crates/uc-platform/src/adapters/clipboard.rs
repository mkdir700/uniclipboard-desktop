//! Placeholder clipboard representation materializer port implementation
//! 占位符剪贴板表示物化器端口实现

use anyhow::Result;
use async_trait::async_trait;
use uc_core::clipboard::{ObservedClipboardRepresentation, PersistedClipboardRepresentation};
use uc_core::ports::ClipboardRepresentationMaterializerPort;

#[async_trait]
impl ClipboardRepresentationMaterializerPort
    for PlaceholderClipboardRepresentationMaterializerPort
{
    async fn materialize(
        &self,
        _observed: &ObservedClipboardRepresentation,
    ) -> Result<PersistedClipboardRepresentation> {
        // TODO: Implement actual clipboard representation materialization
        // 实现实际的剪贴板表示物化
        Err(anyhow::anyhow!(
            "ClipboardRepresentationMaterializerPort not implemented yet"
        ))
    }
}

/// Placeholder clipboard representation materializer port implementation
/// 占位符剪贴板表示物化器端口实现
#[derive(Debug, Clone)]
pub struct PlaceholderClipboardRepresentationMaterializerPort;
