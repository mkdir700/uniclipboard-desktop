//! In-memory and logging adapters for the lifecycle coordinator ports.
//!
//! These adapters are used to wire the `AppLifecycleCoordinator` into the
//! Tauri runtime. Tauri-specific emitters (using `AppHandle`) will be added
//! in a later task when the frontend lifecycle UI is connected.

use anyhow::Result;
use async_trait::async_trait;
use uc_app::usecases::{
    LifecycleEvent, LifecycleEventEmitter, LifecycleState, LifecycleStatusPort, SessionReadyEmitter,
};

// ---------------------------------------------------------------------------
// InMemoryLifecycleStatus
// ---------------------------------------------------------------------------

/// Stores lifecycle state in a `tokio::sync::Mutex`.
///
/// This adapter is intended to live as an `Arc<InMemoryLifecycleStatus>` inside
/// `AppRuntime` so that repeated calls to `app_lifecycle_coordinator()` share
/// the same status instance.
pub struct InMemoryLifecycleStatus {
    state: tokio::sync::Mutex<LifecycleState>,
}

impl InMemoryLifecycleStatus {
    pub fn new() -> Self {
        Self {
            state: tokio::sync::Mutex::new(LifecycleState::Idle),
        }
    }
}

#[async_trait]
impl LifecycleStatusPort for InMemoryLifecycleStatus {
    async fn set_state(&self, state: LifecycleState) -> Result<()> {
        *self.state.lock().await = state;
        Ok(())
    }

    async fn get_state(&self) -> LifecycleState {
        self.state.lock().await.clone()
    }
}

// ---------------------------------------------------------------------------
// LoggingLifecycleEventEmitter
// ---------------------------------------------------------------------------

/// Logs lifecycle events using `tracing`. Does not emit to the frontend.
///
/// This is a placeholder adapter. A Tauri-specific emitter that uses
/// `AppHandle::emit()` will replace this once the frontend lifecycle UI
/// is implemented.
pub struct LoggingLifecycleEventEmitter;

#[async_trait]
impl LifecycleEventEmitter for LoggingLifecycleEventEmitter {
    async fn emit_lifecycle_event(&self, event: LifecycleEvent) -> Result<()> {
        tracing::info!(event = ?event, "Lifecycle event");
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// LoggingSessionReadyEmitter
// ---------------------------------------------------------------------------

/// Logs the session-ready signal using `tracing`. Does not emit to the frontend.
///
/// This is a placeholder adapter. A Tauri-specific emitter that uses
/// `AppHandle::emit()` will replace this once the frontend is ready.
pub struct LoggingSessionReadyEmitter;

#[async_trait]
impl SessionReadyEmitter for LoggingSessionReadyEmitter {
    async fn emit_ready(&self) -> Result<()> {
        tracing::info!("Session ready");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn in_memory_lifecycle_status_defaults_to_idle() {
        let status = InMemoryLifecycleStatus::new();
        assert_eq!(status.get_state().await, LifecycleState::Idle);
    }

    #[tokio::test]
    async fn in_memory_lifecycle_status_set_and_get() {
        let status = InMemoryLifecycleStatus::new();
        status.set_state(LifecycleState::Ready).await.unwrap();
        assert_eq!(status.get_state().await, LifecycleState::Ready);
    }

    #[tokio::test]
    async fn logging_lifecycle_event_emitter_does_not_error() {
        let emitter = LoggingLifecycleEventEmitter;
        let result = emitter.emit_lifecycle_event(LifecycleEvent::Ready).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn logging_session_ready_emitter_does_not_error() {
        let emitter = LoggingSessionReadyEmitter;
        let result = emitter.emit_ready().await;
        assert!(result.is_ok());
    }
}
