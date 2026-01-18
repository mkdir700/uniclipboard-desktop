//! Placeholder blob writer implementation
//! 占位符 blob 写入器实现

use anyhow::Result;
use async_trait::async_trait;
use uc_core::ports::BlobWriterPort;
use uc_core::{Blob, ContentHash};

// === Blob Writer ===

/// Placeholder blob writer port implementation
/// 占位符 blob 写入器端口实现
#[derive(Debug, Clone)]
pub struct PlaceholderBlobWriterPort;

#[async_trait]
impl BlobWriterPort for PlaceholderBlobWriterPort {
    async fn write_if_absent(
        &self,
        _content_id: &ContentHash,
        _encrypted_bytes: &[u8],
    ) -> Result<Blob> {
        // TODO: Implement actual blob writing
        // 实现实际的 blob 写入
        Err(anyhow::anyhow!("BlobWriterPort not implemented yet"))
    }
}
