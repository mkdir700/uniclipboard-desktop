//! Onboarding-related Tauri commands
//! 入门引导相关的 Tauri 命令

use std::sync::Arc;
use tauri::State;
use crate::bootstrap::AppRuntime;
use uc_app::usecases::OnboardingStateDto;

/// Get current onboarding state
/// 获取当前入门引导状态
///
/// This command uses the GetOnboardingState use case.
/// 此命令使用 GetOnboardingState 用例。
#[tauri::command]
pub async fn get_onboarding_state(
    runtime: State<'_, Arc<AppRuntime>>,
) -> Result<OnboardingStateDto, String> {
    let uc = runtime.usecases().get_onboarding_state();
    uc.execute().await.map_err(|e| e.to_string())
}

/// Complete onboarding
/// 完成入门引导
///
/// This command uses the CompleteOnboarding use case.
/// 此命令使用 CompleteOnboarding 用例。
#[tauri::command]
pub async fn complete_onboarding(
    runtime: State<'_, Arc<AppRuntime>>,
) -> Result<(), String> {
    let uc = runtime.usecases().complete_onboarding();
    uc.execute().await.map_err(|e| e.to_string())
}

/// Initialize onboarding and get initial state
/// 初始化入门引导并获取初始状态
///
/// This command uses the InitializeOnboarding use case.
/// 此命令使用 InitializeOnboarding 用例。
#[tauri::command]
pub async fn initialize_onboarding(
    runtime: State<'_, Arc<AppRuntime>>,
) -> Result<OnboardingStateDto, String> {
    let uc = runtime.usecases().initialize_onboarding();
    uc.execute().await.map_err(|e| e.to_string())
}
