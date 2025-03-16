use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri_plugin_autostart::ManagerExt;

use crate::models::clipboard_record::DbClipboardRecord;
use crate::setting::Setting;
use crate::uni_clipboard::UniClipboard;

#[derive(Serialize, Deserialize)]
pub struct ClipboardRecordResponse {
    pub id: String,
    pub device_id: String,
    pub file_url: String,
    pub created_at: i32,
    pub updated_at: i32,
}

impl From<DbClipboardRecord> for ClipboardRecordResponse {
    fn from(record: DbClipboardRecord) -> Self {
        Self {
            id: record.id,
            device_id: record.device_id,
            file_url: record.file_url,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

// test func
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// 从前端获取配置的Tauri命令
#[tauri::command]
pub fn save_setting(setting_json: &str) -> Result<(), String> {
    match serde_json::from_str::<Setting>(setting_json) {
        Ok(setting) => {
            if let Err(e) = setting.save(None) {
                return Err(format!("保存设置失败: {}", e));
            }
            Ok(())
        },
        Err(e) => Err(format!("解析TOML设置失败: {}", e)),
    }
}

// 获取当前配置的Tauri命令
#[tauri::command]
pub fn get_setting() -> Result<String, String> {
    match Setting::load(None) {
        Ok(setting) => {
            match serde_json::to_string_pretty(&setting) {
                Ok(json_str) => Ok(json_str),
                Err(e) => Err(format!("序列化设置为JSON失败: {}", e)),
            }
        },
        Err(e) => Err(format!("加载设置失败: {}", e)),
    }
}

// 启用开机自启动
#[tauri::command]
pub async fn enable_autostart(app_handle: tauri::AppHandle) -> Result<(), String> {
    let autostart_manager = app_handle.autolaunch();
    let _ = autostart_manager.enable();
    Ok(())
}

// 禁用开机自启动
#[tauri::command]
pub async fn disable_autostart(app_handle: tauri::AppHandle) -> Result<(), String> {
    let autostart_manager = app_handle.autolaunch();
    let _ = autostart_manager.disable();
    Ok(())
}

// 检查是否已启用开机自启动
#[tauri::command]
pub async fn is_autostart_enabled(app_handle: tauri::AppHandle) -> Result<bool, String> {
    let autostart_manager = app_handle.autolaunch();
    autostart_manager.is_enabled().map_err(|e| e.to_string())
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

// 获取指定ID的剪贴板记录
#[tauri::command]
pub async fn get_clipboard_record_by_id(
    state: tauri::State<'_, Arc<Mutex<Option<Arc<UniClipboard>>>>>,
    id: String,
) -> Result<Option<ClipboardRecordResponse>, String> {
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
    match record_manager.get_record_by_id(&id).await {
        Ok(record) => Ok(record.map(ClipboardRecordResponse::from)),
        Err(e) => Err(format!("获取剪贴板记录失败: {}", e)),
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
    
    // 锁已释放，可以安全地使用await
    match record_manager.delete_record(&id).await {
        Ok(deleted) => Ok(deleted),
        Err(e) => Err(format!("删除剪贴板记录失败: {}", e)),
    }
}

// 清空所有剪贴板历史记录
#[tauri::command]
pub async fn clear_clipboard_records(
    state: tauri::State<'_, Arc<Mutex<Option<Arc<UniClipboard>>>>>,
) -> Result<usize, String> {
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
    match record_manager.clear_all_records().await {
        Ok(count) => Ok(count),
        Err(e) => Err(format!("清空剪贴板历史记录失败: {}", e)),
    }
}
