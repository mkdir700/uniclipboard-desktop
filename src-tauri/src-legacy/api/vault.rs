use crate::infrastructure::security::password::PasswordManager;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct VaultStatus {
    pub is_initialized: bool,
}

/// 检查 vault 状态
#[tauri::command]
pub fn check_vault_status() -> Result<VaultStatus, String> {
    let is_initialized = PasswordManager::vault_key_exists();
    Ok(VaultStatus { is_initialized })
}

/// 重置 vault（删除所有加密数据）
#[tauri::command]
pub fn reset_vault() -> Result<(), String> {
    // 删除 vault 密钥文件
    let vault_key_path = PasswordManager::get_vault_key_path();
    if vault_key_path.exists() {
        std::fs::remove_file(vault_key_path)
            .map_err(|e| format!("Failed to remove vault key: {}", e))?;
    }

    // 删除 stronghold 数据文件
    let stronghold_path = PasswordManager::get_snapshot_path();
    if stronghold_path.exists() {
        std::fs::remove_file(stronghold_path)
            .map_err(|e| format!("Failed to remove vault data: {}", e))?;
    }

    Ok(())
}
