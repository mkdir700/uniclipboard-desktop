//! Setup-related Tauri commands
//! 设置流程相关的 Tauri 命令

use crate::bootstrap::AppRuntime;
use crate::commands::record_trace_fields;
use std::sync::Arc;
use tauri::State;
use tracing::{info_span, Instrument};
use uc_core::ports::observability::TraceMetadata;
use uc_core::setup::{SetupEvent, SetupState};

/// Get current setup state.
/// 获取当前设置流程状态。
#[tauri::command]
pub async fn get_setup_state(
    runtime: State<'_, Arc<AppRuntime>>,
    _trace: Option<TraceMetadata>,
) -> Result<SetupState, String> {
    let span = info_span!(
        "command.setup.get_state",
        trace_id = tracing::field::Empty,
        trace_ts = tracing::field::Empty,
    );
    record_trace_fields(&span, &_trace);
    async {
        let orchestrator = runtime.usecases().setup_orchestrator();
        Ok(orchestrator.get_state().await)
    }
    .instrument(span)
    .await
}

/// Dispatch a setup event and return the next state.
/// 分发设置事件并返回下一状态。
#[tauri::command]
pub async fn dispatch_setup_event(
    runtime: State<'_, Arc<AppRuntime>>,
    event: SetupEvent,
    _trace: Option<TraceMetadata>,
) -> Result<SetupState, String> {
    let span = info_span!(
        "command.setup.dispatch_event",
        trace_id = tracing::field::Empty,
        trace_ts = tracing::field::Empty,
    );
    record_trace_fields(&span, &_trace);
    async {
        let orchestrator = runtime.usecases().setup_orchestrator();
        let result: Result<SetupState, String> = orchestrator
            .dispatch(event)
            .await
            .map_err(|err| err.to_string());
        result
    }
    .instrument(span)
    .await
}
