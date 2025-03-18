use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::infrastructure::storage::db::models::clipboard_record::DbClipboardRecord;
use crate::core::UniClipboard;

#[derive(Serialize, Deserialize)]
pub struct ClipboardRecordResponse {
    pub id: String,
    pub device_id: String,
    pub content_type: String,
    pub is_downloaded: bool,
    pub is_favorited: bool,
    pub created_at: i32,
    pub updated_at: i32,
}

impl From<DbClipboardRecord> for ClipboardRecordResponse {
    fn from(record: DbClipboardRecord) -> Self {
        Self {
            id: record.id,
            device_id: record.device_id,
            content_type: record.content_type,
            is_downloaded: record.local_file_path.is_some(),
            is_favorited: record.is_favorited,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}


// 获取剪贴板历史记录
#[tauri::command]
pub async fn get_clipboard_records(
    state: tauri::State<'_, Arc<Mutex<Option<Arc<UniClipboard>>>>>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<ClipboardRecordResponse>, String> {
    // 在作用域内获取锁，确保在await前释放
    let record_manager = {
        let app = state.lock().unwrap();
        if let Some(app) = app.as_ref() {
            app.get_record_manager()
        } else {
            return Err("应用未初始化".to_string());
        }
    };
    
    // 锁已释放，可以安全地使用await
    match record_manager.get_records(limit, offset).await {
        Ok(records) => Ok(records.into_iter().map(ClipboardRecordResponse::from).collect()),
        Err(e) => Err(format!("获取剪贴板历史记录失败: {}", e)),
    }
}

// 删除指定ID的剪贴板记录
#[tauri::command]
pub async fn delete_clipboard_record(
    state: tauri::State<'_, Arc<Mutex<Option<Arc<UniClipboard>>>>>,
    id: String,
) -> Result<bool, String> {
    // 在作用域内获取锁，确保在await前释放
    let record_manager = {
        let app = state.lock().unwrap();
        if let Some(app) = app.as_ref() {
            app.get_record_manager()
        } else {
            return Err("应用未初始化".to_string());
        }
    };

    let record = record_manager.get_record_by_id(&id).await
        .map_err(|e| format!("获取剪贴板记录失败: {}", e))?;
    if record.is_none() {
        return Ok(true);
    }
    let record = record.unwrap();

    let file_storage = {
        let app = state.lock().unwrap();
        if let Some(app) = app.as_ref() {
            app.get_file_storage_manager()
        } else {
            return Err("应用未初始化".to_string());
        }
    };
    
    // 删除文件（如果存在）
    if let Some(path) = &record.local_file_path {
        match file_storage.delete(&Path::new(&path)).await {
            Ok(_) => {},
            Err(e) => {
                log::warn!("删除文件失败，但会继续删除记录: {}", e);
            }
        }
    }
    
    // 删除记录
    match record_manager.delete_record(&id).await {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("删除剪贴板记录失败: {}", e)),
    }
}

// 清空所有剪贴板历史记录
#[tauri::command]
pub async fn clear_clipboard_records(
    state: tauri::State<'_, Arc<Mutex<Option<Arc<UniClipboard>>>>>,
) -> Result<usize, String> {
    // 获取记录管理器和文件存储管理器
    let (record_manager, file_storage) = {
        let app = state.lock().unwrap();
        if let Some(app) = app.as_ref() {
            (app.get_record_manager(), app.get_file_storage_manager())
        } else {
            return Err("应用未初始化".to_string());
        }
    };
    
    // 先获取所有记录
    let records = match record_manager.get_all_records().await {
        Ok(records) => records,
        Err(e) => return Err(format!("获取剪贴板记录失败: {}", e)),
    };
    
    // 删除所有文件
    for record in &records {
        if let Some(path) = &record.local_file_path {
            match file_storage.delete(&Path::new(&path)).await {
                Ok(_) => {},
                Err(e) => {
                    log::warn!("删除文件失败: {}, 但会继续删除记录", e);
                }
            }
        }
    }
    
    // 清空所有记录
    match record_manager.clear_all_records().await {
        Ok(count) => Ok(count),
        Err(e) => Err(format!("清空剪贴板历史记录失败: {}", e)),
    }
}
