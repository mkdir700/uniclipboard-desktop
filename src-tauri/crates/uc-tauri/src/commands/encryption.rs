//! Encryption-related Tauri commands
//! 加密相关的 Tauri 命令

use tauri::State;
use crate::bootstrap::AppRuntime;
use uc_core::security::state::EncryptionState;


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
    runtime: State<'_, AppRuntime>,
    passphrase: String,
) -> Result<(), String> {
    let uc = runtime.usecases().initialize_encryption();
    uc.execute(uc_core::security::model::Passphrase(passphrase))
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Check if encryption is initialized
/// 检查加密是否已初始化
#[tauri::command]
pub async fn is_encryption_initialized(
    runtime: State<'_, AppRuntime>,
) -> Result<bool, String> {
    let deps = &runtime.deps;
    let state = deps.encryption_state
        .load_state()
        .await
        .map_err(|e| format!("Failed to load encryption state: {}", e))?;

    Ok(state == EncryptionState::Initialized)
}
