use tokio::sync::oneshot;

use crate::application::clipboard_service::ClipboardItemResponse;
use crate::infrastructure::runtime::{AppRuntimeHandle, ClipboardCommand};
use crate::infrastructure::storage::db::models::clipboard_record::{Filter, OrderBy};
use crate::infrastructure::storage::record_manager::ClipboardStats;

/// 获取剪切板记录的统计信息
/// 包含：
/// 1. 总条数
/// 2. 已占用的空间
#[tauri::command]
pub async fn get_clipboard_stats(
    state: tauri::State<'_, AppRuntimeHandle>,
) -> Result<ClipboardStats, String> {
    let (tx, rx) = oneshot::channel();
    state
        .clipboard_tx
        .send(ClipboardCommand::GetStats { respond_to: tx })
        .await
        .map_err(|e| format!("发送命令失败: {}", e))?;

    rx.await.map_err(|e| format!("接收响应失败: {}", e))?
}

// 获取剪贴板历史记录
#[tauri::command]
pub async fn get_clipboard_items(
    state: tauri::State<'_, AppRuntimeHandle>,
    order_by: Option<OrderBy>,
    limit: Option<i64>,
    offset: Option<i64>,
    filter: Option<Filter>,
) -> Result<Vec<ClipboardItemResponse>, String> {
    let (tx, rx) = oneshot::channel();
    state
        .clipboard_tx
        .send(ClipboardCommand::GetItems {
            order_by,
            limit,
            offset,
            filter,
            respond_to: tx,
        })
        .await
        .map_err(|e| format!("发送命令失败: {}", e))?;

    rx.await.map_err(|e| format!("接收响应失败: {}", e))?
}

// 获取单个剪贴板项目
#[tauri::command]
pub async fn get_clipboard_item(
    state: tauri::State<'_, AppRuntimeHandle>,
    id: String,
    full_content: Option<bool>,
) -> Result<Option<ClipboardItemResponse>, String> {
    let (tx, rx) = oneshot::channel();
    state
        .clipboard_tx
        .send(ClipboardCommand::GetItem {
            id,
            full_content: full_content.unwrap_or(false),
            respond_to: tx,
        })
        .await
        .map_err(|e| format!("发送命令失败: {}", e))?;

    rx.await.map_err(|e| format!("接收响应失败: {}", e))?
}

// 删除指定ID的剪贴板记录
#[tauri::command]
pub async fn delete_clipboard_item(
    state: tauri::State<'_, AppRuntimeHandle>,
    id: String,
) -> Result<bool, String> {
    let (tx, rx) = oneshot::channel();
    state
        .clipboard_tx
        .send(ClipboardCommand::DeleteItem { id, respond_to: tx })
        .await
        .map_err(|e| format!("发送命令失败: {}", e))?;

    rx.await.map_err(|e| format!("接收响应失败: {}", e))?
}

// 清空所有剪贴板历史记录
#[tauri::command]
pub async fn clear_clipboard_items(
    state: tauri::State<'_, AppRuntimeHandle>,
) -> Result<usize, String> {
    let (tx, rx) = oneshot::channel();
    state
        .clipboard_tx
        .send(ClipboardCommand::ClearItems { respond_to: tx })
        .await
        .map_err(|e| format!("发送命令失败: {}", e))?;

    rx.await.map_err(|e| format!("接收响应失败: {}", e))?
}

/// 复制剪贴板内容
#[tauri::command]
pub async fn copy_clipboard_item(
    state: tauri::State<'_, AppRuntimeHandle>,
    id: String,
) -> Result<bool, String> {
    let (tx, rx) = oneshot::channel();
    state
        .clipboard_tx
        .send(ClipboardCommand::CopyItem { id, respond_to: tx })
        .await
        .map_err(|e| format!("发送命令失败: {}", e))?;

    rx.await.map_err(|e| format!("接收响应失败: {}", e))?
}

/// 收藏剪贴板内容
#[tauri::command]
pub async fn toggle_favorite_clipboard_item(
    state: tauri::State<'_, AppRuntimeHandle>,
    id: String,
    is_favorited: bool,
) -> Result<bool, String> {
    let (tx, rx) = oneshot::channel();
    state
        .clipboard_tx
        .send(ClipboardCommand::ToggleFavorite {
            id,
            is_favorited,
            respond_to: tx,
        })
        .await
        .map_err(|e| format!("发送命令失败: {}", e))?;

    rx.await.map_err(|e| format!("接收响应失败: {}", e))?
}
