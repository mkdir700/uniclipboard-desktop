//! Encryption-related Tauri commands
//! 加密相关的 Tauri 命令

use crate::bootstrap::AppRuntime;
use crate::events::EncryptionEvent;
use std::sync::Arc;
use std::time::SystemTime;
use tauri::{AppHandle, Emitter, Runtime, State};
use tracing::{info, info_span, warn, Instrument};

const LOG_CONTEXT: &str = "[initialize_encryption]";
const UNLOCK_CONTEXT: &str = "[unlock_encryption_session]";

/// Event payload for onboarding-password-set event
#[derive(Debug, Clone, serde::Serialize)]
struct OnboardingPasswordSetEvent {
    timestamp: u64,
}

/// Encryption session status payload
/// 加密会话状态载荷
#[derive(Debug, Clone, serde::Serialize)]
pub struct EncryptionSessionStatus {
    initialized: bool,
    session_ready: bool,
}

/// Initialize encryption with passphrase
/// 使用密码短语初始化加密
///
/// This command uses the InitializeEncryption use case through the UseCases accessor.
/// 此命令通过 UseCases 访问器使用 InitializeEncryption 用例。
///
/// ## Architecture / 架构
///
/// - Commands layer (Driving Adapter) → UseCases accessor → Use Case → Ports
/// - Command triggers watcher start via WatcherControlPort after successful init
/// - 命令层（驱动适配器）→ UseCases 访问器 → 用例 → 端口
/// - 加密成功后通过 WatcherControlPort 启动监控器
#[tauri::command]
pub async fn initialize_encryption(
    runtime: State<'_, Arc<AppRuntime>>,
    app_handle: AppHandle,
    passphrase: String,
) -> Result<(), String> {
    let span = info_span!(
        "command.encryption.initialize",
        device_id = %runtime.deps.device_identity.current_device_id(),
    );

    let uc = runtime.usecases().initialize_encryption();
    log::debug!("{} Use case created, executing...", LOG_CONTEXT);

    uc.execute(uc_core::security::model::Passphrase(passphrase))
        .instrument(span)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to initialize encryption");
            e.to_string()
        })?;
    tracing::info!("Encryption initialized successfully");

    match runtime.usecases().start_clipboard_watcher().execute().await {
        Ok(()) => {
            tracing::info!("Clipboard watcher started after encryption initialization");
        }
        Err(e) => {
            tracing::error!("Failed to start clipboard watcher: {}", e);
            if let Err(err) = app_handle.emit("encryption://watcher-start-failed", format!("{}", e))
            {
                tracing::error!("Failed to emit watcher start failure event: {}", err);
            }
            return Err(format!(
                "Encryption initialized, but failed to start clipboard watcher: {}",
                e
            ));
        }
    }

    if let Err(e) = emit_session_ready(&app_handle) {
        tracing::warn!("Failed to emit encryption session ready event: {}", e);
    }

    // Emit onboarding-password-set event for frontend
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| format!("Failed to get timestamp: {}", e))?
        .as_millis() as u64;

    let event = OnboardingPasswordSetEvent { timestamp };
    app_handle
        .emit("onboarding-password-set", event)
        .map_err(|e| format!("Failed to emit event: {}", e))?;

    tracing::debug!("{} Event emitted successfully", LOG_CONTEXT);
    tracing::info!("Onboarding: encryption password initialized successfully");

    Ok(())
}

fn emit_session_ready<R: Runtime>(app_handle: &tauri::AppHandle<R>) -> Result<(), String> {
    app_handle
        .emit("encryption://event", EncryptionEvent::SessionReady)
        .map_err(|e| format!("emit session ready event failed: {}", e))
}

fn emit_session_failed<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
    reason: String,
) -> Result<(), String> {
    app_handle
        .emit("encryption://event", EncryptionEvent::Failed { reason })
        .map_err(|e| format!("emit session failed event failed: {}", e))
}

pub async fn unlock_encryption_session_with_runtime(
    runtime: &Arc<AppRuntime>,
    app_handle: &AppHandle,
) -> Result<bool, String> {
    let span = info_span!(
        "command.encryption.unlock_session",
        device_id = %runtime.deps.device_identity.current_device_id(),
    );
    let uc = runtime.usecases().auto_unlock_encryption_session();
    info!("{} Attempting keyring unlock", UNLOCK_CONTEXT);
    async {
        match uc.execute().await {
            Ok(true) => {
                info!("{} Keyring unlock completed", UNLOCK_CONTEXT);
                if let Err(e) = emit_session_ready(app_handle) {
                    warn!(
                        "{} Failed to emit session ready event: {}",
                        UNLOCK_CONTEXT, e
                    );
                }

                if let Err(e) = runtime.usecases().start_clipboard_watcher().execute().await {
                    warn!(
                        "{} Failed to start clipboard watcher: {}",
                        UNLOCK_CONTEXT, e
                    );
                    if let Err(emit_err) =
                        app_handle.emit("encryption://watcher-start-failed", format!("{}", e))
                    {
                        warn!(
                            "{} Failed to emit watcher-start-failed event: {}",
                            UNLOCK_CONTEXT, emit_err
                        );
                    }
                }

                Ok(true)
            }
            Ok(false) => {
                info!(
                    "{} Encryption not initialized, unlock skipped",
                    UNLOCK_CONTEXT
                );
                Ok(false)
            }
            Err(err) => {
                let reason = err.to_string();
                warn!("{} Keyring unlock failed: {}", UNLOCK_CONTEXT, reason);
                if let Err(emit_err) = emit_session_failed(app_handle, reason.clone()) {
                    warn!(
                        "{} Failed to emit session failed event: {}",
                        UNLOCK_CONTEXT, emit_err
                    );
                }
                Err(reason)
            }
        }
    }
    .instrument(span)
    .await
}

#[tauri::command]
pub async fn unlock_encryption_session(
    runtime: State<'_, Arc<AppRuntime>>,
    app_handle: AppHandle,
) -> Result<bool, String> {
    unlock_encryption_session_with_runtime(runtime.inner(), &app_handle).await
}

#[cfg(test)]
mod tests {
    use super::emit_session_ready;
    use serde_json::Value;
    use tauri::Listener;
    #[tokio::test]
    async fn emit_session_ready_emits_event() {
        let app = tauri::test::mock_app();
        let app_handle = app.handle();
        let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(1);

        let tx_clone = tx.clone();
        app_handle.listen("encryption://event", move |event: tauri::Event| {
            let _ = tx_clone.try_send(event.payload().to_string());
        });

        emit_session_ready(&app_handle).expect("emit session ready event");

        let payload = rx.recv().await.expect("event payload");
        let value: Value = serde_json::from_str(&payload).expect("json payload");
        assert_eq!(value, serde_json::json!({ "type": "SessionReady" }));
    }
}

/// Check if encryption is initialized
/// 检查加密是否已初始化
///
/// This command uses the IsEncryptionInitialized use case.
/// 此命令使用 IsEncryptionInitialized 用例。
#[tauri::command]
pub async fn is_encryption_initialized(
    runtime: State<'_, Arc<AppRuntime>>,
) -> Result<bool, String> {
    let span = info_span!(
        "command.encryption.is_initialized",
        device_id = %runtime.deps.device_identity.current_device_id(),
    );
    async {
        let uc = runtime.usecases().is_encryption_initialized();
        let result = uc.execute().await.map_err(|e| {
            tracing::error!(error = %e, "Failed to check encryption status");
            e.to_string()
        })?;

        tracing::info!(is_initialized = result, "Encryption status checked");
        Ok(result)
    }
    .instrument(span)
    .await
}

/// Get encryption session readiness
/// 获取加密会话就绪状态
///
/// This command reports whether encryption is initialized and whether the session is ready.
/// 此命令返回加密是否已初始化以及会话是否就绪。
#[tauri::command]
pub async fn get_encryption_session_status(
    runtime: State<'_, Arc<AppRuntime>>,
) -> Result<EncryptionSessionStatus, String> {
    let span = info_span!(
        "command.encryption.session_status",
        device_id = %runtime.deps.device_identity.current_device_id(),
    );

    async {
        let state = runtime
            .deps
            .encryption_state
            .load_state()
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to load encryption state");
                e.to_string()
            })?;

        let session_ready = runtime.deps.encryption_session.is_ready().await;
        let initialized = state == uc_core::security::state::EncryptionState::Initialized;

        tracing::info!(
            initialized,
            session_ready,
            "Encryption session status checked"
        );

        Ok(EncryptionSessionStatus {
            initialized,
            session_ready,
        })
    }
    .instrument(span)
    .await
}
