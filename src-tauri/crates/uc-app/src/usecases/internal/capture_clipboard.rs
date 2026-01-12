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

/// Capture clipboard content and create persistent entries.
///
/// 捕获剪贴板内容并创建持久化条目。
///
/// # Behavior / 行为
/// - 1. Capture raw snapshot from platform clipboard (事实)
/// - 2. Generate ClipboardEvent with timestamp (时间点)
/// - 3. Persist snapshot and representations (原始证据)
/// - 4. Apply representation selection policy (策略决策)
/// - 5. Create ClipboardEntry for user consumption (用户可见结果)
///
/// - 1. 从平台剪贴板获取原始快照（事实）
/// - 2. 生成带时间戳的剪贴板事件（时间点）
/// - 3. 持久化快照和表示形式（原始证据）
/// - 4. 应用表示形式选择策略（策略决策）
/// - 5. 为用户消费创建剪贴板条目（用户可见结果）
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
    /// Create a new CaptureClipboardUseCase with all required dependencies.
    ///
    /// 创建包含所有必需依赖项的新 CaptureClipboardUseCase 实例。
    ///
    /// # Parameters / 参数
    /// - `platform_clipboard_port`: Platform clipboard access
    /// - `entry_repo`: Clipboard entry persistence
    /// - `event_writer`: Event and representation storage
    /// - `representation_policy`: Selection strategy for optimal representation
    /// - `representation_materializer`: Binary data materialization
    /// - `device_identity`: Current device identification
    ///
    /// - `platform_clipboard_port`: 平台剪贴板访问
    /// - `entry_repo`: 剪贴板条目持久化
    /// - `event_writer`: 事件和表示形式存储
    /// - `representation_policy`: 最佳表示形式的选择策略
    /// - `representation_materializer`: 二进制数据物化
    /// - `device_identity`: 当前设备标识
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
    /// Execute the clipboard capture workflow.
    ///
    /// 执行剪贴板捕获工作流。
    ///
    /// # Behavior / 行为
    /// - Captures current clipboard state from platform
    /// - Creates event and materializes all representations
    /// - Applies selection policy to determine optimal representation
    /// - Persists both event evidence and user-facing entry
    ///
    /// - 从平台捕获当前剪贴板状态
    /// - 创建事件并物化所有表示形式
    /// - 应用选择策略确定最佳表示形式
    /// - 持久化事件证据和用户可见条目
    ///
    /// # Returns / 返回值
    /// - `EventId` of the created capture event
    /// - 创建的捕获事件的 `EventId`
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
