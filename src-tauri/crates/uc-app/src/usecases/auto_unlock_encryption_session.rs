//! Auto-unlock encryption session on startup.
//!
//! This use case loads the MasterKey from persisted keyslot + KEK
//! and sets it in the EncryptionSessionPort for transparent encryption.

use std::sync::Arc;
use tracing::{info_span, info, Instrument};

use uc_core::{
    ports::{
        security::{
            encryption_state::EncryptionStatePort,
            key_scope::KeyScopePort,
        },
        EncryptionPort, EncryptionSessionPort, KeyMaterialPort,
    },
    security::{
        model::EncryptionError,
        state::EncryptionState,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum AutoUnlockError {
    #[error("encryption state check failed: {0}")]
    StateCheckFailed(String),

    #[error("key scope resolution failed: {0}")]
    ScopeFailed(String),

    #[error("failed to load keyslot: {0}")]
    KeySlotLoadFailed(#[source] EncryptionError),

    #[error("failed to load KEK from keyring: {0}")]
    KekLoadFailed(#[source] EncryptionError),

    #[error("keyslot has no wrapped master key")]
    MissingWrappedMasterKey,

    #[error("failed to unwrap master key: {0}")]
    UnwrapFailed(#[source] EncryptionError),

    #[error("failed to set master key in session: {0}")]
    SessionSetFailed(#[source] EncryptionError),
}

/// Use case for automatically unlocking encryption session on startup.
///
/// ## Behavior
///
/// - If encryption is **Uninitialized**: Returns `Ok(false)` (not unlocked, but not an error)
/// - If encryption is **Initialized**: Attempts to load and set MasterKey, returns `Ok(true)` on success
/// - Any failure during unlock returns an error
pub struct AutoUnlockEncryptionSession {
    encryption_state: Arc<dyn EncryptionStatePort>,
    key_scope: Arc<dyn KeyScopePort>,
    key_material: Arc<dyn KeyMaterialPort>,
    encryption: Arc<dyn EncryptionPort>,
    encryption_session: Arc<dyn EncryptionSessionPort>,
}

impl AutoUnlockEncryptionSession {
    pub fn new(
        encryption_state: Arc<dyn EncryptionStatePort>,
        key_scope: Arc<dyn KeyScopePort>,
        key_material: Arc<dyn KeyMaterialPort>,
        encryption: Arc<dyn EncryptionPort>,
        encryption_session: Arc<dyn EncryptionSessionPort>,
    ) -> Self {
        Self {
            encryption_state,
            key_scope,
            key_material,
            encryption,
            encryption_session,
        }
    }

    pub fn from_ports(
        encryption_state: Arc<dyn EncryptionStatePort>,
        key_scope: Arc<dyn KeyScopePort>,
        key_material: Arc<dyn KeyMaterialPort>,
        encryption: Arc<dyn EncryptionPort>,
        encryption_session: Arc<dyn EncryptionSessionPort>,
    ) -> Self {
        Self::new(encryption_state, key_scope, key_material, encryption, encryption_session)
    }

    /// Execute the auto-unlock flow.
    ///
    /// # Returns
    ///
    /// - `Ok(true)` - Session unlocked successfully
    /// - `Ok(false)` - Encryption not initialized (no unlock needed)
    /// - `Err(_)` - Unlock failed
    pub async fn execute(&self) -> Result<bool, AutoUnlockError> {
        let span = info_span!("usecase.auto_unlock_encryption_session.execute");

        async {
            info!("Checking encryption state for auto-unlock");

            // 1. Check encryption state
            let state = self.encryption_state.load_state().await
                .map_err(|e| AutoUnlockError::StateCheckFailed(e.to_string()))?;

            if state == EncryptionState::Uninitialized {
                info!("Encryption not initialized, skipping auto-unlock");
                return Ok(false);
            }

            info!("Encryption initialized, attempting auto-unlock");

            // 2. Get key scope
            let scope = self.key_scope.current_scope().await
                .map_err(|e| AutoUnlockError::ScopeFailed(e.to_string()))?;

            // 3. Load keyslot
            let keyslot = self.key_material.load_keyslot(&scope).await
                .map_err(AutoUnlockError::KeySlotLoadFailed)?;

            // 4. Get wrapped master key
            let wrapped_master_key = keyslot.wrapped_master_key
                .ok_or(AutoUnlockError::MissingWrappedMasterKey)?;

            // 5. Load KEK from keyring
            let kek = self.key_material.load_kek(&scope).await
                .map_err(AutoUnlockError::KekLoadFailed)?;

            // 6. Unwrap master key
            let master_key = self.encryption.unwrap_master_key(&kek, &wrapped_master_key.blob).await
                .map_err(AutoUnlockError::UnwrapFailed)?;

            // 7. Set master key in session
            self.encryption_session.set_master_key(master_key).await
                .map_err(AutoUnlockError::SessionSetFailed)?;

            info!("Auto-unlock completed successfully");
            Ok(true)
        }
        .instrument(span)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use uc_core::{
        ports::security::key_scope::ScopeError,
        security::{
            model::{KeyScope, MasterKey, Kek, WrappedMasterKey, EncryptedBlob, EncryptionFormatVersion, EncryptionAlgo},
            state::EncryptionStateError,
        },
    };
    use async_trait::async_trait;

    /// Mock EncryptionStatePort that returns a fixed state
    struct MockEncryptionState {
        state: EncryptionState,
    }

    impl MockEncryptionState {
        fn new(state: EncryptionState) -> Self {
            Self { state }
        }
    }

    #[async_trait]
    impl uc_core::ports::security::encryption_state::EncryptionStatePort for MockEncryptionState {
        async fn load_state(&self) -> Result<EncryptionState, EncryptionStateError> {
            Ok(self.state.clone())
        }

        async fn persist_initialized(&self) -> Result<(), EncryptionStateError> {
            Ok(())
        }
    }

    /// Mock KeyScopePort
    struct MockKeyScope {
        scope: Option<KeyScope>,
    }

    impl MockKeyScope {
        fn new(scope: Option<KeyScope>) -> Self {
            Self { scope }
        }

        fn succeed_with(scope: KeyScope) -> Self {
            Self::new(Some(scope))
        }

        fn fail() -> Self {
            Self::new(None)
        }
    }

    #[async_trait]
    impl KeyScopePort for MockKeyScope {
        async fn current_scope(&self) -> Result<KeyScope, ScopeError> {
            self.scope.clone().ok_or(ScopeError::FailedToGetCurrentScope)
        }
    }

    /// Mock KeyMaterialPort
    struct MockKeyMaterial {
        keyslot: Option<uc_core::security::model::KeySlot>,
        kek: Option<Kek>,
    }

    impl MockKeyMaterial {
        fn new() -> Self {
            Self {
                keyslot: None,
                kek: None,
            }
        }

        fn with_keyslot(mut self, keyslot: uc_core::security::model::KeySlot) -> Self {
            self.keyslot = Some(keyslot);
            self
        }

        fn with_kek(mut self, kek: Kek) -> Self {
            self.kek = Some(kek);
            self
        }
    }

    #[async_trait]
    impl uc_core::ports::KeyMaterialPort for MockKeyMaterial {
        async fn load_keyslot(&self, _scope: &KeyScope) -> Result<uc_core::security::model::KeySlot, EncryptionError> {
            self.keyslot.clone().ok_or(EncryptionError::KeyNotFound)
        }

        async fn store_keyslot(&self, _keyslot: &uc_core::security::model::KeySlot) -> Result<(), EncryptionError> {
            Ok(())
        }

        async fn delete_keyslot(&self, _scope: &KeyScope) -> Result<(), EncryptionError> {
            Ok(())
        }

        async fn load_kek(&self, _scope: &KeyScope) -> Result<Kek, EncryptionError> {
            self.kek.clone().ok_or(EncryptionError::KeyNotFound)
        }

        async fn store_kek(&self, _scope: &KeyScope, _kek: &Kek) -> Result<(), EncryptionError> {
            Ok(())
        }

        async fn delete_kek(&self, _scope: &KeyScope) -> Result<(), EncryptionError> {
            Ok(())
        }
    }

    /// Mock EncryptionPort
    struct MockEncryption {
        should_fail_unwrap: bool,
    }

    impl MockEncryption {
        fn new() -> Self {
            Self { should_fail_unwrap: false }
        }

        fn fail_on_unwrap(mut self) -> Self {
            self.should_fail_unwrap = true;
            self
        }
    }

    #[async_trait]
    impl uc_core::ports::EncryptionPort for MockEncryption {
        async fn derive_kek(
            &self,
            _passphrase: &uc_core::security::model::Passphrase,
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
            if self.should_fail_unwrap {
                return Err(EncryptionError::CryptoFailure);
            }
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

    /// Mock EncryptionSessionPort
    struct MockEncryptionSession {
        should_fail_set: bool,
        master_key_set: Arc<std::sync::atomic::AtomicBool>,
    }

    impl MockEncryptionSession {
        fn new() -> Self {
            Self {
                should_fail_set: false,
                master_key_set: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            }
        }

        fn fail_on_set(mut self) -> Self {
            self.should_fail_set = true;
            self
        }

        fn was_master_key_set(&self) -> bool {
            self.master_key_set.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl EncryptionSessionPort for MockEncryptionSession {
        async fn is_ready(&self) -> bool {
            self.master_key_set.load(std::sync::atomic::Ordering::SeqCst)
        }

        async fn get_master_key(&self) -> Result<MasterKey, EncryptionError> {
            if self.master_key_set.load(std::sync::atomic::Ordering::SeqCst) {
                MasterKey::from_bytes(&[0u8; 32])
            } else {
                Err(EncryptionError::Locked)
            }
        }

        async fn set_master_key(&self, _master_key: MasterKey) -> Result<(), EncryptionError> {
            if self.should_fail_set {
                return Err(EncryptionError::CryptoFailure);
            }
            self.master_key_set.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }

        async fn clear(&self) -> Result<(), EncryptionError> {
            self.master_key_set.store(false, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
    }

    /// Creates a valid test keyslot with wrapped master key
    fn create_test_keyslot(scope: KeyScope) -> uc_core::security::model::KeySlot {
        uc_core::security::model::KeySlot {
            version: uc_core::security::model::KeySlotVersion::V1,
            scope,
            kdf: uc_core::security::model::KdfParams::for_initialization(),
            salt: vec![0u8; 16],
            wrapped_master_key: Some(WrappedMasterKey {
                blob: EncryptedBlob {
                    version: EncryptionFormatVersion::V1,
                    aead: EncryptionAlgo::XChaCha20Poly1305,
                    nonce: vec![0u8; 24],
                    ciphertext: vec![0u8; 32],
                    aad_fingerprint: None,
                },
            }),
        }
    }

    /// Creates a test KEK
    fn create_test_kek() -> Kek {
        Kek([0u8; 32])
    }

    #[tokio::test]
    async fn test_auto_unlock_returns_false_when_uninitialized() {
        // When encryption state is Uninitialized, auto-unlock should return Ok(false)
        let state = Arc::new(MockEncryptionState::new(EncryptionState::Uninitialized));
        let scope = Arc::new(MockKeyScope::succeed_with(KeyScope {
            profile_id: "test".to_string(),
        }));
        let key_material = Arc::new(MockKeyMaterial::new());
        let encryption = Arc::new(MockEncryption::new());
        let session = Arc::new(MockEncryptionSession::new());

        let use_case = AutoUnlockEncryptionSession::new(
            state,
            scope,
            key_material,
            encryption,
            session,
        );

        let result = use_case.execute().await;

        assert!(result.is_ok(), "should succeed when uninitialized");
        assert_eq!(result.unwrap(), false, "should return false when uninitialized");
    }

    #[tokio::test]
    async fn test_auto_unlock_succeeds_when_initialized() {
        // When all dependencies succeed, auto-unlock should return Ok(true)
        let scope_value = KeyScope {
            profile_id: "test".to_string(),
        };
        let state = Arc::new(MockEncryptionState::new(EncryptionState::Initialized));
        let scope_port = Arc::new(MockKeyScope::succeed_with(scope_value.clone()));
        let key_material = Arc::new(
            MockKeyMaterial::new()
                .with_keyslot(create_test_keyslot(scope_value))
                .with_kek(create_test_kek()),
        );
        let encryption = Arc::new(MockEncryption::new());
        let session = Arc::new(MockEncryptionSession::new());

        let use_case = AutoUnlockEncryptionSession::new(
            state,
            scope_port,
            key_material,
            encryption,
            session.clone(),
        );

        let result = use_case.execute().await;

        assert!(result.is_ok(), "should succeed when all dependencies succeed");
        assert_eq!(result.unwrap(), true, "should return true on successful unlock");
        assert!(session.was_master_key_set(), "master key should be set in session");
    }

    #[tokio::test]
    async fn test_auto_unlock_propagates_state_check_error() {
        // When state check fails, should return StateCheckFailed error
        struct FailingState;

        #[async_trait]
        impl uc_core::ports::security::encryption_state::EncryptionStatePort for FailingState {
            async fn load_state(&self) -> Result<EncryptionState, EncryptionStateError> {
                Err(EncryptionStateError::LoadError("state check failed".to_string()))
            }

            async fn persist_initialized(&self) -> Result<(), EncryptionStateError> {
                Ok(())
            }
        }

        let state = Arc::new(FailingState);
        let scope = Arc::new(MockKeyScope::succeed_with(KeyScope {
            profile_id: "test".to_string(),
        }));
        let key_material = Arc::new(MockKeyMaterial::new());
        let encryption = Arc::new(MockEncryption::new());
        let session = Arc::new(MockEncryptionSession::new());

        let use_case = AutoUnlockEncryptionSession::new(
            state,
            scope,
            key_material,
            encryption,
            session,
        );

        let result = use_case.execute().await;

        assert!(result.is_err(), "should fail when state check fails");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("encryption state check failed"),
            "error should indicate state check failure: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_auto_unlock_propagates_scope_error() {
        // When scope resolution fails, should return ScopeFailed error
        let state = Arc::new(MockEncryptionState::new(EncryptionState::Initialized));
        let scope = Arc::new(MockKeyScope::fail());
        let key_material = Arc::new(MockKeyMaterial::new());
        let encryption = Arc::new(MockEncryption::new());
        let session = Arc::new(MockEncryptionSession::new());

        let use_case = AutoUnlockEncryptionSession::new(
            state,
            scope,
            key_material,
            encryption,
            session,
        );

        let result = use_case.execute().await;

        assert!(result.is_err(), "should fail when scope resolution fails");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("key scope resolution failed"),
            "error should indicate scope failure: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_auto_unlock_propagates_keyslot_load_error() {
        // When keyslot load fails, should return KeySlotLoadFailed error
        let scope_value = KeyScope {
            profile_id: "test".to_string(),
        };
        let state = Arc::new(MockEncryptionState::new(EncryptionState::Initialized));
        let scope = Arc::new(MockKeyScope::succeed_with(scope_value));
        let key_material = Arc::new(MockKeyMaterial::new()); // No keyslot = fails
        let encryption = Arc::new(MockEncryption::new());
        let session = Arc::new(MockEncryptionSession::new());

        let use_case = AutoUnlockEncryptionSession::new(
            state,
            scope,
            key_material,
            encryption,
            session,
        );

        let result = use_case.execute().await;

        assert!(result.is_err(), "should fail when keyslot load fails");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("failed to load keyslot"),
            "error should indicate keyslot load failure: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_auto_unlock_fails_when_missing_wrapped_master_key() {
        // When keyslot exists but has no wrapped master key, should return MissingWrappedMasterKey
        let scope_value = KeyScope {
            profile_id: "test".to_string(),
        };
        let state = Arc::new(MockEncryptionState::new(EncryptionState::Initialized));
        let scope = Arc::new(MockKeyScope::succeed_with(scope_value));
        let mut keyslot = create_test_keyslot(KeyScope {
            profile_id: "test".to_string(),
        });
        keyslot.wrapped_master_key = None; // Remove wrapped master key
        let key_material = Arc::new(MockKeyMaterial::new().with_keyslot(keyslot));
        let encryption = Arc::new(MockEncryption::new());
        let session = Arc::new(MockEncryptionSession::new());

        let use_case = AutoUnlockEncryptionSession::new(
            state,
            scope,
            key_material,
            encryption,
            session,
        );

        let result = use_case.execute().await;

        assert!(result.is_err(), "should fail when wrapped master key is missing");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("keyslot has no wrapped master key"),
            "error should indicate missing wrapped master key: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_auto_unlock_propagates_unwrap_error() {
        // When unwrap fails, should return UnwrapFailed error
        let scope_value = KeyScope {
            profile_id: "test".to_string(),
        };
        let state = Arc::new(MockEncryptionState::new(EncryptionState::Initialized));
        let scope = Arc::new(MockKeyScope::succeed_with(scope_value));
        let key_material = Arc::new(
            MockKeyMaterial::new()
                .with_keyslot(create_test_keyslot(KeyScope {
                    profile_id: "test".to_string(),
                }))
                .with_kek(create_test_kek()),
        );
        let encryption = Arc::new(MockEncryption::new().fail_on_unwrap());
        let session = Arc::new(MockEncryptionSession::new());

        let use_case = AutoUnlockEncryptionSession::new(
            state,
            scope,
            key_material,
            encryption,
            session,
        );

        let result = use_case.execute().await;

        assert!(result.is_err(), "should fail when unwrap fails");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("failed to unwrap master key"),
            "error should indicate unwrap failure: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_auto_unlock_propagates_session_set_error() {
        // When session set fails, should return SessionSetFailed error
        let scope_value = KeyScope {
            profile_id: "test".to_string(),
        };
        let state = Arc::new(MockEncryptionState::new(EncryptionState::Initialized));
        let scope = Arc::new(MockKeyScope::succeed_with(scope_value));
        let key_material = Arc::new(
            MockKeyMaterial::new()
                .with_keyslot(create_test_keyslot(KeyScope {
                    profile_id: "test".to_string(),
                }))
                .with_kek(create_test_kek()),
        );
        let encryption = Arc::new(MockEncryption::new());
        let session = Arc::new(MockEncryptionSession::new().fail_on_set());

        let use_case = AutoUnlockEncryptionSession::new(
            state,
            scope,
            key_material,
            encryption,
            session,
        );

        let result = use_case.execute().await;

        assert!(result.is_err(), "should fail when session set fails");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("failed to set master key in session"),
            "error should indicate session set failure: {}",
            err
        );
    }
}
