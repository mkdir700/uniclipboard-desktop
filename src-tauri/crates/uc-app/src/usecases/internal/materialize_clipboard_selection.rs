use crate::models::MaterializedPayload;
use anyhow::{Ok, Result};
use uc_core::ids::EntryId;
use uc_core::ports::{
    BlobMaterializerPort, ClipboardEntryRepositoryPort, ClipboardRepresentationRepositoryPort,
    ClipboardSelectionRepositoryPort, ContentHashPort,
};
use uc_core::PersistedClipboardRepresentation;

pub struct MaterializeClipboardSelectionUseCase<E, R, B, H, S>
where
    E: ClipboardEntryRepositoryPort,
    R: ClipboardRepresentationRepositoryPort,
    B: BlobMaterializerPort,
    H: ContentHashPort,
    S: ClipboardSelectionRepositoryPort,
{
    entry_repository: E,
    representation_repository: R,
    blob_materializer: B,
    hasher: H,
    selection_repository: S,
}

impl<E, R, B, H, S> MaterializeClipboardSelectionUseCase<E, R, B, H, S>
where
    E: ClipboardEntryRepositoryPort,
    R: ClipboardRepresentationRepositoryPort,
    B: BlobMaterializerPort,
    H: ContentHashPort,
    S: ClipboardSelectionRepositoryPort,
{
    pub fn new(
        entry_repository: E,
        representation_repository: R,
        blob_materializer: B,
        hasher: H,
        selection_repository: S,
    ) -> Self {
        Self {
            entry_repository,
            representation_repository,
            blob_materializer,
            hasher,
            selection_repository,
        }
    }

    pub async fn execute(&self, entry_id: &EntryId) -> Result<MaterializedPayload> {
        // 1️⃣ 读取 entry（事实）
        let entry = self
            .entry_repository
            .get_entry(entry_id)
            .await?
            .ok_or(anyhow::anyhow!("Entry {} has no event id", entry_id))?;
        let selection_decision = self
            .selection_repository
            .get_selection(entry_id)
            .await?
            .ok_or(anyhow::anyhow!(
                "Entry {} has no selection decision",
                entry_id
            ))?;

        // 2️⃣ 读取被选中的 representation（事实）
        let selected_representation_id = selection_decision.selection.primary_rep_id;
        let rep = self
            .representation_repository
            .get_representation(&entry.event_id, &selected_representation_id)
            .await?
            .ok_or(anyhow::anyhow!(
                "Representation {} not found for event {}",
                selected_representation_id,
                entry.event_id
            ))?;

        // 3️⃣ 基于事实推导状态（不是存储状态）
        if rep.is_inline() {
            return Ok(MaterializedPayload::Inline {
                mime: rep.mime_type,
                bytes: rep.inline_data.unwrap(),
            });
        }

        if let Some(blob_id) = rep.blob_id {
            return Ok(MaterializedPayload::Blob {
                mime: rep.mime_type,
                blob_id,
            });
        }

        // 4️⃣ 走到这里，唯一含义：
        //     "现在没有 blob，但可以 materialize"
        let raw_bytes = self.load_representation_bytes(&rep).await?;
        let content_hash = self.hasher.hash_bytes(&raw_bytes)?;

        let blob = self
            .blob_materializer
            .materialize(&raw_bytes, &content_hash)
            .await?;

        // 5️⃣ 更新事实（representation 现在有 blob 了）
        self.representation_repository
            .update_blob_id(&rep.id, &blob.blob_id)
            .await?;

        Ok(MaterializedPayload::Blob {
            mime: rep.mime_type,
            blob_id: blob.blob_id,
        })
    }

    async fn load_representation_bytes(
        &self,
        _rep: &PersistedClipboardRepresentation,
    ) -> Result<Vec<u8>> {
        unimplemented!()
    }
}
