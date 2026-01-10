use anyhow::Result;
use async_trait::async_trait;

use crate::clipboard::NewBlob;

use super::meta::BlobMeta;

#[async_trait]
pub trait BlobStorePort: Send + Sync {
    async fn create(&self, meta: BlobMeta, data: Vec<u8>) -> Result<String>;
    async fn read_meta(&self, blob_id: &str) -> Result<BlobMeta>;
    async fn read_data(&self, blob_id: &str) -> Result<Vec<u8>>;
    async fn exists(&self, blob_id: &str) -> Result<bool>;
    async fn delete(&self, blob_id: &str) -> Result<()>;
}

#[async_trait]
pub trait BlobRepositoryPort: Send + Sync {
    fn insert_blob(&self, new_blob: NewBlob) -> Result<()>;
}
