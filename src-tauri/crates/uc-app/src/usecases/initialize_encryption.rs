use std::sync::Arc;
use tracing::{info_span, info, Instrument};

use uc_core::{
    ports::{
        security::{
            encryption_state::EncryptionStatePort,
            key_scope::{KeyScopePort, ScopeError},
        },
        EncryptionPort, EncryptionSessionPort, KeyMaterialPort,
    },
    security::{
        model::{EncryptionAlgo, EncryptionError, KeySlot, MasterKey, Passphrase, WrappedMasterKey},
        state::{EncryptionState, EncryptionStateError},
    },
};

const LOG_CONTEXT: &str = "[InitializeEncryption]";

#[derive(Debug, thiserror::Error)]
pub enum InitializeEncryptionError {
    #[error("encryption is already initialized")]
    AlreadyInitialized,

    #[error("failed to encrypt master key")]
    EncryptionFailed(#[from] EncryptionError),

    #[error("failed to persist encryption state")]
    StatePersistenceFailed(#[from] EncryptionStateError),

    #[error("failed to resolve key scope")]
    ScopeFailed(#[from] ScopeError),
}

/// Use case for initializing encryption with a passphrase.
///
/// ## Architecture / 架构
///
/// This use case uses **trait objects** (`dyn Port`) instead of generic type parameters.
/// This is the recommended pattern for use cases in the uc-app layer:
///
/// - **Type stability**: The use case has a concrete type, not a generic one
/// - **Easy testing**: Can easily mock ports in tests
/// - **Bootstrap simplicity**: UseCases accessor can instantiate this with Arc<dyn Port>
///
/// 此用例使用 **trait 对象** (`dyn Port`) 而不是泛型类型参数。
/// 这是 uc-app 层用例的推荐模式：
///
/// - **类型稳定性**：用例具有具体类型，而不是泛型类型
/// - **易于测试**：可以轻松在测试中模拟端口
/// - **装配简单性**：UseCases 访问器可以用 Arc<dyn Port> 实例化此用例
///
/// ## Trade-offs / 权衡
///
/// - **Pros**: Clean separation, type stability, easier DI
/// - **Cons**: Slight runtime overhead from dynamic dispatch (negligible for I/O-bound operations)
///
/// ## 优势**：清晰的分离、类型稳定性、更容易的依赖注入
/// ## **劣势**：动态分发带来的轻微运行时开销（对于 I/O 密集型操作可忽略不计）
pub struct InitializeEncryption {
    encryption: Arc<dyn EncryptionPort>,
    key_material: Arc<dyn KeyMaterialPort>,
    key_scope: Arc<dyn KeyScopePort>,
    encryption_state_repo: Arc<dyn EncryptionStatePort>,
    encryption_session: Arc<dyn EncryptionSessionPort>,
}

impl InitializeEncryption {
    /// Create a new InitializeEncryption use case from trait objects.
    /// 从 trait 对象创建新的 InitializeEncryption 用例。
    ///
    /// This follows the `dyn Port` pattern recommended for uc-app use cases.
    /// 遵循 uc-app 用例推荐的 `dyn Port` 模式。
    pub fn new(
        encryption: Arc<dyn EncryptionPort>,
        key_material: Arc<dyn KeyMaterialPort>,
        key_scope: Arc<dyn KeyScopePort>,
        encryption_state_repo: Arc<dyn EncryptionStatePort>,
        encryption_session: Arc<dyn EncryptionSessionPort>,
    ) -> Self {
        Self {
            encryption,
            key_material,
            key_scope,
            encryption_state_repo,
            encryption_session,
        }
    }

    /// Create a new InitializeEncryption use case from cloned Arc<dyn Port> references.
    /// 从克隆的 Arc<dyn Port> 引用创建新的 InitializeEncryption 用例。
    ///
    /// This is a convenience method for the UseCases accessor pattern.
    /// 这是 UseCases 访问器模式的便捷方法。
    pub fn from_ports(
        encryption: Arc<dyn EncryptionPort>,
        key_material: Arc<dyn KeyMaterialPort>,
        key_scope: Arc<dyn KeyScopePort>,
        encryption_state_repo: Arc<dyn EncryptionStatePort>,
        encryption_session: Arc<dyn EncryptionSessionPort>,
    ) -> Self {
        Self::new(encryption, key_material, key_scope, encryption_state_repo, encryption_session)
    }

    pub async fn execute(&self, passphrase: Passphrase) -> Result<(), InitializeEncryptionError> {
        let span = info_span!("usecase.initialize_encryption.execute");

        async {
            info!("Starting encryption initialization");

            let state = self.encryption_state_repo.load_state().await?;
            log::debug!("{} Loaded encryption state: {:?}", LOG_CONTEXT, state);

            // 1. assert not initialized
            if state == EncryptionState::Initialized {
                return Err(InitializeEncryptionError::AlreadyInitialized);
            }

            log::debug!("{} Getting current scope...", LOG_CONTEXT);
            let scope = self.key_scope.current_scope().await?;
            log::debug!("{} Got scope: {}", LOG_CONTEXT, scope.to_identifier());

            log::debug!("{} Creating keyslot draft...", LOG_CONTEXT);
            let keyslot_draft = KeySlot::draft_v1(scope.clone())?;
            log::debug!("{} Keyslot draft created", LOG_CONTEXT);

            // 2. derive KEK
            log::debug!("{} Deriving KEK...", LOG_CONTEXT);
            let kek = self
                .encryption
                .derive_kek(&passphrase, &keyslot_draft.salt, &keyslot_draft.kdf)
                .await?;
            log::debug!("{} KEK derived successfully", LOG_CONTEXT);

            // 3. generate MasterKey
            log::debug!("{} Generating master key...", LOG_CONTEXT);
            let master_key = MasterKey::generate()?;
            log::debug!("{} Master key generated", LOG_CONTEXT);

            // 4. wrap MasterKey
            log::debug!("{} Wrapping master key...", LOG_CONTEXT);
            let blob = self
                .encryption
                .wrap_master_key(&kek, &master_key, EncryptionAlgo::XChaCha20Poly1305)
                .await?;
            log::debug!("{} Master key wrapped successfully", LOG_CONTEXT);

            let keyslot = keyslot_draft.finalize(WrappedMasterKey { blob });
            log::debug!("{} Keyslot finalized", LOG_CONTEXT);

            // 5. persist wrapped key, store keyslot
            log::debug!("{} Storing keyslot...", LOG_CONTEXT);
            self.key_material.store_keyslot(&keyslot).await?;
            log::debug!("{} Keyslot stored successfully", LOG_CONTEXT);

            // 6. store KEK material into keyring
            log::debug!("{} Storing KEK in keyring...", LOG_CONTEXT);
            self.key_material.store_kek(&scope, &kek).await?;
            log::debug!("{} KEK stored successfully", LOG_CONTEXT);

            // 7. persist initialized state
            log::debug!("{} Persisting initialized state...", LOG_CONTEXT);
            self.encryption_state_repo.persist_initialized().await?;
            log::debug!("{} Encryption state persisted", LOG_CONTEXT);

            // 8. set master key in session for immediate use
            log::debug!("{} Setting master key in session...", LOG_CONTEXT);
            self.encryption_session.set_master_key(master_key).await?;
            log::debug!("{} Master key set in session successfully", LOG_CONTEXT);

            info!("Encryption initialized successfully");
            Ok(())
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
        security::model::{KeyScope, Kek, EncryptedBlob, EncryptionFormatVersion, EncryptionAlgo},
        security::state::EncryptionStateError,
    };
    use async_trait::async_trait;

    /// Mock EncryptionStatePort
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
    }

    #[async_trait]
    impl KeyScopePort for MockKeyScope {
        async fn current_scope(&self) -> Result<KeyScope, ScopeError> {
            self.scope.clone().ok_or(ScopeError::FailedToGetCurrentScope)
        }
    }

    /// Mock KeyMaterialPort
    struct MockKeyMaterial {
        kek_stored: Arc<std::sync::atomic::AtomicBool>,
        keyslot_stored: Arc<std::sync::atomic::AtomicBool>,
    }

    impl MockKeyMaterial {
        fn new() -> Self {
            Self {
                kek_stored: Arc::new(std::sync::atomic::AtomicBool::new(false)),
                keyslot_stored: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            }
        }

        fn was_kek_stored(&self) -> bool {
            self.kek_stored.load(std::sync::atomic::Ordering::SeqCst)
        }

        fn was_keyslot_stored(&self) -> bool {
            self.keyslot_stored.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl KeyMaterialPort for MockKeyMaterial {
        async fn load_keyslot(&self, _scope: &KeyScope) -> Result<KeySlot, EncryptionError> {
            Err(EncryptionError::KeyNotFound)
        }

        async fn store_keyslot(&self, _keyslot: &KeySlot) -> Result<(), EncryptionError> {
            self.keyslot_stored.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }

        async fn delete_keyslot(&self, _scope: &KeyScope) -> Result<(), EncryptionError> {
            Ok(())
        }

        async fn load_kek(&self, _scope: &KeyScope) -> Result<Kek, EncryptionError> {
            Err(EncryptionError::KeyNotFound)
        }

        async fn store_kek(&self, _scope: &KeyScope, _kek: &Kek) -> Result<(), EncryptionError> {
            self.kek_stored.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }

        async fn delete_kek(&self, _scope: &KeyScope) -> Result<(), EncryptionError> {
            Ok(())
        }
    }

    /// Mock EncryptionPort
    struct MockEncryption {
        should_fail: bool,
    }

    impl MockEncryption {
        fn new() -> Self {
            Self { should_fail: false }
        }
    }

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
            if self.should_fail {
                return Err(EncryptionError::EncryptFailed);
            }
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

    /// Mock EncryptionSessionPort
    struct MockEncryptionSession {
        master_key_set: Arc<std::sync::atomic::AtomicBool>,
    }

    impl MockEncryptionSession {
        fn new() -> Self {
            Self {
                master_key_set: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            }
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
            self.master_key_set.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }

        async fn clear(&self) -> Result<(), EncryptionError> {
            self.master_key_set.store(false, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_initialize_encryption_sets_master_key_in_session() {
        // Test that initialization sets the master key in the session
        let state = Arc::new(MockEncryptionState::new(EncryptionState::Uninitialized));
        let scope = Arc::new(MockKeyScope::succeed_with(KeyScope {
            profile_id: "test".to_string(),
        }));
        let key_material = Arc::new(MockKeyMaterial::new());
        let encryption = Arc::new(MockEncryption::new());
        let session = Arc::new(MockEncryptionSession::new());

        let use_case = InitializeEncryption::new(
            encryption,
            key_material,
            scope,
            state,
            session.clone(),
        );

        let passphrase = Passphrase("test-password".to_string());
        let result = use_case.execute(passphrase).await;

        assert!(result.is_ok(), "initialization should succeed");
        assert!(session.was_master_key_set(), "master key should be set in session");
    }

    #[tokio::test]
    async fn test_initialize_encryption_fails_when_already_initialized() {
        // Test that initialization fails when already initialized
        let state = Arc::new(MockEncryptionState::new(EncryptionState::Initialized));
        let scope = Arc::new(MockKeyScope::succeed_with(KeyScope {
            profile_id: "test".to_string(),
        }));
        let key_material = Arc::new(MockKeyMaterial::new());
        let encryption = Arc::new(MockEncryption::new());
        let session = Arc::new(MockEncryptionSession::new());

        let use_case = InitializeEncryption::new(
            encryption,
            key_material,
            scope,
            state,
            session,
        );

        let passphrase = Passphrase("test-password".to_string());
        let result = use_case.execute(passphrase).await;

        assert!(result.is_err(), "initialization should fail");
        let err = result.unwrap_err();
        assert!(matches!(err, InitializeEncryptionError::AlreadyInitialized));
    }

    #[tokio::test]
    async fn test_initialize_encryption_does_not_set_session_on_failure() {
        // Test that session is not set when initialization fails
        let state = Arc::new(MockEncryptionState::new(EncryptionState::Initialized));
        let scope = Arc::new(MockKeyScope::succeed_with(KeyScope {
            profile_id: "test".to_string(),
        }));
        let key_material = Arc::new(MockKeyMaterial::new());
        let encryption = Arc::new(MockEncryption::new());
        let session = Arc::new(MockEncryptionSession::new());

        let use_case = InitializeEncryption::new(
            encryption,
            key_material,
            scope,
            state,
            session.clone(),
        );

        let passphrase = Passphrase("test-password".to_string());
        let _ = use_case.execute(passphrase).await;

        assert!(!session.was_master_key_set(), "master key should NOT be set when initialization fails");
    }

    #[tokio::test]
    async fn test_initialize_encryption_stores_kek_and_keyslot() {
        // Test that both kek and keyslot are stored during initialization
        let state = Arc::new(MockEncryptionState::new(EncryptionState::Uninitialized));
        let scope = Arc::new(MockKeyScope::succeed_with(KeyScope {
            profile_id: "test".to_string(),
        }));
        let key_material = Arc::new(MockKeyMaterial::new());
        let encryption = Arc::new(MockEncryption::new());
        let session = Arc::new(MockEncryptionSession::new());

        let use_case = InitializeEncryption::new(
            encryption,
            key_material.clone(),
            scope,
            state,
            session,
        );

        let passphrase = Passphrase("test-password".to_string());
        let result = use_case.execute(passphrase).await;

        assert!(result.is_ok(), "initialization should succeed");
        assert!(key_material.was_kek_stored(), "kek should be stored");
        assert!(key_material.was_keyslot_stored(), "keyslot should be stored");
    }

    #[tokio::test]
    async fn test_initialize_encryption_does_not_store_keys_on_failure() {
        // Test that keys are not stored when initialization fails
        let state = Arc::new(MockEncryptionState::new(EncryptionState::Initialized));
        let scope = Arc::new(MockKeyScope::succeed_with(KeyScope {
            profile_id: "test".to_string(),
        }));
        let key_material = Arc::new(MockKeyMaterial::new());
        let encryption = Arc::new(MockEncryption::new());
        let session = Arc::new(MockEncryptionSession::new());

        let use_case = InitializeEncryption::new(
            encryption,
            key_material.clone(),
            scope,
            state,
            session,
        );

        let passphrase = Passphrase("test-password".to_string());
        let _ = use_case.execute(passphrase).await;

        assert!(!key_material.was_kek_stored(), "kek should NOT be stored on failure");
        assert!(!key_material.was_keyslot_stored(), "keyslot should NOT be stored on failure");
    }
}
