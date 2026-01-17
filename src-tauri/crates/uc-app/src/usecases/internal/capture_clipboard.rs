use std::sync::Arc;
use std::time::SystemTime;

use anyhow::Result;
use futures::future::try_join_all;
use tracing::{debug, info, info_span, Instrument};

use uc_core::ids::{EntryId, EventId};
use uc_core::ports::{
    ClipboardEntryRepositoryPort, ClipboardEventWriterPort,
    ClipboardRepresentationMaterializerPort, DeviceIdentityPort, PlatformClipboardPort,
    SelectRepresentationPolicyPort,
};
use uc_core::{
    ClipboardEntry, ClipboardEvent, ClipboardSelectionDecision, SystemClipboardSnapshot,
};

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
///
/// # Architecture / 架构
///
/// This use case uses **trait objects** (`Arc<dyn Port>`) instead of generic type parameters.
/// This is the recommended pattern for use cases in the uc-app layer.
///
/// 此用例使用 **trait 对象** (`Arc<dyn Port>`) 而不是泛型类型参数。
/// 这是 uc-app 层用例的推荐模式。
pub struct CaptureClipboardUseCase {
    platform_clipboard_port: Arc<dyn PlatformClipboardPort>,
    entry_repo: Arc<dyn ClipboardEntryRepositoryPort>,
    event_writer: Arc<dyn ClipboardEventWriterPort>,
    representation_policy: Arc<dyn SelectRepresentationPolicyPort>,
    representation_materializer: Arc<dyn ClipboardRepresentationMaterializerPort>,
    device_identity: Arc<dyn DeviceIdentityPort>,
}

impl CaptureClipboardUseCase {
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
        platform_clipboard_port: Arc<dyn PlatformClipboardPort>,
        entry_repo: Arc<dyn ClipboardEntryRepositoryPort>,
        event_writer: Arc<dyn ClipboardEventWriterPort>,
        representation_policy: Arc<dyn SelectRepresentationPolicyPort>,
        representation_materializer: Arc<dyn ClipboardRepresentationMaterializerPort>,
        device_identity: Arc<dyn DeviceIdentityPort>,
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
        let span = info_span!(
            "usecase.capture_clipboard.execute",
            source = "platform_clipboard",
        );
        async {
            info!("Starting clipboard capture from platform");

            let snapshot = self.platform_clipboard_port.read_snapshot()?;

            debug!(
                representations = snapshot.representations.len(),
                "Captured system snapshot"
            );

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
                .map_err(|e| anyhow::anyhow!("Failed to get system time: {}", e))?
                .as_millis() as i64;
            let total_size = snapshot.total_size_bytes();

            let new_entry = ClipboardEntry::new(
                entry_id.clone(),
                event_id.clone(),
                created_at_ms,
                Self::generate_title(&snapshot),
                total_size,
            );
            self.entry_repo
                .save_entry_and_selection(&new_entry, &new_selection)
                .await?;

            info!(event_id = %event_id, "Clipboard capture completed");
            Ok(event_id)
        }
        .instrument(span)
        .await
    }

    /// Execute the clipboard capture workflow with a pre-captured snapshot.
    ///
    /// 执行剪贴板捕获工作流，使用预先捕获的快照。
    ///
    /// # Behavior / 行为
    /// - Uses the provided snapshot instead of reading from platform clipboard
    /// - Creates event and materializes all representations
    /// - Applies selection policy to determine optimal representation
    /// - Persists both event evidence and user-facing entry
    ///
    /// - 使用提供的快照而不是从平台剪贴板读取
    /// - 创建事件并物化所有表示形式
    /// - 应用选择策略确定最佳表示形式
    /// - 持久化事件证据和用户可见条目
    ///
    /// # Parameters / 参数
    /// - `snapshot`: Pre-captured clipboard snapshot from platform layer
    ///               来自平台层的预捕获剪贴板快照
    ///
    /// # Returns / 返回值
    /// - `EventId` of the created capture event
    /// - 创建的捕获事件的 `EventId`
    ///
    /// # When to Use / 使用时机
    /// - Called from clipboard change callback (snapshot already read)
    /// - 从剪贴板变化回调调用时（快照已读取）
    /// - Avoids redundant system clipboard reads
    /// - 避免重复读取系统剪贴板
    pub async fn execute_with_snapshot(
        &self,
        snapshot: SystemClipboardSnapshot,
    ) -> Result<EventId> {
        let span = info_span!(
            "usecase.capture_clipboard.execute",
            source = "callback",
            representations = snapshot.representations.len(),
        );
        async move {
            info!("Starting clipboard capture with provided snapshot");

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
                .map_err(|e| anyhow::anyhow!("Failed to get system time: {}", e))?
                .as_millis() as i64;
            let total_size = snapshot.total_size_bytes();

            let new_entry = ClipboardEntry::new(
                entry_id.clone(),
                event_id.clone(),
                created_at_ms,
                Self::generate_title(&snapshot),
                total_size,
            );
            self.entry_repo
                .save_entry_and_selection(&new_entry, &new_selection)
                .await?;

            info!(event_id = %event_id, "Clipboard capture completed");
            Ok(event_id)
        }
        .instrument(span)
        .await
    }

    /// Generate a title from the clipboard snapshot for display.
    /// 从剪贴板快照生成用于显示的标题。
    ///
    /// Tries to extract text content from text/plain representations,
    /// falling back to a size-based description if no text is found.
    ///
    /// 尝试从 text/plain 表示中提取文本内容，
    /// 如果没有找到文本，则回退到基于大小的描述。
    fn generate_title(snapshot: &SystemClipboardSnapshot) -> Option<String> {
        const MAX_TITLE_LENGTH: usize = 200;

        // Try to find text/plain representation
        // 尝试找到 text/plain 表示
        for rep in &snapshot.representations {
            if let Some(mime) = &rep.mime {
                let mime_str = mime.as_str();
                // Check for text MIME types (text/plain, public.utf8-plain-text, etc.)
                // 检查文本 MIME 类型
                if mime_str.eq_ignore_ascii_case("text/plain")
                    || mime_str.eq_ignore_ascii_case("public.utf8-plain-text")
                    || mime_str.eq_ignore_ascii_case("text/plain;charset=utf-8")
                    || mime_str.starts_with("text/")
                {
                    // Try to convert bytes to UTF-8 string
                    // 尝试将字节转换为 UTF-8 字符串
                    if let Ok(text) = std::str::from_utf8(&rep.bytes) {
                        let trimmed = text.trim();
                        if !trimmed.is_empty() {
                            // Truncate if too long and add ellipsis
                            // 如果太长则截断并添加省略号
                            // Use char_indices() to find a safe character boundary
                            // 使用 char_indices() 找到安全的字符边界
                            let char_count = trimmed.chars().count();
                            if char_count > MAX_TITLE_LENGTH {
                                let truncate_at = trimmed
                                    .char_indices()
                                    .nth(MAX_TITLE_LENGTH)
                                    .map(|(idx, _)| idx)
                                    .unwrap_or(trimmed.len());
                                let truncated = &trimmed[..truncate_at];
                                return Some(format!("{}...", truncated));
                            }
                            return Some(trimmed.to_string());
                        }
                    }
                }
            }
        }

        // Fallback: no text representation found
        // 回退：没有找到文本表示
        debug!("No text representation found in snapshot, title will be None");
        None
    }
}
