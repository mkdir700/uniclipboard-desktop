use anyhow::{Ok, Result};
use async_trait::async_trait;
use tracing::debug_span;
use uc_core::blob::BlobStorageLocator;
use uc_core::ports::ClockPort;
use uc_core::ports::{BlobMaterializerPort, BlobRepositoryPort, BlobStorePort};
use uc_core::ContentHash;
use uc_core::{Blob, BlobId};

pub struct BlobMaterializer<B, BR, C>
where
    B: BlobStorePort,
    BR: BlobRepositoryPort,
    C: ClockPort,
{
    blob_store: B,
    blob_repo: BR,
    clock: C,
}

impl<B, BR, C> BlobMaterializer<B, BR, C>
where
    B: BlobStorePort,
    BR: BlobRepositoryPort,
    C: ClockPort,
{
    pub fn new(blob_store: B, blob_repo: BR, clock: C) -> Self {
        BlobMaterializer {
            blob_store,
            blob_repo,
            clock,
        }
    }
}

#[async_trait]
impl<B, BR, C> BlobMaterializerPort for BlobMaterializer<B, BR, C>
where
    B: BlobStorePort,
    BR: BlobRepositoryPort,
    C: ClockPort,
{
    async fn materialize(&self, data: &[u8], content_hash: &ContentHash) -> Result<Blob> {
        let span = debug_span!(
            "infra.blob.materialize",
            size_bytes = data.len(),
            content_hash = %content_hash,
        );
        let _enter = span.enter();

        if let Some(blob) = self.blob_repo.find_by_hash(content_hash).await? {
            return Ok(blob);
        }

        let blob_id = BlobId::new();

        // TODO: Implement encryption for blob data
        let storage_path = self.blob_store.put(&blob_id, data).await?;

        let created_at_ms = self.clock.now_ms();
        let blob_storage_locator = BlobStorageLocator::new_local_fs(storage_path);
        let result = Blob::new(
            blob_id,
            blob_storage_locator,
            data.len() as i64,
            content_hash.clone(),
            created_at_ms,
        );

        self.blob_repo.insert_blob(&result).await?;
        Ok(result)
    }
}
