//! Onboarding-related Tauri commands
//! 入门引导相关的 Tauri 命令

use crate::bootstrap::AppRuntime;
use crate::commands::record_trace_fields;
use std::sync::Arc;
use tauri::State;
use tracing::{info_span, Instrument};
use uc_app::usecases::OnboardingStateDto;
use uc_core::ports::observability::TraceMetadata;

/// Get current onboarding state
/// 获取当前入门引导状态
///
/// This command uses the GetOnboardingState use case.
/// 此命令使用 GetOnboardingState 用例。
#[tauri::command]
pub async fn get_onboarding_state(
    runtime: State<'_, Arc<AppRuntime>>,
    _trace: Option<TraceMetadata>,
) -> Result<OnboardingStateDto, String> {
    let span = info_span!(
        "command.onboarding.get_state",
        trace_id = tracing::field::Empty,
        trace_ts = tracing::field::Empty,
    );
    record_trace_fields(&span, &_trace);
    async {
        let uc = runtime.usecases().get_onboarding_state();
        uc.execute().await.map_err(|e| e.to_string())
    }
    .instrument(span)
    .await
}

/// Complete onboarding
/// 完成入门引导
///
/// This command uses the CompleteOnboarding use case.
/// 此命令使用 CompleteOnboarding 用例。
#[tauri::command]
pub async fn complete_onboarding(
    runtime: State<'_, Arc<AppRuntime>>,
    _trace: Option<TraceMetadata>,
) -> Result<(), String> {
    let span = info_span!(
        "command.onboarding.complete",
        trace_id = tracing::field::Empty,
        trace_ts = tracing::field::Empty,
    );
    record_trace_fields(&span, &_trace);
    async {
        let uc = runtime.usecases().complete_onboarding();
        uc.execute().await.map_err(|e| e.to_string())
    }
    .instrument(span)
    .await
}

/// Initialize onboarding and get initial state
/// 初始化入门引导并获取初始状态
///
/// This command uses the InitializeOnboarding use case.
/// 此命令使用 InitializeOnboarding 用例。
#[tauri::command]
pub async fn initialize_onboarding(
    runtime: State<'_, Arc<AppRuntime>>,
    _trace: Option<TraceMetadata>,
) -> Result<OnboardingStateDto, String> {
    let span = info_span!(
        "command.onboarding.initialize",
        trace_id = tracing::field::Empty,
        trace_ts = tracing::field::Empty,
    );
    record_trace_fields(&span, &_trace);
    async {
        let uc = runtime.usecases().initialize_onboarding();
        uc.execute().await.map_err(|e| e.to_string())
    }
    .instrument(span)
    .await
}
