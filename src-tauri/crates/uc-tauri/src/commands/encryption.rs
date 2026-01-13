//! Encryption-related Tauri commands
//! 加密相关的 Tauri 命令

use tauri::State;
use uc_app::AppDeps;

/// Initialize encryption with passphrase
/// 使用密码短语初始化加密
#[tauri::command]
pub async fn initialize_encryption(
    _deps: State<'_, AppDeps>,
    _passphrase: String,
) -> Result<(), String> {
    // TODO: Implement after InitializeEncryption use case is wired
    Err("Not yet implemented".to_string())
}

/// Check if encryption is initialized
/// 检查加密是否已初始化
#[tauri::command]
pub async fn is_encryption_initialized(
    _deps: State<'_, AppDeps>,
) -> Result<bool, String> {
    // TODO: Implement
    Err("Not yet implemented".to_string())
}
