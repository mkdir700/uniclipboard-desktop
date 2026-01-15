use std::sync::Arc;

use uc_core::{
    ports::{
        security::{
            encryption_state::EncryptionStatePort,
            key_scope::{KeyScopePort, ScopeError},
        },
        EncryptionPort, KeyMaterialPort,
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
    ) -> Self {
        Self {
            encryption,
            key_material,
            key_scope,
            encryption_state_repo,
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
    ) -> Self {
        Self::new(encryption, key_material, key_scope, encryption_state_repo)
    }

    pub async fn execute(&self, passphrase: Passphrase) -> Result<(), InitializeEncryptionError> {
        log::debug!("{} Starting execution", LOG_CONTEXT);

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

        log::info!("{} All steps completed successfully", LOG_CONTEXT);
        Ok(())
    }
}
