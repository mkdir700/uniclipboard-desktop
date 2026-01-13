//! Placeholder blob materializer implementation
//! 占位符 blob 物化器实现

use uc_core::ports::BlobMaterializerPort;
use uc_core::{Blob, ContentHash};
use anyhow::Result;
use async_trait::async_trait;

// === Blob Materializer ===

#[async_trait]
impl BlobMaterializerPort for PlaceholderBlobMaterializerPort {
    async fn materialize(&self, _data: &[u8], _content_hash: &ContentHash) -> Result<Blob> {
        // TODO: Implement actual blob materialization
        // 实现实际的 blob 物化
        Err(anyhow::anyhow!("BlobMaterializerPort not implemented yet"))
    }
}

/// Placeholder blob materializer port implementation
/// 占位符 blob 物化器端口实现
#[derive(Debug, Clone)]
pub struct PlaceholderBlobMaterializerPort;
