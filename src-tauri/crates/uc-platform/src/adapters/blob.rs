//! Placeholder blob materializer implementation
//! 占位符 blob 物化器实现

use anyhow::Result;
use async_trait::async_trait;
use uc_core::ports::{BlobMaterializerPort, BlobWriterPort};
use uc_core::{Blob, ContentHash};

// === Blob Materializer ===

#[async_trait]
impl BlobMaterializerPort for PlaceholderBlobMaterializerPort {
    async fn materialize(&self, _data: &[u8], _content_hash: &ContentHash) -> Result<Blob> {
        // TODO: Implement actual blob materialization
        // 实现实际的 blob 物化
        Err(anyhow::anyhow!("BlobMaterializerPort not implemented yet"))
    }
}

// === Blob Writer ===

/// Placeholder blob writer port implementation
/// 占位符 blob 写入器端口实现
#[derive(Debug, Clone)]
pub struct PlaceholderBlobWriterPort;

#[async_trait]
impl BlobWriterPort for PlaceholderBlobWriterPort {
    async fn write(&self, _data: &[u8], _content_hash: &ContentHash) -> Result<Blob> {
        // TODO: Implement actual blob writing
        // 实现实际的 blob 写入
        Err(anyhow::anyhow!("BlobWriterPort not implemented yet"))
    }
}

/// Placeholder blob materializer port implementation
/// 占位符 blob 物化器端口实现
#[derive(Debug, Clone)]
pub struct PlaceholderBlobMaterializerPort;
