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
#[tauri::command]
pub async fn delete_clipboard_entry(
    _runtime: State<'_, AppRuntime>,
    _entry_id: String,
) -> Result<(), String> {
    // TODO: Implement after deletion use case is ready
    Err("Not yet implemented".to_string())
}

/// Capture current clipboard content
/// 捕获当前剪贴板内容
///
/// NOTE: This is a simplified version that directly uses AppDeps.
/// The full implementation with proper use case patterns will be added later.
///
/// 注意：这是简化版本，直接使用 AppDeps。
/// 完整的用例模式实现将在稍后添加。
#[tauri::command]
pub async fn capture_clipboard(
    _runtime: State<'_, AppRuntime>,
) -> Result<String, String> {
    // TODO: Implement full capture flow
    // This requires:
    // 1. Reading clipboard snapshot
    // 2. Materializing representations
    // 3. Creating event and entry
    // 4. Persisting to database
    //
    // For now, return a placeholder
    Err("Capture clipboard not yet implemented - requires ClipboardEventWriterPort in AppDeps".to_string())
}
