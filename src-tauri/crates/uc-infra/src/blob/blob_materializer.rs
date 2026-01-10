use anyhow::{anyhow, Context, Ok, Result};
use uc_core::ports::{BlobMaterializerPort, BlobRepositoryPort, BlobStorePort};
use uc_core::ports::{ClockPort, ContentHashPort};
use uc_core::{BlobId, MaterializeResult};
use uc_core::{BlobStorageLocator, MaterializedPayload};
use uc_core::{ContentHash, SelectionState};

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

impl<B, BR, C> BlobMaterializerPort for BlobMaterializer<B, BR, C>
where
    B: BlobStorePort,
    BR: BlobRepositoryPort,
    C: ClockPort,
{
    async fn materialize(
        &self,
        data: &[u8],
        content_hash: &ContentHash,
    ) -> Result<MaterializeResult> {
        if let Some(blob) = self.blob_repo.find_by_hash(content_hash)? {
            // 复用已有 blob，并把 selection 更新为 materialized
            self.clipboard_entry_repository.update_selection_to_blob(
                entry_id,
                &blob.blob_id,
                resolved_mime,
            )?;
            return Ok(blob);
        }

        let blob_id = BlobId::new();

        // TODO: Implement encryption for blob data
        let storage_path = self.blob_store.put(&blob_id, &rep.bytes).await?;

        let create_at_ms = self.clock.now_ms();
        let blob_storage_locator = BlobStorageLocator::new_local_fs(storage_path);
        let result = MaterializeResult::new(
            blob_id,
            blob_storage_locator,
            rep.bytes.len() as i64,
            content_hash,
            created_at_ms,
        );

        self.blob_repo.insert_blob(materialize_result);
        Ok(result)
    }
}
