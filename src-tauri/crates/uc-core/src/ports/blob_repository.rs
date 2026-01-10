// use crate::{command::NewBlobRecord, models::BlobRecord};
use anyhow::Result;
use async_trait::async_trait;

use crate::ContentHash;
use crate::Blob;

#[async_trait]
pub trait BlobRepositoryPort: Send + Sync {
    async fn insert_blob(&self, materialize_result: &Blob) -> Result<()>;
    async fn find_by_hash(&self, content_hash: &ContentHash) -> Result<Option<Blob>>;
}
