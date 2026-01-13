//! Encryption-related Tauri commands
//! 加密相关的 Tauri 命令

use tauri::State;
use uc_app::AppDeps;
use uc_core::security::model::Passphrase;
use uc_core::security::state::EncryptionState;

/// Initialize encryption with passphrase
/// 使用密码短语初始化加密
///
/// NOTE: This is a simplified implementation. The full use case with
/// InitializeEncryption will be implemented when the remaining ports
/// are available.
///
/// 注意：这是简化实现。完整的 InitializeEncryption 用例将在
/// 其他端口可用时实现。
#[tauri::command]
pub async fn initialize_encryption(
    deps: State<'_, AppDeps>,
    passphrase: String,
) -> Result<(), String> {
    use uc_core::security::model::{KeySlot, WrappedMasterKey};
    use uc_core::security::model::{EncryptionAlgo, MasterKey};

    // 1. Check if already initialized
    let state = deps.encryption_state
        .load_state()
        .await
        .map_err(|e| format!("Failed to load encryption state: {}", e))?;

    if state == EncryptionState::Initialized {
        return Err("Encryption is already initialized".to_string());
    }

    // 2. Get current scope and create keyslot draft
    let scope = deps.key_scope
        .current_scope()
        .await
        .map_err(|e| format!("Failed to get key scope: {}", e))?;

    let keyslot_draft = KeySlot::draft_v1(scope.clone())
        .map_err(|e| format!("Failed to create keyslot draft: {}", e))?;

    // 3. Derive KEK from passphrase
    let kek = deps.encryption
        .derive_kek(&Passphrase(passphrase), &keyslot_draft.salt, &keyslot_draft.kdf)
        .await
        .map_err(|e| format!("Failed to derive KEK: {}", e))?;

    // 4. Generate Master Key
    let master_key = MasterKey::generate()
        .map_err(|e| format!("Failed to generate master key: {}", e))?;

    // 5. Wrap Master Key
    let blob = deps.encryption
        .wrap_master_key(&kek, &master_key, EncryptionAlgo::XChaCha20Poly1305)
        .await
        .map_err(|e| format!("Failed to wrap master key: {}", e))?;

    let keyslot = keyslot_draft.finalize(WrappedMasterKey { blob });

    // 6. Store keyslot
    deps.key_material
        .store_keyslot(&keyslot)
        .await
        .map_err(|e| format!("Failed to store keyslot: {}", e))?;

    // 7. Store KEK in keyring
    deps.key_material
        .store_kek(&scope, &kek)
        .await
        .map_err(|e| format!("Failed to store KEK: {}", e))?;

    // 8. Persist initialized state
    deps.encryption_state
        .persist_initialized()
        .await
        .map_err(|e| format!("Failed to persist encryption state: {}", e))?;

    Ok(())
}

/// Check if encryption is initialized
/// 检查加密是否已初始化
#[tauri::command]
pub async fn is_encryption_initialized(
    deps: State<'_, AppDeps>,
) -> Result<bool, String> {
    let state = deps.encryption_state
        .load_state()
        .await
        .map_err(|e| format!("Failed to load encryption state: {}", e))?;

    Ok(state == EncryptionState::Initialized)
}
