//! Clipboard-related Tauri commands
//! 剪贴板相关的 Tauri 命令

use tauri::State;
use crate::bootstrap::AppRuntime;
use crate::models::ClipboardEntryProjection;

/// Get clipboard history entries
/// 获取剪贴板历史条目
#[tauri::command]
pub async fn get_clipboard_entries(
    runtime: State<'_, AppRuntime>,
    limit: Option<usize>,
) -> Result<Vec<ClipboardEntryProjection>, String> {
    // Use UseCases accessor pattern (consistent with other commands)
    let uc = runtime.usecases().list_clipboard_entries();
    let limit = limit.unwrap_or(50);

    // Query entries through use case
    let entries = uc.execute(limit, 0)
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

/// Deletes a clipboard entry identified by `entry_id`.
///
/// This command converts the provided `entry_id` to the domain `EntryId` type and invokes the runtime's
/// delete clipboard-entry use case; on success it returns without value, otherwise it returns a stringified error.
///
/// # Examples
///
/// ```no_run
/// # async fn example(runtime: tauri::State<'_, uc_tauri::AppRuntime>) {
/// // Tauri provides `State<AppRuntime>` when invoking commands from the frontend.
/// let result = uc_tauri::commands::clipboard::delete_clipboard_entry(runtime, "entry-id-123".to_string()).await;
/// match result {
///     Ok(()) => println!("Deleted"),
///     Err(e) => eprintln!("Delete failed: {}", e),
/// }
/// # }
/// ```
#[tauri::command]
pub async fn delete_clipboard_entry(
    runtime: State<'_, AppRuntime>,
    entry_id: String,
) -> Result<(), String> {
    // Parse entry_id
    let entry_id = uc_core::ids::EntryId::from(entry_id);

    // Execute use case
    let use_case = runtime.usecases().delete_clipboard_entry();
    use_case.execute(&entry_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
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