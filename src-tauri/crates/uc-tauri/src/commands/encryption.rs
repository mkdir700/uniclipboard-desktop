//! Encryption-related Tauri commands
//! 加密相关的 Tauri 命令

use std::sync::Arc;
use std::time::SystemTime;
use tauri::{AppHandle, Emitter, State};
use crate::bootstrap::AppRuntime;

/// Event payload for onboarding-password-set event
#[derive(Debug, Clone, serde::Serialize)]
struct OnboardingPasswordSetEvent {
    timestamp: u64,
}

/// Initialize encryption with passphrase
/// 使用密码短语初始化加密
///
/// This command uses the InitializeEncryption use case through the UseCases accessor.
/// 此命令通过 UseCases 访问器使用 InitializeEncryption 用例。
///
/// ## Architecture / 架构
///
/// - Commands layer (Driving Adapter) → UseCases accessor → Use Case → Ports
/// - No direct Port access from commands
/// - 命令层（驱动适配器）→ UseCases 访问器 → 用例 → 端口
/// - 命令不直接访问端口
#[tauri::command]
pub async fn initialize_encryption(
    runtime: State<'_, Arc<AppRuntime>>,
    app_handle: AppHandle,
    passphrase: String,
) -> Result<(), String> {
    eprintln!("[initialize_encryption] Command called with passphrase length: {}", passphrase.len());

    let uc = runtime.usecases().initialize_encryption();
    eprintln!("[initialize_encryption] Use case created, executing...");

    uc.execute(uc_core::security::model::Passphrase(passphrase))
        .await
        .map_err(|e| {
            eprintln!("[initialize_encryption] Use case execution failed: {:?}", e);
            e.to_string()
        })?;

    eprintln!("[initialize_encryption] Use case executed successfully, emitting event...");

    // Emit onboarding-password-set event for frontend
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| format!("Failed to get timestamp: {}", e))?
        .as_millis() as u64;

    let event = OnboardingPasswordSetEvent { timestamp };
    app_handle
        .emit("onboarding-password-set", event)
        .map_err(|e| format!("Failed to emit event: {}", e))?;

    eprintln!("[initialize_encryption] Event emitted successfully");
    log::info!("Onboarding: encryption password initialized successfully");
    Ok(())
}

/// Check if encryption is initialized
/// 检查加密是否已初始化
///
/// This command uses the IsEncryptionInitialized use case.
/// 此命令使用 IsEncryptionInitialized 用例。
#[tauri::command]
pub async fn is_encryption_initialized(
    runtime: State<'_, Arc<AppRuntime>>,
) -> Result<bool, String> {
    let uc = runtime.usecases().is_encryption_initialized();
    uc.execute().await.map_err(|e| e.to_string())
}
