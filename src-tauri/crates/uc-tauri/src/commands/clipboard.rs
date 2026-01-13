//! Clipboard-related Tauri commands
//! 剪贴板相关的 Tauri 命令

use tauri::State;
use uc_app::AppDeps;

/// Get clipboard history entries
/// 获取剪贴板历史条目
#[tauri::command]
pub async fn get_clipboard_entries(
    _deps: State<'_, AppDeps>,
    _limit: Option<usize>,
) -> Result<Vec<crate::models::ClipboardEntryProjection>, String> {
    // For now, return empty list
    // TODO: Implement after use cases are wired
    Ok(vec![])
}

/// Delete a clipboard entry
/// 删除剪贴板条目
#[tauri::command]
pub async fn delete_clipboard_entry(
    _deps: State<'_, AppDeps>,
    _entry_id: String,
) -> Result<(), String> {
    // TODO: Implement
    Err("Not yet implemented".to_string())
}
