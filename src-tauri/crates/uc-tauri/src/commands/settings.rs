//! Settings-related Tauri commands
//! 设置相关的 Tauri 命令

use serde_json::Value;
use tauri::State;
use uc_app::AppDeps;

/// Get application settings
/// 获取应用设置
///
/// **TODO**: Implement GetSettings use case
/// **TODO**: This command currently returns placeholder error
/// **NOTE**: Uses `State<'_, AppDeps>` instead of `State<'_, AppRuntime>` (needs refactoring)
/// **Tracking**: Requires use case implementation
///
/// ## Required Changes / 所需更改
///
/// 1. Change parameter from `State<'_, AppDeps>` to `State<'_, AppRuntime>`
/// 2. Create `GetSettings` use case in `uc-app/src/usecases/`
/// 3. Add `get_settings()` method to `UseCases` accessor
/// 4. Update this command to use `runtime.usecases().get_settings()`
///
/// ## Use Case Requirements / 用例需求
///
/// - Input: None (query operation)
/// - Output: `Settings` domain model → `Value` (JSON)
/// - Port: `SettingsPort` or `SettingsRepositoryPort` (needs definition)
/// - Error handling: Use `map_err` utility
///
/// ## Issue Tracking / 问题跟踪
///
/// - [ ] Define SettingsPort in uc-core/ports/
/// - [ ] Create use case: `uc-app/src/usecases/get_settings.rs`
/// - [ ] Change parameter to `State<'_, AppRuntime>`
/// - [ ] Add to UseCases accessor: `uc-tauri/src/bootstrap/runtime.rs`
/// - [ ] Update command implementation
#[tauri::command]
pub async fn get_settings(
    _deps: State<'_, AppDeps>,
) -> Result<Value, String> {
    // TODO: Implement GetSettings use case
    // This requires:
    // 1. Define SettingsPort in uc-core/ports/
    // 2. Create use case in uc-app/src/usecases/
    // 3. Change parameter to State<'_, AppRuntime>
    // 4. Add to UseCases accessor in uc-tauri/src/bootstrap/runtime.rs
    // 5. Update this command to use runtime.usecases().get_settings()
    Err("Not yet implemented - requires GetSettings use case".to_string())
}

/// Update application settings
/// 更新应用设置
///
/// **TODO**: Implement UpdateSettings use case
/// **TODO**: This command currently returns placeholder error
/// **NOTE**: Uses `State<'_, AppDeps>` instead of `State<'_, AppRuntime>` (needs refactoring)
/// **Tracking**: Requires use case implementation
///
/// ## Required Changes / 所需更改
///
/// 1. Change parameter from `State<'_, AppDeps>` to `State<'_, AppRuntime>`
/// 2. Create `UpdateSettings` use case in `uc-app/src/usecases/`
/// 3. Add `update_settings()` method to `UseCases` accessor
/// 4. Update this command to use `runtime.usecases().update_settings()`
///
/// ## Use Case Requirements / 用例需求
///
/// - Input: `settings: Value` (JSON from frontend)
/// - Conversion: `Value` → `Settings` domain model
/// - Port: `SettingsPort` or `SettingsRepositoryPort` (needs definition)
/// - Validation: Validate settings before persisting
/// - Error handling: Use `map_err` utility
///
/// ## Issue Tracking / 问题跟踪
///
/// - [ ] Define SettingsPort in uc-core/ports/
/// - [ ] Create use case: `uc-app/src/usecases/update_settings.rs`
/// - [ ] Change parameter to `State<'_, AppRuntime>`
/// - [ ] Add to UseCases accessor: `uc-tauri/src/bootstrap/runtime.rs`
/// - [ ] Update command implementation
#[tauri::command]
pub async fn update_settings(
    _deps: State<'_, AppDeps>,
    _settings: Value,
) -> Result<(), String> {
    // TODO: Implement UpdateSettings use case
    // This requires:
    // 1. Define SettingsPort in uc-core/ports/
    // 2. Create use case in uc-app/src/usecases/
    // 3. Change parameter to State<'_, AppRuntime>
    // 4. Add to UseCases accessor in uc-tauri/src/bootstrap/runtime.rs
    // 5. Update this command to use runtime.usecases().update_settings()
    Err("Not yet implemented - requires UpdateSettings use case".to_string())
}
