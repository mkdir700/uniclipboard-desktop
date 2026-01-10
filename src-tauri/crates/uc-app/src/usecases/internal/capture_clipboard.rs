use std::time::SystemTime;

use anyhow::Result;
use uc_core::BlobId;
use uuid::Uuid;

use uc_core::ports::{BlobRepositoryPort, PlatformClipboardPort, SelectRepresentationPolicyPort};
use uc_core::ports::{ClipboardEntryWriterPort, ClipboardEventWriterPort};

use crate::command::{EntryId, EventId};
use crate::command::{NewBlobRecord, NewClipboardEntry, NewClipboardEvent, NewClipboardSelection};

// 1. 从 platform 获取 Raw Snapshot（事实）
// 2. 生成 ClipboardEvent（时间点）
// 3. 保存 Snapshot + Representations（原始证据）
// 4. 调用 SelectRepresentationPolicy
// 5. 生成 ClipboardEntry（用户可见结果）
pub struct CaptureClipboardUseCase<P, CEW, CNW, B, S>
where
    P: PlatformClipboardPort,
    CEW: ClipboardEntryWriterPort,
    CNW: ClipboardEventWriterPort,
    B: BlobRepositoryPort,
    S: SelectRepresentationPolicyPort,
{
    platform_clipboard_port: P,
    entry_writer: CEW,
    event_writer: CNW,
    blob_record_repository: B,
    representation_policy: S,
}

impl<P, CEW, CNW, B, S> CaptureClipboardUseCase<P, CEW, CNW, B, S>
where
    P: PlatformClipboardPort,
    CEW: ClipboardEntryWriterPort,
    CNW: ClipboardEventWriterPort,
    B: BlobRepositoryPort,
    S: SelectRepresentationPolicyPort,
{
    pub fn new(
        platform_clipboard_port: P,
        entry_writer: CEW,
        event_writer: CNW,
        blob_repository: B,
        representation_policy: S,
    ) -> Self {
        Self {
            platform_clipboard_port,
            entry_writer,
            event_writer,
            blob_record_repository: blob_repository,
            representation_policy,
        }
    }
}

impl<P, CEW, CNW, B, S> CaptureClipboardUseCase<P, CEW, CNW, B, S>
where
    P: PlatformClipboardPort,
    CEW: ClipboardEntryWriterPort,
    CNW: ClipboardEventWriterPort,
    B: BlobRepositoryPort,
    S: SelectRepresentationPolicyPort,
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
        let new_event =
            NewClipboardEvent::new(event_id, captured_at_ms, source_device, snapshot_hash);

        // 3. event_repo.insert_event
        self.event_writer
            .insert_event(&new_event, &snapshot.representations);

        // 4. policy.select(snapshot)
        let entry_id = EntryId::new();
        let selection = self.representation_policy.select(&snapshot)?;
        let new_selection = NewClipboardSelection::new(entry_id, selection);

        // 5. entry_repo.insert_entry
        let created_at_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before UNIX EPOCH")
            .as_millis() as i64;
        let total_size = snapshot.total_size_bytes();

        let new_entry = NewClipboardEntry::new(
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
