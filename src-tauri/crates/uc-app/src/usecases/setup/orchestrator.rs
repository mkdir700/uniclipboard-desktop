//! Setup orchestrator.
//!
//! This module coordinates the setup state machine and side effects.

use std::sync::Arc;

use tokio::sync::Mutex;
use tracing::{error, info_span, Instrument};

use uc_core::{
    ports::{
        security::{encryption_state::EncryptionStatePort, key_scope::KeyScopePort},
        EncryptionPort, EncryptionSessionPort, KeyMaterialPort, OnboardingStatePort,
    },
    security::model::Passphrase,
    setup::{SetupAction, SetupEvent, SetupState, SetupStateMachine},
};

use crate::usecases::initialize_encryption::InitializeEncryptionError;
use crate::usecases::CompleteOnboarding;
use crate::usecases::InitializeEncryption;

/// Errors produced by the setup orchestrator.
#[derive(Debug, thiserror::Error)]
pub enum SetupOrchestratorError {
    #[error("initialize encryption failed: {0}")]
    InitializeEncryption(#[from] InitializeEncryptionError),
    #[error("complete onboarding failed: {0}")]
    CompleteOnboarding(#[from] anyhow::Error),
    #[error("setup action not implemented: {0}")]
    ActionNotImplemented(&'static str),
}

/// Orchestrator that drives setup state and side effects.
pub struct SetupOrchestrator {
    state: Arc<Mutex<SetupState>>,
    /// Serializes dispatch calls to prevent concurrent state/action races.
    /// Ensures the entire transition + execute_actions + state_update runs atomically.
    dispatch_lock: Mutex<()>,
    initialize_encryption: InitializeEncryption,
    complete_onboarding: CompleteOnboarding,
}

impl SetupOrchestrator {
    pub fn new(
        encryption: Arc<dyn EncryptionPort>,
        key_material: Arc<dyn KeyMaterialPort>,
        key_scope: Arc<dyn KeyScopePort>,
        encryption_state: Arc<dyn EncryptionStatePort>,
        encryption_session: Arc<dyn EncryptionSessionPort>,
        onboarding_state: Arc<dyn OnboardingStatePort>,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(SetupState::Welcome)),
            dispatch_lock: Mutex::new(()),
            initialize_encryption: InitializeEncryption::new(
                encryption,
                key_material,
                key_scope,
                encryption_state,
                encryption_session,
            ),
            complete_onboarding: CompleteOnboarding::new(onboarding_state),
        }
    }

    pub fn from_ports(
        encryption: Arc<dyn EncryptionPort>,
        key_material: Arc<dyn KeyMaterialPort>,
        key_scope: Arc<dyn KeyScopePort>,
        encryption_state: Arc<dyn EncryptionStatePort>,
        encryption_session: Arc<dyn EncryptionSessionPort>,
        onboarding_state: Arc<dyn OnboardingStatePort>,
    ) -> Self {
        Self::new(
            encryption,
            key_material,
            key_scope,
            encryption_state,
            encryption_session,
            onboarding_state,
        )
    }

    pub fn with_state(
        state: SetupState,
        encryption: Arc<dyn EncryptionPort>,
        key_material: Arc<dyn KeyMaterialPort>,
        key_scope: Arc<dyn KeyScopePort>,
        encryption_state: Arc<dyn EncryptionStatePort>,
        encryption_session: Arc<dyn EncryptionSessionPort>,
        onboarding_state: Arc<dyn OnboardingStatePort>,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(state)),
            dispatch_lock: Mutex::new(()),
            initialize_encryption: InitializeEncryption::new(
                encryption,
                key_material,
                key_scope,
                encryption_state,
                encryption_session,
            ),
            complete_onboarding: CompleteOnboarding::new(onboarding_state),
        }
    }

    pub async fn get_state(&self) -> SetupState {
        self.state.lock().await.clone()
    }

    pub async fn dispatch(&self, event: SetupEvent) -> Result<SetupState, SetupOrchestratorError> {
        // Acquire dispatch lock to serialize concurrent dispatch calls.
        // This prevents race conditions where multiple calls read the same state
        // and execute duplicate actions.
        let _dispatch_guard = self.dispatch_lock.lock().await;

        let span = info_span!("usecase.setup_orchestrator.dispatch", event = ?event);
        async {
            let current = self.state.lock().await.clone();
            let (next, actions) = SetupStateMachine::transition(current, event);
            self.execute_actions(actions).await?;
            let mut guard = self.state.lock().await;
            *guard = next.clone();
            Ok(next)
        }
        .instrument(span)
        .await
    }

    async fn execute_actions(
        &self,
        actions: Vec<SetupAction>,
    ) -> Result<(), SetupOrchestratorError> {
        for action in actions {
            match action {
                SetupAction::CreateEncryptedSpace { passphrase } => {
                    self.initialize_encryption
                        .execute(Passphrase(passphrase))
                        .await?;
                }
                SetupAction::MarkSetupComplete => {
                    self.complete_onboarding.execute().await?;
                }
                SetupAction::ScanPeers => {
                    error!("Setup action ScanPeers is not implemented yet");
                    return Err(SetupOrchestratorError::ActionNotImplemented("ScanPeers"));
                }
                SetupAction::VerifyPassphraseWithPeer { .. } => {
                    error!("Setup action VerifyPassphraseWithPeer is not implemented yet");
                    return Err(SetupOrchestratorError::ActionNotImplemented(
                        "VerifyPassphraseWithPeer",
                    ));
                }
                SetupAction::StartPairing { .. } => {
                    error!("Setup action StartPairing is not implemented yet");
                    return Err(SetupOrchestratorError::ActionNotImplemented("StartPairing"));
                }
                SetupAction::ConfirmPairing { .. } => {
                    error!("Setup action ConfirmPairing is not implemented yet");
                    return Err(SetupOrchestratorError::ActionNotImplemented(
                        "ConfirmPairing",
                    ));
                }
                SetupAction::CancelPairing { .. } => {
                    error!("Setup action CancelPairing is not implemented yet");
                    return Err(SetupOrchestratorError::ActionNotImplemented(
                        "CancelPairing",
                    ));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use uc_core::{
        onboarding::OnboardingState,
        ports::security::key_scope::ScopeError,
        security::model::{
            EncryptedBlob, EncryptionAlgo, EncryptionError, EncryptionFormatVersion, Kek, KeyScope,
            KeySlot, MasterKey, Passphrase,
        },
        security::state::{EncryptionState, EncryptionStateError},
    };

    struct MockEncryptionState {
        state: EncryptionState,
    }

    #[async_trait]
    impl EncryptionStatePort for MockEncryptionState {
        async fn load_state(&self) -> Result<EncryptionState, EncryptionStateError> {
            Ok(self.state.clone())
        }

        async fn persist_initialized(&self) -> Result<(), EncryptionStateError> {
            Ok(())
        }
    }

    struct MockKeyScope {
        scope: KeyScope,
    }

    #[async_trait]
    impl KeyScopePort for MockKeyScope {
        async fn current_scope(&self) -> Result<KeyScope, ScopeError> {
            Ok(self.scope.clone())
        }
    }

    struct MockKeyMaterial;

    #[async_trait]
    impl KeyMaterialPort for MockKeyMaterial {
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

    struct MockEncryption;

    #[async_trait]
    impl EncryptionPort for MockEncryption {
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
                version: EncryptionFormatVersion::V1,
                aead: EncryptionAlgo::XChaCha20Poly1305,
                nonce: vec![0u8; 24],
                ciphertext: vec![0u8; 32],
                aad_fingerprint: None,
            })
        }

        async fn unwrap_master_key(
            &self,
            _kek: &Kek,
            _blob: &EncryptedBlob,
        ) -> Result<MasterKey, EncryptionError> {
            MasterKey::from_bytes(&[0u8; 32])
        }

        async fn encrypt_blob(
            &self,
            _master_key: &MasterKey,
            _plaintext: &[u8],
            _aad: &[u8],
            _algo: EncryptionAlgo,
        ) -> Result<EncryptedBlob, EncryptionError> {
            Ok(EncryptedBlob {
                version: EncryptionFormatVersion::V1,
                aead: EncryptionAlgo::XChaCha20Poly1305,
                nonce: vec![0u8; 24],
                ciphertext: vec![],
                aad_fingerprint: None,
            })
        }

        async fn decrypt_blob(
            &self,
            _master_key: &MasterKey,
            _blob: &EncryptedBlob,
            _aad: &[u8],
        ) -> Result<Vec<u8>, EncryptionError> {
            Ok(vec![])
        }
    }

    struct MockEncryptionSession {
        master_key_set: AtomicBool,
    }

    #[async_trait]
    impl EncryptionSessionPort for MockEncryptionSession {
        async fn is_ready(&self) -> bool {
            self.master_key_set.load(Ordering::SeqCst)
        }

        async fn get_master_key(&self) -> Result<MasterKey, EncryptionError> {
            if self.master_key_set.load(Ordering::SeqCst) {
                MasterKey::from_bytes(&[0u8; 32])
            } else {
                Err(EncryptionError::Locked)
            }
        }

        async fn set_master_key(&self, _master_key: MasterKey) -> Result<(), EncryptionError> {
            self.master_key_set.store(true, Ordering::SeqCst);
            Ok(())
        }

        async fn clear(&self) -> Result<(), EncryptionError> {
            self.master_key_set.store(false, Ordering::SeqCst);
            Ok(())
        }
    }

    struct MockOnboardingStatePort {
        state: std::sync::Mutex<OnboardingState>,
    }

    #[async_trait]
    impl OnboardingStatePort for MockOnboardingStatePort {
        async fn get_state(&self) -> anyhow::Result<OnboardingState> {
            Ok(self.state.lock().unwrap().clone())
        }

        async fn set_state(&self, state: &OnboardingState) -> anyhow::Result<()> {
            *self.state.lock().unwrap() = state.clone();
            Ok(())
        }

        async fn reset(&self) -> anyhow::Result<()> {
            *self.state.lock().unwrap() = OnboardingState::default();
            Ok(())
        }
    }

    #[tokio::test]
    async fn setup_orchestrator_drives_state_machine_and_executes_actions() {
        let session = Arc::new(MockEncryptionSession {
            master_key_set: AtomicBool::new(false),
        });
        let onboarding_state = Arc::new(MockOnboardingStatePort {
            state: std::sync::Mutex::new(OnboardingState::default()),
        });
        let orchestrator = SetupOrchestrator::new(
            Arc::new(MockEncryption),
            Arc::new(MockKeyMaterial),
            Arc::new(MockKeyScope {
                scope: KeyScope {
                    profile_id: "test".to_string(),
                },
            }),
            Arc::new(MockEncryptionState {
                state: EncryptionState::Uninitialized,
            }),
            session.clone(),
            onboarding_state.clone(),
        );

        let state = orchestrator
            .dispatch(SetupEvent::ChooseCreateSpace)
            .await
            .expect("choose create space");
        assert!(matches!(state, SetupState::CreateSpacePassphrase { .. }));

        let state = orchestrator
            .dispatch(SetupEvent::SubmitCreatePassphrase {
                pass1: "test-passphrase".to_string(),
                pass2: "test-passphrase".to_string(),
            })
            .await
            .expect("submit create passphrase");
        assert_eq!(state, SetupState::Done);
        assert!(session.is_ready().await, "master key should be set");
        assert!(
            onboarding_state
                .get_state()
                .await
                .expect("get onboarding state")
                .has_completed,
            "onboarding should be marked complete"
        );
    }

    struct CountingMockEncryption;

    #[async_trait]
    impl EncryptionPort for CountingMockEncryption {
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
                version: EncryptionFormatVersion::V1,
                aead: EncryptionAlgo::XChaCha20Poly1305,
                nonce: vec![0u8; 24],
                ciphertext: vec![0u8; 32],
                aad_fingerprint: None,
            })
        }

        async fn unwrap_master_key(
            &self,
            _kek: &Kek,
            _blob: &EncryptedBlob,
        ) -> Result<MasterKey, EncryptionError> {
            MasterKey::from_bytes(&[0u8; 32])
        }

        async fn encrypt_blob(
            &self,
            _master_key: &MasterKey,
            _plaintext: &[u8],
            _aad: &[u8],
            _algo: EncryptionAlgo,
        ) -> Result<EncryptedBlob, EncryptionError> {
            Ok(EncryptedBlob {
                version: EncryptionFormatVersion::V1,
                aead: EncryptionAlgo::XChaCha20Poly1305,
                nonce: vec![0u8; 24],
                ciphertext: vec![],
                aad_fingerprint: None,
            })
        }

        async fn decrypt_blob(
            &self,
            _master_key: &MasterKey,
            _blob: &EncryptedBlob,
            _aad: &[u8],
        ) -> Result<Vec<u8>, EncryptionError> {
            Ok(vec![])
        }
    }

    /// Mock key material that tracks store_keyslot calls and adds delay to widen race window.
    struct CountingMockKeyMaterial {
        store_count: AtomicUsize,
    }

    #[async_trait]
    impl KeyMaterialPort for CountingMockKeyMaterial {
        async fn load_keyslot(&self, _scope: &KeyScope) -> Result<KeySlot, EncryptionError> {
            Err(EncryptionError::KeyNotFound)
        }

        async fn store_keyslot(&self, _keyslot: &KeySlot) -> Result<(), EncryptionError> {
            // Add a small delay to widen the race window
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            self.store_count.fetch_add(1, Ordering::SeqCst);
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

    /// Mock session that counts set_master_key calls.
    struct CountingMockEncryptionSession {
        set_count: AtomicUsize,
    }

    #[async_trait]
    impl EncryptionSessionPort for CountingMockEncryptionSession {
        async fn is_ready(&self) -> bool {
            false
        }

        async fn get_master_key(&self) -> Result<MasterKey, EncryptionError> {
            Err(EncryptionError::Locked)
        }

        async fn set_master_key(&self, _master_key: MasterKey) -> Result<(), EncryptionError> {
            self.set_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        async fn clear(&self) -> Result<(), EncryptionError> {
            Ok(())
        }
    }

    /// Test that concurrent dispatch calls are serialized and don't execute duplicate actions.
    /// This test will fail without the dispatch_lock because both concurrent calls
    /// will read the same initial state and execute the same actions.
    #[tokio::test]
    async fn concurrent_dispatch_should_not_execute_duplicate_actions() {
        let key_material = Arc::new(CountingMockKeyMaterial {
            store_count: AtomicUsize::new(0),
        });
        let session = Arc::new(CountingMockEncryptionSession {
            set_count: AtomicUsize::new(0),
        });
        let onboarding_state = Arc::new(MockOnboardingStatePort {
            state: std::sync::Mutex::new(OnboardingState::default()),
        });

        // Start from CreateSpacePassphrase state instead of Welcome
        let orchestrator = Arc::new(SetupOrchestrator::with_state(
            SetupState::CreateSpacePassphrase { error: None },
            Arc::new(CountingMockEncryption),
            key_material.clone(),
            Arc::new(MockKeyScope {
                scope: KeyScope {
                    profile_id: "test".to_string(),
                },
            }),
            Arc::new(MockEncryptionState {
                state: EncryptionState::Uninitialized,
            }),
            session.clone(),
            onboarding_state.clone(),
        ));

        // Spawn two concurrent dispatch calls simultaneously (no delay)
        let orchestrator1 = orchestrator.clone();
        let orchestrator2 = orchestrator.clone();

        let handle1 = tokio::spawn(async move {
            orchestrator1
                .dispatch(SetupEvent::SubmitCreatePassphrase {
                    pass1: "test-passphrase".to_string(),
                    pass2: "test-passphrase".to_string(),
                })
                .await
        });

        let handle2 = tokio::spawn(async move {
            orchestrator2
                .dispatch(SetupEvent::SubmitCreatePassphrase {
                    pass1: "test-passphrase".to_string(),
                    pass2: "test-passphrase".to_string(),
                })
                .await
        });

        // Wait for both to complete
        let result1 = handle1.await.unwrap();
        let result2 = handle2.await.unwrap();

        // At least one should succeed (the first one to execute)
        assert!(
            result1.is_ok() || result2.is_ok(),
            "At least one dispatch should succeed"
        );

        // Key material store should only be called once (not twice)
        // This verifies that the actions were not duplicated
        let store_count = key_material.store_count.load(Ordering::SeqCst);
        assert_eq!(
            store_count, 1,
            "store_keyslot should be called exactly once, but was called {} times. \
             This indicates concurrent dispatch calls executed duplicate actions.",
            store_count
        );

        // Session set_master_key should only be called once
        let set_count = session.set_count.load(Ordering::SeqCst);
        assert_eq!(
            set_count, 1,
            "set_master_key should be called exactly once, but was called {} times.",
            set_count
        );
    }
}
