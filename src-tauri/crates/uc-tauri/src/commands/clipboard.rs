//! Clipboard-related Tauri commands
//! 剪贴板相关的 Tauri 命令

use tauri::State;
use crate::bootstrap::AppRuntime;
use uc_app::usecases::ListClipboardEntries;
use crate::models::ClipboardEntryProjection;

/// Get clipboard history entries
/// 获取剪贴板历史条目
#[tauri::command]
pub async fn get_clipboard_entries(
    runtime: State<'_, AppRuntime>,
    limit: Option<usize>,
) -> Result<Vec<ClipboardEntryProjection>, String> {
    let deps = &runtime.deps;
    // Create use case with repository from AppDeps (using from_arc for trait objects)
    let use_case = ListClipboardEntries::from_arc(deps.clipboard_entry_repo.clone());
    let limit = limit.unwrap_or(50);

    // Query entries through use case
    let entries = use_case
        .execute(limit, 0)
        .await
        .map_err(|e| e.to_string())?;

    // Convert domain models to DTOs
    let projections: Vec<ClipboardEntryProjection> = entries
        .into_iter()
        .map(|entry| ClipboardEntryProjection {
            id: entry.entry_id.to_string(),
            preview: entry.title.unwrap_or_else(|| format!("Entry ({} bytes)", entry.total_size)),
            captured_at: entry.created_at_ms,
            content_type: "clipboard".to_string(),
            is_encrypted: false, // TODO: Determine from actual entry state
        })
        .collect();

    Ok(projections)
}

/// Delete a clipboard entry
/// 删除剪贴板条目
///
/// **TODO**: Implement DeleteClipboardEntry use case
/// **TODO**: This command currently returns placeholder error
/// **Tracking**: Requires use case implementation before activation
///
/// ## Required Changes / 所需更改
///
/// 1. Create `DeleteClipboardEntry` use case in `uc-app/src/usecases/`
/// 2. Add `delete_clipboard_entry()` method to `UseCases` accessor
/// 3. Update this command to use `runtime.usecases().delete_clipboard_entry()`
///
/// ## Use Case Requirements / 用例需求
///
/// - Input: `entry_id: String` (frontend parameter)
/// - Domain model: `ClipboardEntryId` (from uc-core)
/// - Port: `ClipboardEntryRepositoryPort` (already exists)
/// - Error handling: Use `map_err` utility
///
/// ## Issue Tracking / 问题跟踪
///
/// - [ ] Create use case: `uc-app/src/usecases/delete_clipboard_entry.rs`
/// - [ ] Add to UseCases accessor: `uc-tauri/src/bootstrap/runtime.rs`
/// - [ ] Update command implementation
#[tauri::command]
pub async fn delete_clipboard_entry(
    _runtime: State<'_, AppRuntime>,
    _entry_id: String,
) -> Result<(), String> {
    // TODO: Implement DeleteClipboardEntry use case
    // This requires:
    // 1. Create use case in uc-app/src/usecases/
    // 2. Add to UseCases accessor in uc-tauri/src/bootstrap/runtime.rs
    // 3. Wire repository dependency
    // 4. Update this command to use runtime.usecases().delete_clipboard_entry()
    Err("Not yet implemented - requires DeleteClipboardEntry use case".to_string())
}

/// Capture current clipboard content
/// 捕获当前剪贴板内容
///
/// **TODO**: Implement CaptureClipboard use case
/// **TODO**: This command currently returns placeholder error
/// **Tracking**: Complex use case requiring multiple ports
///
/// ## Required Changes / 所需更改
///
/// 1. Create `CaptureClipboard` use case in `uc-app/src/usecases/`
/// 2. Add `capture_clipboard()` method to `UseCases` accessor
/// 3. Update this command to use `runtime.usecases().capture_clipboard()`
///
/// ## Use Case Requirements / 用例需求
///
/// This is a complex use case that orchestrates multiple ports:
///
/// 1. **ClipboardSnapshotPort** - Read current clipboard content
/// 2. **MaterializationPort** - Convert raw data to representations
/// 3. **ClipboardEventWriterPort** - Create and persist clipboard event
/// 4. **ClipboardEntryRepositoryPort** - Store entry in database
///
/// ## Architecture Flow / 架构流程
///
/// ```text
/// Frontend → Command → CaptureClipboard Use Case → Multiple Ports
///                                      ↓
///                    1. Snapshot (ClipboardSnapshotPort)
///                    2. Materialize (MaterializationPort)
///                    3. Create Event (ClipboardEventWriterPort)
///                    4. Persist (ClipboardEntryRepositoryPort)
/// ```
///
/// ## Issue Tracking / 问题跟踪
///
/// - [ ] Create use case: `uc-app/src/usecases/capture_clipboard.rs`
/// - [ ] Add ClipboardSnapshotPort to uc-core/ports/
/// - [ ] Add MaterializationPort to uc-core/ports/
/// - [ ] Add ClipboardEventWriterPort to uc-core/ports/
/// - [ ] Implement ports in uc-platform/ (clipboard adapters)
/// - [ ] Add to UseCases accessor: `uc-tauri/src/bootstrap/runtime.rs`
/// - [ ] Update command implementation
#[tauri::command]
pub async fn capture_clipboard(
    _runtime: State<'_, AppRuntime>,
) -> Result<String, String> {
    // TODO: Implement CaptureClipboard use case
    // This is a complex use case requiring:
    //
    // 1. Create use case in uc-app/src/usecases/
    // 2. Define required ports in uc-core/ports/:
    //    - ClipboardSnapshotPort (read clipboard)
    //    - MaterializationPort (convert to representations)
    //    - ClipboardEventWriterPort (create event)
    // 3. Implement ports in uc-platform/adapters/
    // 4. Add to UseCases accessor in uc-tauri/src/bootstrap/runtime.rs
    // 5. Wire all dependencies
    // 6. Update this command to use runtime.usecases().capture_clipboard()
    //
    // Tracking: Complex multi-port orchestration
    Err("Not yet implemented - requires CaptureClipboard use case with multiple ports".to_string())
}
