//! Settings-related Tauri commands
//! 设置相关的 Tauri 命令

use serde_json::Value;
use tauri::State;
use crate::bootstrap::AppRuntime;

/// Get application settings
/// 获取应用设置
#[tauri::command]
pub async fn get_settings(
    runtime: State<'_, AppRuntime>,
) -> Result<Value, String> {
    let uc = runtime.usecases().get_settings();
    let settings = uc.execute().await.map_err(|e| e.to_string())?;

    // Convert Settings to JSON value
    serde_json::to_value(&settings).map_err(|e| format!("Failed to serialize settings: {}", e))
}

/// Update application settings
/// 更新应用设置
#[tauri::command]
pub async fn update_settings(
    runtime: State<'_, AppRuntime>,
    settings: Value,
) -> Result<(), String> {
    use uc_core::settings::model::Settings;

    // Parse JSON into Settings domain model
    let settings: Settings = serde_json::from_value(settings)
        .map_err(|e| format!("Failed to parse settings: {}", e))?;

    let uc = runtime.usecases().update_settings();
    uc.execute(settings).await.map_err(|e| e.to_string())
}
