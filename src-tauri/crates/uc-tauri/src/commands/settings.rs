//! Settings-related Tauri commands
//! 设置相关的 Tauri 命令

use crate::bootstrap::AppRuntime;
use serde_json::Value;
use std::sync::Arc;
use tauri::State;
use tracing::{info_span, Instrument};
use uc_core::settings::model::Settings;

/// Get application settings
/// 获取应用设置
///
/// Returns the complete application settings as JSON.
///
/// ## Returns / 返回值
/// - JSON representation of current Settings
#[tauri::command]
pub async fn get_settings(runtime: State<'_, Arc<AppRuntime>>) -> Result<Value, String> {
    let span = info_span!(
        "command.settings.get",
        device_id = %runtime.deps.device_identity.current_device_id(),
    );
    async {
        let uc = runtime.usecases().get_settings();
        let settings = uc.execute().await.map_err(|e| {
            tracing::error!(error = %e, "Failed to get settings");
            e.to_string()
        })?;

        // Convert Settings to JSON value
        let json_value = serde_json::to_value(&settings).map_err(|e| {
            tracing::error!(error = %e, "Failed to serialize settings");
            format!("Failed to serialize settings: {}", e)
        })?;

        // DIAGNOSTIC: Log device_name in the JSON being sent to frontend
        tracing::info!(
            device_name = ?json_value.get("general").and_then(|g| g.get("device_name")),
            "Retrieved settings successfully"
        );
        Ok(json_value)
    }
    .instrument(span)
    .await
}

/// Update application settings
/// 更新应用设置
///
/// Updates application settings from JSON.
///
/// ## Parameters / 参数
/// - `settings`: JSON value containing settings to update
#[tauri::command]
pub async fn update_settings(
    runtime: State<'_, Arc<AppRuntime>>,
    settings: Value,
) -> Result<(), String> {
    let span = info_span!(
        "command.settings.update",
        device_id = %runtime.deps.device_identity.current_device_id(),
    );
    async {
        // Parse JSON into Settings domain model
        let parsed_settings: Settings = serde_json::from_value(settings.clone()).map_err(|e| {
            tracing::error!(error = %e, "Failed to parse settings JSON");
            format!("Failed to parse settings: {}", e)
        })?;

        let uc = runtime.usecases().update_settings();
        uc.execute(parsed_settings).await.map_err(|e| {
            tracing::error!(error = %e, "Failed to update settings");
            e.to_string()
        })?;

        tracing::info!("Settings updated successfully");
        Ok(())
    }
    .instrument(span)
    .await
}
