use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;

use crate::BlobId;

#[async_trait]
pub trait BlobStorePort: Send + Sync {
    // 把 bytes 写入 blob 存储，返回 storage_path（或 key）
    async fn put(&self, blob_id: &BlobId, data: &[u8]) -> Result<PathBuf>;

    // 从 blob 存储读取 bytes
    async fn get(&self, blob_id: &BlobId) -> Result<Vec<u8>>;
}
