use anyhow::Result;
use async_trait::async_trait;

use super::meta::BlobMeta;

#[async_trait]
pub trait BlobStorePort: Send + Sync {
    async fn create(&self, meta: BlobMeta, data: Vec<u8>) -> Result<String>;
    async fn read_meta(&self, blob_id: &str) -> Result<BlobMeta>;
    async fn read_data(&self, blob_id: &str) -> Result<Vec<u8>>;
    async fn delete(&self, blob_id: &str) -> Result<()>;
}
#[cfg(test)]
mockall::mock! {
    pub BlobStore {}

    #[async_trait]
    impl BlobStorePort for BlobStore {
        async fn create(&self, meta: BlobMeta, data: Vec<u8>) -> Result<String>;
        async fn read_meta(&self, blob_id: &str) -> Result<BlobMeta>;
        async fn read_data(&self, blob_id: &str) -> Result<Vec<u8>>;
        async fn delete(&self, blob_id: &str) -> Result<()>;
    }
}
