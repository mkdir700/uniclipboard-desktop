//! Placeholder blob materializer and store port implementations
//! 占位符 blob 物化器和存储端口实现

use uc_core::ports::{BlobMaterializerPort, BlobStorePort};
use uc_core::{Blob, ContentHash, BlobId};
use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;

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

// === Blob Store ===

#[async_trait]
impl BlobStorePort for PlaceholderBlobStorePort {
    async fn put(&self, _blob_id: &BlobId, _data: &[u8]) -> Result<PathBuf> {
        // TODO: Implement actual blob storage
        // 实现实际的 blob 存储
        Err(anyhow::anyhow!("BlobStorePort::put not implemented yet"))
    }

    async fn get(&self, _blob_id: &BlobId) -> Result<Vec<u8>> {
        // TODO: Implement actual blob retrieval
        // 实现实际的 blob 检索
        Err(anyhow::anyhow!("BlobStorePort::get not implemented yet"))
    }
}

/// Placeholder blob store port implementation
/// 占位符 blob 存储端口实现
#[derive(Debug, Clone)]
pub struct PlaceholderBlobStorePort;
