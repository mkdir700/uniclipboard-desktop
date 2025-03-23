use std::sync::{Arc, Mutex};

use crate::application::clipboard_service::{ClipboardItemResponse, ClipboardService};
use crate::core::UniClipboard;
use crate::infrastructure::storage::db::models::clipboard_record::OrderBy;

// 获取剪贴板历史记录
#[tauri::command]
pub async fn get_clipboard_items(
    state: tauri::State<'_, Arc<Mutex<Option<Arc<UniClipboard>>>>>,
    order_by: Option<OrderBy>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<ClipboardItemResponse>, String> {
    let app = state
        .lock()
        .unwrap()
        .as_ref()
        .ok_or("应用未初始化")?
        .clone();

    let service = ClipboardService::new(app);
    service
        .get_clipboard_items(order_by, limit, offset)
        .await
        .map_err(|e| format!("获取剪贴板历史记录失败: {}", e))
}

// 获取单个剪贴板项目
#[tauri::command]
pub async fn get_clipboard_item(
    state: tauri::State<'_, Arc<Mutex<Option<Arc<UniClipboard>>>>>,
    id: String,
    full_content: Option<bool>,
) -> Result<Option<ClipboardItemResponse>, String> {
    let app = state
        .lock()
        .unwrap()
        .as_ref()
        .ok_or("应用未初始化")?
        .clone();

    let service = ClipboardService::new(app);
    service
        .get_clipboard_item(&id, full_content.unwrap_or(false))
        .await
        .map_err(|e| format!("获取剪贴板记录失败: {}", e))
}

// 删除指定ID的剪贴板记录
#[tauri::command]
pub async fn delete_clipboard_item(
    state: tauri::State<'_, Arc<Mutex<Option<Arc<UniClipboard>>>>>,
    id: String,
) -> Result<bool, String> {
    let app = state
        .lock()
        .unwrap()
        .as_ref()
        .ok_or("应用未初始化")?
        .clone();

    let service = ClipboardService::new(app);
    service
        .delete_clipboard_item(&id)
        .await
        .map_err(|e| format!("删除剪贴板记录失败: {}", e))
}

// 清空所有剪贴板历史记录
#[tauri::command]
pub async fn clear_clipboard_items(
    state: tauri::State<'_, Arc<Mutex<Option<Arc<UniClipboard>>>>>,
) -> Result<usize, String> {
    let app = state
        .lock()
        .unwrap()
        .as_ref()
        .ok_or("应用未初始化")?
        .clone();

    let service = ClipboardService::new(app);
    service
        .clear_clipboard_items()
        .await
        .map_err(|e| format!("清空剪贴板历史记录失败: {}", e))
}

/// 复制剪贴板内容
#[tauri::command]
pub async fn copy_clipboard_item(
    state: tauri::State<'_, Arc<Mutex<Option<Arc<UniClipboard>>>>>,
    id: String,
) -> Result<bool, String> {
    let app = state
        .lock()
        .unwrap()
        .as_ref()
        .ok_or("应用未初始化")?
        .clone();

    let service = ClipboardService::new(app);
    service
        .copy_clipboard_item(&id)
        .await
        .map_err(|e| format!("复制剪贴板记录失败: {}", e))
}
