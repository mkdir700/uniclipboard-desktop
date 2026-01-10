use crate::models::MaterializedPayload;
use anyhow::{anyhow, Context, Ok, Result};
use uc_core::ids::EntryId;
use uc_core::ports::BlobMaterializerPort;
use uc_core::ports::{
    ClipboardEntryRepositoryPort, ClipboardEventRepositoryPort, ClockPort, ContentHashPort,
};
use uc_core::{ContentHash, SelectionState};

pub struct MaterializeClipboardSelectionUseCase<CER, CNR, B, CH>
where
    CER: ClipboardEventRepositoryPort,
    CNR: ClipboardEntryRepositoryPort,
    B: BlobMaterializerPort,
    CH: ContentHashPort,
{
    clipboard_event_repository: CER,
    clipboard_entry_repository: CNR,
    blob_materializer: B,
    hasher: CH,
}

impl<CER, CNR, B, CH> MaterializeClipboardSelectionUseCase<CER, CNR, B, CH>
where
    CER: ClipboardEventRepositoryPort,
    CNR: ClipboardEntryRepositoryPort,
    B: BlobMaterializerPort,
    CH: ContentHashPort,
{
    pub fn new(
        clipboard_event_repository: CER,
        clipboard_entry_repository: CNR,
        blob_materializer: B,
        hasher: CH,
    ) -> Self {
        Self {
            clipboard_event_repository,
            clipboard_entry_repository,
            blob_materializer,
            hasher,
        }
    }

    pub async fn execute(&self, entry_id: &EntryId) -> Result<MaterializedPayload> {
        let selection = self.clipboard_entry_repository.get_selection(entry_id)?;

        match selection.state {
            SelectionState::Inline { mime, bytes } => Ok(MaterializedPayload::Inline {
                mime: mime,
                bytes: bytes,
            }),

            SelectionState::MaterializedBlob { mime, blob_id } => Ok(MaterializedPayload::Blob {
                mime: mime,
                blob_id: blob_id,
            }),

            SelectionState::PendingBlob {
                representation_id,
                mime,
            } => {
                // 1) 找到 event_id & representation bytes
                // 1) Find event_id & representation bytes
                let event_id = self
                    .clipboard_entry_repository
                    .get_event_id_by_entry_id(entry_id)?;

                let rep = self
                    .clipboard_event_repository
                    .get_representation(event_id, &representation_id)
                    .await?;

                if rep.bytes.is_empty() {
                    return Err(anyhow!("representation bytes is empty")).with_context(|| {
                        format!(
                            "representation {} for event {} has empty bytes",
                            representation_id, event_id
                        )
                    });
                }

                let resolved_mime = mime.or(rep.mime);

                // 2) 阈值策略：小内容可以直接 inline
                // TODO: 未来支持
                // if rep.size_bytes >= 0 && rep.size_bytes <= self.inline_threshold_bytes {
                //     // 这里也可以选择“仍然 materialize blob”，但一般 inline 更省
                //     return Ok(MaterializedPayload::Inline {
                //         mime: mime.or(rep.mime),
                //         bytes: rep.bytes,
                //     });
                // }

                // 3) 计算 hash，用于查重
                let content_hash = self.hasher.hash_bytes(&rep.bytes)?;
                let materialized_blob = self
                    .blob_materializer
                    .materialize(&rep.bytes, content_hash)
                    .await?;

                // 6) 更新 selection 状态
                let blob_id = materialized_blob.id;
                self.clipboard_entry_repository.update_selection_to_blob(
                    entry_id,
                    blob_id,
                    resolved_mime.clone(),
                )?;

                Ok(MaterializedPayload::Blob {
                    mime: resolved_mime,
                    blob_id,
                })
            }
        }
    }
}
