use crate::{Blob, ContentHash};
use anyhow::Result;

#[async_trait::async_trait]
pub trait BlobMaterializerPort {
    async fn materialize(&self, data: &[u8], content_hash: &ContentHash) -> Result<Blob>;
}
