//! Settings-related Tauri commands
//! 设置相关的 Tauri 命令

use serde_json::Value;
use tauri::State;
use uc_app::AppDeps;

/// Get application settings
/// 获取应用设置
#[tauri::command]
pub async fn get_settings(
    _deps: State<'_, AppDeps>,
) -> Result<Value, String> {
    // TODO: Implement
    Err("Not yet implemented".to_string())
}

/// Update application settings
/// 更新应用设置
#[tauri::command]
pub async fn update_settings(
    _deps: State<'_, AppDeps>,
    _settings: Value,
) -> Result<(), String> {
    // TODO: Implement
    Err("Not yet implemented".to_string())
}
