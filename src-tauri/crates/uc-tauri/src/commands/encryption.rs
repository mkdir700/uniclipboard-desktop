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
///
/// **TODO**: Implement IsEncryptionInitialized use case
/// **TODO**: This command currently accesses Port directly (architecture violation)
/// **Tracking**: Needs refactoring to use UseCases accessor pattern
///
/// ## Required Changes / 所需更改
///
/// 1. Create `IsEncryptionInitialized` use case in `uc-app/src/usecases/`
/// 2. Add `is_encryption_initialized()` method to `UseCases` accessor
/// 3. Update this command to use `runtime.usecases().is_encryption_initialized()`
///
/// ## Issue Tracking / 问题跟踪
///
/// - [ ] Create use case: `uc-app/src/usecases/is_encryption_initialized.rs`
/// - [ ] Add to UseCases accessor: `uc-tauri/src/bootstrap/runtime.rs`
/// - [ ] Update command implementation
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
