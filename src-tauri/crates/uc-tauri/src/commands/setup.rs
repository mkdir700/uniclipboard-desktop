//! Setup-related Tauri commands
//! 设置流程相关的 Tauri 命令

use crate::bootstrap::AppRuntime;
use crate::commands::record_trace_fields;
use std::sync::Arc;
use tauri::State;
use tracing::{info_span, Instrument};
use uc_core::ports::observability::TraceMetadata;
use uc_core::setup::SetupState;

/// Get current setup state.
/// 获取当前设置流程状态。
#[tauri::command]
pub async fn get_state(
    runtime: State<'_, Arc<AppRuntime>>,
    _trace: Option<TraceMetadata>,
) -> Result<SetupState, String> {
    let span = info_span!(
        "command.setup.get_state",
        trace_id = tracing::field::Empty,
        trace_ts = tracing::field::Empty,
    );
    record_trace_fields(&span, &_trace);
    async { Ok(runtime.usecases().setup_orchestrator().get_state().await) }
        .instrument(span)
        .await
}

#[tauri::command]
pub async fn start_new_space(
    runtime: State<'_, Arc<AppRuntime>>,
    _trace: Option<TraceMetadata>,
) -> Result<SetupState, String> {
    let span = info_span!(
        "command.setup.start_new_space",
        trace_id = tracing::field::Empty,
        trace_ts = tracing::field::Empty,
    );
    record_trace_fields(&span, &_trace);
    async {
        let orchestrator = runtime.usecases().setup_orchestrator();
        orchestrator.new_space().await.map_err(|e| e.to_string())
    }
    .instrument(span)
    .await
}

#[tauri::command]
pub async fn start_join_space(
    runtime: State<'_, Arc<AppRuntime>>,
    _trace: Option<TraceMetadata>,
) -> Result<SetupState, String> {
    let span = info_span!(
        "command.setup.start_join_space",
        trace_id = tracing::field::Empty,
        trace_ts = tracing::field::Empty,
    );
    record_trace_fields(&span, &_trace);
    async {
        let orchestrator = runtime.usecases().setup_orchestrator();
        orchestrator.join_space().await.map_err(|e| e.to_string())
    }
    .instrument(span)
    .await
}

#[tauri::command]
pub async fn select_device(
    runtime: State<'_, Arc<AppRuntime>>,
    peer_id: String,
    _trace: Option<TraceMetadata>,
) -> Result<SetupState, String> {
    let span = info_span!(
        "command.setup.select_device",
        trace_id = tracing::field::Empty,
        trace_ts = tracing::field::Empty,
    );
    record_trace_fields(&span, &_trace);
    async {
        let orchestrator = runtime.usecases().setup_orchestrator();
        orchestrator
            .select_device(peer_id)
            .await
            .map_err(|e| e.to_string())
    }
    .instrument(span)
    .await
}

#[tauri::command]
pub async fn submit_passphrase(
    runtime: State<'_, Arc<AppRuntime>>,
    passphrase1: String,
    passphrase2: String,
    _trace: Option<TraceMetadata>,
) -> Result<SetupState, String> {
    let span = info_span!(
        "command.setup.submit_passphrase",
        trace_id = tracing::field::Empty,
        trace_ts = tracing::field::Empty,
    );
    record_trace_fields(&span, &_trace);
    async {
        let orchestrator = runtime.usecases().setup_orchestrator();
        orchestrator
            .submit_passphrase(passphrase1, passphrase2)
            .await
            .map_err(|e| e.to_string())
    }
    .instrument(span)
    .await
}

#[tauri::command]
pub async fn verify_passphrase(
    runtime: State<'_, Arc<AppRuntime>>,
    passphrase: String,
    _trace: Option<TraceMetadata>,
) -> Result<SetupState, String> {
    let span = info_span!(
        "command.setup.verify_passphrase",
        trace_id = tracing::field::Empty,
        trace_ts = tracing::field::Empty,
    );
    record_trace_fields(&span, &_trace);
    async {
        let orchestrator = runtime.usecases().setup_orchestrator();
        orchestrator
            .verify_passphrase(passphrase)
            .await
            .map_err(|e| e.to_string())
    }
    .instrument(span)
    .await
}

#[tauri::command]
pub async fn cancel_setup(
    runtime: State<'_, Arc<AppRuntime>>,
    _trace: Option<TraceMetadata>,
) -> Result<SetupState, String> {
    let span = info_span!(
        "command.setup.cancel_setup",
        trace_id = tracing::field::Empty,
        trace_ts = tracing::field::Empty,
    );
    record_trace_fields(&span, &_trace);
    async {
        let orchestrator = runtime.usecases().setup_orchestrator();
        orchestrator.cancel_setup().await.map_err(|e| e.to_string())
    }
    .instrument(span)
    .await
}
