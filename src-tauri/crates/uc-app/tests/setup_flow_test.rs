use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use uc_app::usecases::SetupOrchestrator;
use uc_core::onboarding::OnboardingState;
use uc_core::ports::security::key_scope::ScopeError;
use uc_core::ports::security::{encryption_state::EncryptionStatePort, key_scope::KeyScopePort};
use uc_core::ports::{EncryptionPort, EncryptionSessionPort, KeyMaterialPort, OnboardingStatePort};
use uc_core::security::model::{
    EncryptedBlob, EncryptionAlgo, EncryptionError, EncryptionFormatVersion, Kek, KeyScope,
    KeySlot, MasterKey, Passphrase,
};
use uc_core::security::state::{EncryptionState, EncryptionStateError};
use uc_core::setup::{SetupEvent, SetupState};

#[tokio::test]
async fn setup_flow_test_create_space_flow_marks_setup_complete() {
    let (orchestrator, onboarding_state, session) = build_orchestrator();

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

    let onboarding = onboarding_state
        .get_state()
        .await
        .expect("get onboarding state");
    assert!(onboarding.has_completed, "onboarding should be complete");
    assert!(session.is_ready().await, "master key should be set");
}

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
    state: Mutex<OnboardingState>,
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

fn build_orchestrator() -> (
    SetupOrchestrator,
    Arc<MockOnboardingStatePort>,
    Arc<MockEncryptionSession>,
) {
    let session = Arc::new(MockEncryptionSession {
        master_key_set: AtomicBool::new(false),
    });
    let onboarding_state = Arc::new(MockOnboardingStatePort {
        state: Mutex::new(OnboardingState::default()),
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
    (orchestrator, onboarding_state, session)
}
