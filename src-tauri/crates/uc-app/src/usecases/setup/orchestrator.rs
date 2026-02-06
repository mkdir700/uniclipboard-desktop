//! Setup orchestrator.
//!
//! This module coordinates the setup state machine and side effects.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::sync::Mutex;
use tracing::{error, info_span, Instrument};

use uc_core::{
    ports::SetupStatusPort,
    security::{model::Passphrase, SecretString},
    setup::{SetupAction, SetupEvent, SetupState, SetupStateMachine},
};

use crate::usecases::initialize_encryption::InitializeEncryptionError;
use crate::usecases::setup::context::SetupContext;
use crate::usecases::setup::MarkSetupComplete;
use crate::usecases::AppLifecycleCoordinator;
use crate::usecases::InitializeEncryption;

/// Errors produced by the setup orchestrator.
#[derive(Debug, thiserror::Error)]
pub enum SetupError {
    #[error("initialize encryption failed: {0}")]
    InitializeEncryption(#[from] InitializeEncryptionError),
    #[error("mark setup complete failed: {0}")]
    MarkSetupComplete(#[from] anyhow::Error),
    #[error("lifecycle boot failed: {0}")]
    LifecycleFailed(#[source] anyhow::Error),
    #[error("setup action not implemented: {0}")]
    ActionNotImplemented(&'static str),
}

/// Orchestrator that drives setup state and side effects.
pub struct SetupOrchestrator {
    context: Arc<SetupContext>,

    selected_peer_id: Arc<Mutex<Option<String>>>,
    passphrase: Arc<Mutex<Option<Passphrase>>>,
    seeded: AtomicBool,

    // 能力型 use cases (依赖注入)
    initialize_encryption: Arc<InitializeEncryption>,
    mark_setup_complete: Arc<MarkSetupComplete>,
    setup_status: Arc<dyn SetupStatusPort>,
    app_lifecycle: Arc<AppLifecycleCoordinator>,
}

impl SetupOrchestrator {
    pub fn new(
        initialize_encryption: Arc<InitializeEncryption>,
        mark_setup_complete: Arc<MarkSetupComplete>,
        setup_status: Arc<dyn SetupStatusPort>,
        app_lifecycle: Arc<AppLifecycleCoordinator>,
    ) -> Self {
        Self {
            context: SetupContext::default().arc(),
            selected_peer_id: Arc::new(Mutex::new(None)),
            passphrase: Arc::new(Mutex::new(None)),
            seeded: AtomicBool::new(false),
            initialize_encryption,
            mark_setup_complete,
            setup_status,
            app_lifecycle,
        }
    }

    pub async fn new_space(&self) -> Result<SetupState, SetupError> {
        let event = SetupEvent::StartNewSpace;
        self.dispatch(event).await
    }

    pub async fn join_space(&self) -> Result<SetupState, SetupError> {
        let event = SetupEvent::StartJoinSpace;
        self.dispatch(event).await
    }

    pub async fn select_device(&self, peer_id: String) -> Result<SetupState, SetupError> {
        let event = SetupEvent::ChooseJoinPeer { peer_id };
        self.dispatch(event).await
    }

    pub async fn submit_passphrase(
        &self,
        pass1: String,
        _pass2: String,
    ) -> Result<SetupState, SetupError> {
        let event = SetupEvent::SubmitPassphrase {
            passphrase: SecretString::new(pass1),
        };
        self.dispatch(event).await
    }

    pub async fn verify_passphrase(&self, passphrase: String) -> Result<SetupState, SetupError> {
        let event = SetupEvent::VerifyPassphrase {
            passphrase: SecretString::new(passphrase),
        };
        self.dispatch(event).await
    }

    pub async fn cancel_setup(&self) -> Result<SetupState, SetupError> {
        let event = SetupEvent::CancelSetup;
        self.dispatch(event).await
    }

    pub async fn get_state(&self) -> SetupState {
        self.seed_state_from_status().await;
        self.context.get_state().await
    }

    async fn dispatch(&self, event: SetupEvent) -> Result<SetupState, SetupError> {
        let event = self.capture_context(event).await;
        // Acquire dispatch lock to serialize concurrent dispatch calls.
        // This prevents race conditions where multiple calls read the same state
        // and execute duplicate actions.
        let _dispatch_guard = self.context.acquire_dispatch_lock().await;

        let span = info_span!("usecase.setup_orchestrator.dispatch", event = ?event);
        async {
            let mut current = self.context.get_state().await;
            let mut pending_events = vec![event];

            while let Some(event) = pending_events.pop() {
                let (next, actions) = SetupStateMachine::transition(current, event);
                let follow_up_events = self.execute_actions(actions).await?;
                self.context.set_state(next.clone()).await;
                current = next;
                pending_events.extend(follow_up_events);
            }

            Ok(current)
        }
        .instrument(span)
        .await
    }

    async fn execute_actions(
        &self,
        actions: Vec<SetupAction>,
    ) -> Result<Vec<SetupEvent>, SetupError> {
        let mut follow_up_events = Vec::new();
        for action in actions {
            match action {
                SetupAction::CreateEncryptedSpace => {
                    let passphrase = self.take_passphrase().await?;
                    self.initialize_encryption.execute(passphrase).await?;
                    // Boot watcher + network + session ready
                    self.app_lifecycle
                        .ensure_ready()
                        .await
                        .map_err(SetupError::LifecycleFailed)?;
                    follow_up_events.push(SetupEvent::CreateSpaceSucceeded);
                }
                SetupAction::MarkSetupComplete => {
                    self.mark_setup_complete.execute().await?;
                }
                SetupAction::EnsureDiscovery => {
                    error!("Setup action EnsureDiscovery is not implemented yet");
                    return Err(SetupError::ActionNotImplemented("EnsureDiscovery"));
                }
                SetupAction::EnsurePairing => {
                    error!("Setup action EnsurePairing is not implemented yet");
                    return Err(SetupError::ActionNotImplemented("EnsurePairing"));
                }
                SetupAction::ConfirmPeerTrust => {
                    error!("Setup action ConfirmPeerTrust is not implemented yet");
                    return Err(SetupError::ActionNotImplemented("ConfirmPeerTrust"));
                }
                SetupAction::AbortPairing => {
                    error!("Setup action AbortPairing is not implemented yet");
                    return Err(SetupError::ActionNotImplemented("AbortPairing"));
                }
                SetupAction::StartJoinSpaceAccess => {
                    error!("Setup action StartJoinSpaceAccess is not implemented yet");
                    return Err(SetupError::ActionNotImplemented("StartJoinSpaceAccess"));
                }
            }
        }

        Ok(follow_up_events)
    }

    async fn capture_context(&self, event: SetupEvent) -> SetupEvent {
        match event {
            SetupEvent::ChooseJoinPeer { peer_id } => {
                *self.selected_peer_id.lock().await = Some(peer_id.clone());
                SetupEvent::ChooseJoinPeer { peer_id }
            }
            SetupEvent::SubmitPassphrase { passphrase } => {
                let (event_passphrase, stored_passphrase) = Self::split_passphrase(passphrase);
                *self.passphrase.lock().await = Some(stored_passphrase);
                SetupEvent::SubmitPassphrase {
                    passphrase: event_passphrase,
                }
            }
            SetupEvent::VerifyPassphrase { passphrase } => {
                let (event_passphrase, stored_passphrase) = Self::split_passphrase(passphrase);
                *self.passphrase.lock().await = Some(stored_passphrase);
                SetupEvent::VerifyPassphrase {
                    passphrase: event_passphrase,
                }
            }
            other => other,
        }
    }

    async fn take_passphrase(&self) -> Result<Passphrase, SetupError> {
        let mut guard = self.passphrase.lock().await;
        guard
            .take()
            .ok_or(SetupError::ActionNotImplemented("CreateEncryptedSpace"))
    }

    fn split_passphrase(passphrase: SecretString) -> (SecretString, Passphrase) {
        let raw = passphrase.into_inner();
        let stored = Passphrase(raw.clone());
        (SecretString::new(raw), stored)
    }

    async fn seed_state_from_status(&self) {
        if self.seeded.swap(true, Ordering::SeqCst) {
            return;
        }

        match self.setup_status.get_status().await {
            Ok(status) => {
                if status.has_completed {
                    self.context.set_state(SetupState::Completed).await;
                }
            }
            Err(err) => {
                error!(error = %err, "failed to load setup status");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex as StdMutex};
    use uc_core::ports::network_control::NetworkControlPort;
    use uc_core::ports::security::encryption::EncryptionPort;
    use uc_core::ports::security::encryption_session::EncryptionSessionPort;
    use uc_core::ports::security::encryption_state::EncryptionStatePort;
    use uc_core::ports::security::key_material::KeyMaterialPort;
    use uc_core::ports::security::key_scope::{KeyScopePort, ScopeError};
    use uc_core::ports::watcher_control::{WatcherControlError, WatcherControlPort};
    use uc_core::security::model::{
        EncryptedBlob, EncryptionAlgo, EncryptionError, Kek, KeyScope, KeySlot, MasterKey,
        Passphrase,
    };
    use uc_core::security::state::{EncryptionState, EncryptionStateError};
    use uc_core::setup::SetupStatus;

    use crate::usecases::{
        AppLifecycleCoordinatorDeps, LifecycleEvent, LifecycleEventEmitter, LifecycleState,
        LifecycleStatusPort, SessionReadyEmitter, StartClipboardWatcher, StartNetworkAfterUnlock,
    };

    struct MockSetupStatusPort {
        status: StdMutex<SetupStatus>,
        set_calls: AtomicUsize,
    }

    impl MockSetupStatusPort {
        fn new(status: SetupStatus) -> Self {
            Self {
                status: StdMutex::new(status),
                set_calls: AtomicUsize::new(0),
            }
        }

        fn set_call_count(&self) -> usize {
            self.set_calls.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl SetupStatusPort for MockSetupStatusPort {
        async fn get_status(&self) -> anyhow::Result<SetupStatus> {
            Ok(self.status.lock().unwrap().clone())
        }

        async fn set_status(&self, status: &SetupStatus) -> anyhow::Result<()> {
            *self.status.lock().unwrap() = status.clone();
            self.set_calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    struct NoopEncryption;

    #[async_trait]
    impl EncryptionPort for NoopEncryption {
        async fn derive_kek(
            &self,
            _passphrase: &Passphrase,
            _salt: &[u8],
            _kdf_params: &uc_core::security::model::KdfParams,
        ) -> Result<Kek, EncryptionError> {
            Err(EncryptionError::NotInitialized)
        }

        async fn wrap_master_key(
            &self,
            _kek: &Kek,
            _master_key: &MasterKey,
            _aead: EncryptionAlgo,
        ) -> Result<EncryptedBlob, EncryptionError> {
            Err(EncryptionError::NotInitialized)
        }

        async fn unwrap_master_key(
            &self,
            _kek: &Kek,
            _blob: &EncryptedBlob,
        ) -> Result<MasterKey, EncryptionError> {
            Err(EncryptionError::NotInitialized)
        }

        async fn encrypt_blob(
            &self,
            _master_key: &MasterKey,
            _plaintext: &[u8],
            _aad: &[u8],
            _algo: EncryptionAlgo,
        ) -> Result<EncryptedBlob, EncryptionError> {
            Err(EncryptionError::NotInitialized)
        }

        async fn decrypt_blob(
            &self,
            _master_key: &MasterKey,
            _blob: &EncryptedBlob,
            _aad: &[u8],
        ) -> Result<Vec<u8>, EncryptionError> {
            Err(EncryptionError::NotInitialized)
        }
    }

    struct NoopKeyMaterial;

    #[async_trait]
    impl KeyMaterialPort for NoopKeyMaterial {
        async fn load_keyslot(&self, _scope: &KeyScope) -> Result<KeySlot, EncryptionError> {
            Err(EncryptionError::KeyNotFound)
        }

        async fn store_keyslot(&self, _keyslot: &KeySlot) -> Result<(), EncryptionError> {
            Ok(())
        }

        async fn delete_keyslot(&self, _scope: &KeyScope) -> Result<(), EncryptionError> {
            Ok(())
        }

        async fn load_kek(&self, _scope: &KeyScope) -> Result<Kek, EncryptionError> {
            Err(EncryptionError::KeyNotFound)
        }

        async fn store_kek(&self, _scope: &KeyScope, _kek: &Kek) -> Result<(), EncryptionError> {
            Ok(())
        }

        async fn delete_kek(&self, _scope: &KeyScope) -> Result<(), EncryptionError> {
            Ok(())
        }
    }

    struct NoopKeyScope;

    #[async_trait]
    impl KeyScopePort for NoopKeyScope {
        async fn current_scope(&self) -> Result<KeyScope, ScopeError> {
            Err(ScopeError::FailedToGetCurrentScope)
        }
    }

    struct NoopEncryptionState;

    #[async_trait]
    impl EncryptionStatePort for NoopEncryptionState {
        async fn load_state(&self) -> Result<EncryptionState, EncryptionStateError> {
            Err(EncryptionStateError::LoadError("noop".to_string()))
        }

        async fn persist_initialized(&self) -> Result<(), EncryptionStateError> {
            Ok(())
        }
    }

    struct NoopEncryptionSession;

    #[async_trait]
    impl EncryptionSessionPort for NoopEncryptionSession {
        async fn is_ready(&self) -> bool {
            false
        }

        async fn get_master_key(&self) -> Result<MasterKey, EncryptionError> {
            Err(EncryptionError::NotInitialized)
        }

        async fn set_master_key(&self, _master_key: MasterKey) -> Result<(), EncryptionError> {
            Ok(())
        }

        async fn clear(&self) -> Result<(), EncryptionError> {
            Ok(())
        }
    }

    struct SucceedEncryption;

    #[async_trait]
    impl EncryptionPort for SucceedEncryption {
        async fn derive_kek(
            &self,
            _passphrase: &Passphrase,
            _salt: &[u8],
            _kdf_params: &uc_core::security::model::KdfParams,
        ) -> Result<Kek, EncryptionError> {
            Ok(Kek([0u8; 32]))
        }

        async fn wrap_master_key(
            &self,
            _kek: &Kek,
            _master_key: &MasterKey,
            _aead: EncryptionAlgo,
        ) -> Result<EncryptedBlob, EncryptionError> {
            Ok(EncryptedBlob {
                version: uc_core::security::model::EncryptionFormatVersion::V1,
                aead: EncryptionAlgo::XChaCha20Poly1305,
                nonce: vec![0u8; 24],
                ciphertext: vec![1u8; 32],
                aad_fingerprint: None,
            })
        }

        async fn unwrap_master_key(
            &self,
            _kek: &Kek,
            _blob: &EncryptedBlob,
        ) -> Result<MasterKey, EncryptionError> {
            Ok(MasterKey([0u8; 32]))
        }

        async fn encrypt_blob(
            &self,
            _master_key: &MasterKey,
            _plaintext: &[u8],
            _aad: &[u8],
            _algo: EncryptionAlgo,
        ) -> Result<EncryptedBlob, EncryptionError> {
            Ok(EncryptedBlob {
                version: uc_core::security::model::EncryptionFormatVersion::V1,
                aead: EncryptionAlgo::XChaCha20Poly1305,
                nonce: vec![0u8; 24],
                ciphertext: vec![1u8; 32],
                aad_fingerprint: None,
            })
        }

        async fn decrypt_blob(
            &self,
            _master_key: &MasterKey,
            _blob: &EncryptedBlob,
            _aad: &[u8],
        ) -> Result<Vec<u8>, EncryptionError> {
            Ok(vec![0u8; 32])
        }
    }

    struct SucceedKeyMaterial;

    #[async_trait]
    impl KeyMaterialPort for SucceedKeyMaterial {
        async fn load_keyslot(&self, _scope: &KeyScope) -> Result<KeySlot, EncryptionError> {
            Err(EncryptionError::KeyNotFound)
        }

        async fn store_keyslot(&self, _keyslot: &KeySlot) -> Result<(), EncryptionError> {
            Ok(())
        }

        async fn delete_keyslot(&self, _scope: &KeyScope) -> Result<(), EncryptionError> {
            Ok(())
        }

        async fn load_kek(&self, _scope: &KeyScope) -> Result<Kek, EncryptionError> {
            Err(EncryptionError::KeyNotFound)
        }

        async fn store_kek(&self, _scope: &KeyScope, _kek: &Kek) -> Result<(), EncryptionError> {
            Ok(())
        }

        async fn delete_kek(&self, _scope: &KeyScope) -> Result<(), EncryptionError> {
            Ok(())
        }
    }

    struct SucceedKeyScope;

    #[async_trait]
    impl KeyScopePort for SucceedKeyScope {
        async fn current_scope(&self) -> Result<KeyScope, ScopeError> {
            Ok(KeyScope {
                profile_id: "default".to_string(),
            })
        }
    }

    struct SucceedEncryptionState;

    #[async_trait]
    impl EncryptionStatePort for SucceedEncryptionState {
        async fn load_state(&self) -> Result<EncryptionState, EncryptionStateError> {
            Ok(EncryptionState::Uninitialized)
        }

        async fn persist_initialized(&self) -> Result<(), EncryptionStateError> {
            Ok(())
        }
    }

    struct SucceedEncryptionSession;

    #[async_trait]
    impl EncryptionSessionPort for SucceedEncryptionSession {
        async fn is_ready(&self) -> bool {
            false
        }

        async fn get_master_key(&self) -> Result<MasterKey, EncryptionError> {
            Err(EncryptionError::NotInitialized)
        }

        async fn set_master_key(&self, _master_key: MasterKey) -> Result<(), EncryptionError> {
            Ok(())
        }

        async fn clear(&self) -> Result<(), EncryptionError> {
            Ok(())
        }
    }

    // -- Lifecycle mocks -------------------------------------------------------

    struct MockWatcherControl;

    #[async_trait]
    impl WatcherControlPort for MockWatcherControl {
        async fn start_watcher(&self) -> Result<(), WatcherControlError> {
            Ok(())
        }

        async fn stop_watcher(&self) -> Result<(), WatcherControlError> {
            Ok(())
        }
    }

    struct MockNetworkControl;

    #[async_trait]
    impl NetworkControlPort for MockNetworkControl {
        async fn start_network(&self) -> anyhow::Result<()> {
            Ok(())
        }
    }

    struct MockSessionReadyEmitter;

    #[async_trait]
    impl SessionReadyEmitter for MockSessionReadyEmitter {
        async fn emit_ready(&self) -> anyhow::Result<()> {
            Ok(())
        }
    }

    struct MockLifecycleStatus;

    #[async_trait]
    impl LifecycleStatusPort for MockLifecycleStatus {
        async fn set_state(&self, _state: LifecycleState) -> anyhow::Result<()> {
            Ok(())
        }

        async fn get_state(&self) -> LifecycleState {
            LifecycleState::Idle
        }
    }

    struct MockLifecycleEventEmitter;

    #[async_trait]
    impl LifecycleEventEmitter for MockLifecycleEventEmitter {
        async fn emit_lifecycle_event(&self, _event: LifecycleEvent) -> anyhow::Result<()> {
            Ok(())
        }
    }

    fn build_mock_lifecycle() -> Arc<AppLifecycleCoordinator> {
        Arc::new(AppLifecycleCoordinator::from_deps(
            AppLifecycleCoordinatorDeps {
                watcher: Arc::new(StartClipboardWatcher::new(Arc::new(MockWatcherControl))),
                network: Arc::new(StartNetworkAfterUnlock::new(Arc::new(MockNetworkControl))),
                emitter: Arc::new(MockSessionReadyEmitter),
                status: Arc::new(MockLifecycleStatus),
                lifecycle_emitter: Arc::new(MockLifecycleEventEmitter),
            },
        ))
    }

    fn build_initialize_encryption() -> Arc<InitializeEncryption> {
        Arc::new(InitializeEncryption::from_ports(
            Arc::new(NoopEncryption),
            Arc::new(NoopKeyMaterial),
            Arc::new(NoopKeyScope),
            Arc::new(NoopEncryptionState),
            Arc::new(NoopEncryptionSession),
        ))
    }

    fn build_initialize_encryption_success() -> Arc<InitializeEncryption> {
        Arc::new(InitializeEncryption::from_ports(
            Arc::new(SucceedEncryption),
            Arc::new(SucceedKeyMaterial),
            Arc::new(SucceedKeyScope),
            Arc::new(SucceedEncryptionState),
            Arc::new(SucceedEncryptionSession),
        ))
    }

    fn build_orchestrator_with_initialize_encryption(
        setup_status: Arc<dyn SetupStatusPort>,
        initialize_encryption: Arc<InitializeEncryption>,
    ) -> SetupOrchestrator {
        let mark_setup_complete = Arc::new(MarkSetupComplete::new(setup_status.clone()));

        SetupOrchestrator::new(
            initialize_encryption,
            mark_setup_complete,
            setup_status,
            build_mock_lifecycle(),
        )
    }

    fn build_orchestrator(setup_status: Arc<dyn SetupStatusPort>) -> SetupOrchestrator {
        build_orchestrator_with_initialize_encryption(setup_status, build_initialize_encryption())
    }

    #[tokio::test]
    async fn get_state_seeds_completed_when_setup_status_completed() {
        let setup_status = Arc::new(MockSetupStatusPort::new(SetupStatus {
            has_completed: true,
        }));
        let orchestrator = build_orchestrator(setup_status);

        let state = orchestrator.get_state().await;

        assert_eq!(state, SetupState::Completed);
    }

    #[tokio::test]
    async fn join_space_success_marks_setup_complete() {
        let setup_status = Arc::new(MockSetupStatusPort::new(SetupStatus::default()));
        let orchestrator = build_orchestrator(setup_status.clone());

        orchestrator
            .context
            .set_state(SetupState::ProcessingJoinSpace { message: None })
            .await;

        orchestrator
            .dispatch(SetupEvent::JoinSpaceSucceeded)
            .await
            .unwrap();

        let status = setup_status.get_status().await.unwrap();

        assert!(status.has_completed);
        assert_eq!(setup_status.set_call_count(), 1);
    }

    #[tokio::test]
    async fn create_space_success_marks_setup_complete() {
        let setup_status = Arc::new(MockSetupStatusPort::new(SetupStatus::default()));
        let orchestrator = build_orchestrator_with_initialize_encryption(
            setup_status.clone(),
            build_initialize_encryption_success(),
        );

        orchestrator.new_space().await.unwrap();
        let state = orchestrator
            .submit_passphrase("secret".to_string(), "secret".to_string())
            .await
            .unwrap();

        assert_eq!(state, SetupState::Completed);
        let status = setup_status.get_status().await.unwrap();
        assert!(status.has_completed);
        assert_eq!(setup_status.set_call_count(), 1);
    }
}
