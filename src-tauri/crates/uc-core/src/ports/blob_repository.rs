// use crate::{command::NewBlobRecord, models::BlobRecord};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

use crate::Blob;
use crate::ContentHash;

#[async_trait]
pub trait BlobRepositoryPort: Send + Sync {
    async fn insert_blob(&self, blob: &Blob) -> Result<()>;
    async fn find_by_hash(&self, content_hash: &ContentHash) -> Result<Option<Blob>>;
}

#[async_trait]
impl<T: BlobRepositoryPort + ?Sized> BlobRepositoryPort for Arc<T> {
    async fn insert_blob(&self, blob: &Blob) -> Result<()> {
        (**self).insert_blob(blob).await
    }

    async fn find_by_hash(&self, content_hash: &ContentHash) -> Result<Option<Blob>> {
        (**self).find_by_hash(content_hash).await
    }
}
