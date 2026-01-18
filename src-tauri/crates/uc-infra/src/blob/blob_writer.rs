use anyhow::{Ok, Result};
use async_trait::async_trait;
use tracing::{debug, debug_span, Instrument};
use uc_core::blob::BlobStorageLocator;
use uc_core::ports::ClockPort;
use uc_core::ports::{BlobRepositoryPort, BlobStorePort, BlobWriterPort};
use uc_core::ContentHash;
use uc_core::{Blob, BlobId};

pub struct BlobWriter<B, BR, C>
where
    B: BlobStorePort,
    BR: BlobRepositoryPort,
    C: ClockPort,
{
    blob_store: B,
    blob_repo: BR,
    clock: C,
}

impl<B, BR, C> BlobWriter<B, BR, C>
where
    B: BlobStorePort,
    BR: BlobRepositoryPort,
    C: ClockPort,
{
    pub fn new(blob_store: B, blob_repo: BR, clock: C) -> Self {
        BlobWriter {
            blob_store,
            blob_repo,
            clock,
        }
    }
}

#[async_trait]
impl<B, BR, C> BlobWriterPort for BlobWriter<B, BR, C>
where
    B: BlobStorePort,
    BR: BlobRepositoryPort,
    C: ClockPort,
{
    async fn write_if_absent(
        &self,
        content_id: &ContentHash,
        encrypted_bytes: &[u8],
    ) -> Result<Blob> {
        let span = debug_span!(
            "infra.blob.write_if_absent",
            size_bytes = encrypted_bytes.len(),
            content_hash = %content_id,
        );
        async {
            if let Some(blob) = self.blob_repo.find_by_hash(content_id).await? {
                return Ok(blob);
            }

            let blob_id = BlobId::new();

            // TODO: Wire encryption before invoking this port; bytes are assumed encrypted here.
            let storage_path = self.blob_store.put(&blob_id, encrypted_bytes).await?;

            let created_at_ms = self.clock.now_ms();
            let blob_storage_locator = BlobStorageLocator::new_local_fs(storage_path);
            let result = Blob::new(
                blob_id,
                blob_storage_locator,
                encrypted_bytes.len() as i64,
                content_id.clone(),
                created_at_ms,
            );

            if let Err(err) = self.blob_repo.insert_blob(&result).await {
                if let Some(existing) = self.blob_repo.find_by_hash(content_id).await? {
                    debug!(
                        error = %err,
                        content_hash = %content_id,
                        "Insert raced with existing blob; returning existing record",
                    );
                    return Ok(existing);
                }
                return Err(err);
            }
            Ok(result)
        }
        .instrument(span)
        .await
    }
}
