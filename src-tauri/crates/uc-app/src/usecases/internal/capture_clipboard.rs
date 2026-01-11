use std::time::SystemTime;

use anyhow::Result;
use futures::future::try_join_all;

use uc_core::ids::{EntryId, EventId};
use uc_core::ports::{
    ClipboardEntryRepositoryPort, ClipboardEventWriterPort,
    ClipboardRepresentationMaterializerPort, DeviceIdentityPort, PlatformClipboardPort,
    SelectRepresentationPolicyPort,
};
use uc_core::{ClipboardEntry, ClipboardEvent, ClipboardSelectionDecision};

// 1. 从 platform 获取 Raw Snapshot（事实）
// 2. 生成 ClipboardEvent（时间点）
// 3. 保存 Snapshot + Representations（原始证据）
// 4. 调用 SelectRepresentationPolicy
// 5. 生成 ClipboardEntry（用户可见结果）
pub struct CaptureClipboardUseCase<P, CEW, CNW, S, R, D>
where
    P: PlatformClipboardPort,
    CEW: ClipboardEntryRepositoryPort,
    CNW: ClipboardEventWriterPort,
    S: SelectRepresentationPolicyPort,
    R: ClipboardRepresentationMaterializerPort,
    D: DeviceIdentityPort,
{
    platform_clipboard_port: P,
    entry_repo: CEW,
    event_writer: CNW,
    representation_policy: S,
    representation_materializer: R,
    device_identity: D,
}

impl<P, CEW, CNW, S, R, D> CaptureClipboardUseCase<P, CEW, CNW, S, R, D>
where
    P: PlatformClipboardPort,
    CEW: ClipboardEntryRepositoryPort,
    CNW: ClipboardEventWriterPort,
    S: SelectRepresentationPolicyPort,
    R: ClipboardRepresentationMaterializerPort,
    D: DeviceIdentityPort,
{
    pub fn new(
        platform_clipboard_port: P,
        entry_repo: CEW,
        event_writer: CNW,
        representation_policy: S,
        representation_materializer: R,
        device_identity: D,
    ) -> Self {
        Self {
            platform_clipboard_port,
            entry_repo,
            event_writer,
            representation_policy,
            representation_materializer,
            device_identity,
        }
    }
}

impl<P, CEW, CNW, S, R, D> CaptureClipboardUseCase<P, CEW, CNW, S, R, D>
where
    P: PlatformClipboardPort,
    CEW: ClipboardEntryRepositoryPort,
    CNW: ClipboardEventWriterPort,
    S: SelectRepresentationPolicyPort,
    R: ClipboardRepresentationMaterializerPort,
    D: DeviceIdentityPort,
{
    pub async fn execute(&self) -> Result<EventId> {
        let snapshot = self.platform_clipboard_port.read_snapshot()?;

        let event_id = EventId::new();
        let captured_at_ms = snapshot.ts_ms;
        let source_device = self.device_identity.current_device_id();
        let snapshot_hash = snapshot.snapshot_hash();

        // 1. 生成 event + snapshot representations
        let new_event = ClipboardEvent::new(
            event_id.clone(),
            captured_at_ms,
            source_device,
            snapshot_hash,
        );

        // 3. event_repo.insert_event
        let materialized_futures: Vec<_> = snapshot
            .representations
            .iter()
            .map(|rep| self.representation_materializer.materialize(rep))
            .collect();
        let materialized_reps = try_join_all(materialized_futures).await?;
        self.event_writer
            .insert_event(&new_event, &materialized_reps)
            .await?;

        // 4. policy.select(snapshot)
        let entry_id = EntryId::new();
        let selection = self.representation_policy.select(&snapshot)?;
        let new_selection = ClipboardSelectionDecision::new(entry_id.clone(), selection);

        // 5. entry_repo.insert_entry
        let created_at_ms = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("System time before UNIX EPOCH")
            .as_millis() as i64;
        let total_size = snapshot.total_size_bytes();

        let new_entry = ClipboardEntry::new(
            entry_id.clone(),
            event_id.clone(),
            created_at_ms,
            None, // TODO: 暂时为 None
            total_size,
        );
        let _ = self
            .entry_repo
            .save_entry_and_selection(&new_entry, &new_selection);

        Ok(event_id)
    }
}
