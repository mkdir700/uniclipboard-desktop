use std::time::SystemTime;

use anyhow::Result;

use uc_core::ids::{EntryId, EventId};
use uc_core::ports::{BlobRepositoryPort, PlatformClipboardPort, SelectRepresentationPolicyPort};
use uc_core::ports::{
    ClipboardEntryWriterPort, ClipboardEventWriterPort, ClipboardRepresentationMaterializerPort,
};
use uc_core::{ClipboardEntry, ClipboardEvent, ClipboardSelectionDecision};

// 1. 从 platform 获取 Raw Snapshot（事实）
// 2. 生成 ClipboardEvent（时间点）
// 3. 保存 Snapshot + Representations（原始证据）
// 4. 调用 SelectRepresentationPolicy
// 5. 生成 ClipboardEntry（用户可见结果）
pub struct CaptureClipboardUseCase<P, CEW, CNW, B, S, R>
where
    P: PlatformClipboardPort,
    CEW: ClipboardEntryWriterPort,
    CNW: ClipboardEventWriterPort,
    B: BlobRepositoryPort,
    S: SelectRepresentationPolicyPort,
    R: ClipboardRepresentationMaterializerPort,
{
    platform_clipboard_port: P,
    entry_writer: CEW,
    event_writer: CNW,
    blob_record_repository: B,
    representation_policy: S,
    representation_materializer: R,
}

impl<P, CEW, CNW, B, S, R> CaptureClipboardUseCase<P, CEW, CNW, B, S, R>
where
    P: PlatformClipboardPort,
    CEW: ClipboardEntryWriterPort,
    CNW: ClipboardEventWriterPort,
    B: BlobRepositoryPort,
    S: SelectRepresentationPolicyPort,
    R: ClipboardRepresentationMaterializerPort,
{
    pub fn new(
        platform_clipboard_port: P,
        entry_writer: CEW,
        event_writer: CNW,
        blob_repository: B,
        representation_policy: S,
        representation_materializer: R,
    ) -> Self {
        Self {
            platform_clipboard_port,
            entry_writer,
            event_writer,
            blob_record_repository: blob_repository,
            representation_policy,
            representation_materializer,
        }
    }
}

impl<P, CEW, CNW, B, S, R> CaptureClipboardUseCase<P, CEW, CNW, B, S, R>
where
    P: PlatformClipboardPort,
    CEW: ClipboardEntryWriterPort,
    CNW: ClipboardEventWriterPort,
    B: BlobRepositoryPort,
    S: SelectRepresentationPolicyPort,
    R: ClipboardRepresentationMaterializerPort,
{
    pub async fn execute(&self) -> Result<String> {
        let snapshot = self.platform_clipboard_port.read_snapshot()?;

        let event_id = EventId::new();
        let captured_at_ms = snapshot.ts_ms;
        // TODO: 有一个 port 来获取 当前的 device id
        let source_device = "desktop".to_string();
        // TODO: 这个 hash 是否可唯一，可复现
        let snapshot_hash = snapshot.hash();

        // 1. 生成 event + snapshot representations
        let new_event = ClipboardEvent::new(event_id, captured_at_ms, source_device, snapshot_hash);

        // 3. event_repo.insert_event
        let materialized_reps = snapshot
            .representations
            .iter()
            .map(|rep| self.representation_materializer.materialize(&rep))
            .collect();
        self.event_writer
            .insert_event(&new_event, &materialized_reps)
            .await?;

        // 4. policy.select(snapshot)
        let entry_id = EntryId::new();
        let selection = self.representation_policy.select(&snapshot)?;
        let new_selection = ClipboardSelectionDecision::new(entry_id, selection);

        // 5. entry_repo.insert_entry
        let created_at_ms = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("System time before UNIX EPOCH")
            .as_millis() as i64;
        let total_size = snapshot.total_size_bytes();

        let new_entry = ClipboardEntry::new(
            entry_id,
            event_id,
            created_at_ms,
            None, // TODO: 暂时为 None
            total_size,
        );
        self.entry_writer.insert_entry(&new_entry, &new_selection);

        Ok(event_id)
    }
}
