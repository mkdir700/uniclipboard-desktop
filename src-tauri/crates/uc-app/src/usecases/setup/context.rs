use std::sync::Arc;

use tokio::sync::Mutex;
use uc_core::setup::SetupState;

/// Shared setup context containing state and dispatch lock.
///
/// This context is shared between `SetupOrchestrator` and setup usecases
/// to ensure consistent state access and proper serialization of dispatch calls.
///
/// ## Lock Ordering
/// When acquiring both locks, acquire `dispatch_lock` first, then `state`.
/// - `dispatch_lock`: Used only for `dispatch` operations to serialize concurrent calls.
/// - `state`: Used for both reading (`get_state`) and writing (during `dispatch`).
#[derive(Clone)]
pub struct SetupContext {
    /// Current setup state.
    state: Arc<Mutex<SetupState>>,
    /// Serializes dispatch calls to prevent concurrent state/action races.
    /// Ensures the entire transition + execute_actions + state_update runs atomically.
    /// Only acquired during `dispatch`, NOT during `get_state`.
    dispatch_lock: Arc<Mutex<()>>,
}

impl SetupContext {
    /// Creates a new SetupContext with the given initial state.
    pub fn new(initial_state: SetupState) -> Self {
        Self {
            state: Arc::new(Mutex::new(initial_state)),
            dispatch_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Creates a SetupContext with default `Welcome` state.
    pub fn default() -> Self {
        Self::new(SetupState::Welcome)
    }

    /// Returns the context wrapped in Arc for shared ownership.
    pub fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }

    /// Returns a reference to the current state.
    ///
    /// This is a lightweight read operation that does NOT acquire `dispatch_lock`.
    pub async fn get_state(&self) -> SetupState {
        self.state.lock().await.clone()
    }

    /// Acquires the dispatch lock for serializing concurrent dispatch calls.
    ///
    /// Returns a guard that releases the lock when dropped.
    pub async fn acquire_dispatch_lock(&self) -> tokio::sync::MutexGuard<'_, ()> {
        self.dispatch_lock.lock().await
    }

    /// Updates the state to the given value.
    ///
    /// This should only be called after acquiring `dispatch_lock`.
    pub async fn set_state(&self, state: SetupState) {
        let mut guard = self.state.lock().await;
        *guard = state;
    }
}
