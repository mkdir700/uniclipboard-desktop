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
        model::{AeadAlgorithm, EncryptionError, KeySlot, MasterKey, Passphrase, WrappedMasterKey},
        state::{EncryptionState, EncryptionStateError},
    },
};

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

pub struct InitializeEncryption<E, K, KS, ES>
where
    E: EncryptionPort,
    K: KeyMaterialPort,
    KS: KeyScopePort,
    ES: EncryptionStatePort,
{
    encryption: Arc<E>,
    key_material: Arc<K>,
    key_scope: Arc<KS>,
    encryption_state_repo: Arc<ES>,
}

impl<E, K, KS, ES> InitializeEncryption<E, K, KS, ES>
where
    E: EncryptionPort,
    K: KeyMaterialPort,
    KS: KeyScopePort,
    ES: EncryptionStatePort,
{
    pub async fn execute(&self, passphrase: Passphrase) -> Result<(), InitializeEncryptionError> {
        let state = self.encryption_state_repo.load_state().await?;

        // 1. assert not initialized
        if state == EncryptionState::Initialized {
            return Err(InitializeEncryptionError::AlreadyInitialized);
        }

        let scope = self.key_scope.current_scope().await?;
        let keyslot_draft = KeySlot::draft_v1(scope.clone())?;

        // 2. derive KEK
        let kek = self
            .encryption
            .derive_kek(&passphrase, &keyslot_draft.salt, &keyslot_draft.kdf)
            .await?;

        // 3. generate MasterKey
        let master_key = MasterKey::generate()?;

        // 4. wrap MasterKey
        let blob = self
            .encryption
            .wrap_master_key(&kek, &master_key, AeadAlgorithm::XChaCha20Poly1305)
            .await?;

        let keyslot = keyslot_draft.finalize(WrappedMasterKey { blob });

        // 5. persist wrapped key, store keyslot
        self.key_material.store_keyslot(&keyslot).await?;

        // 6. store KEK material into keyring
        self.key_material.store_kek(&scope, &kek).await?;

        // 7. persist initialized state
        self.encryption_state_repo.persist_initialized().await?;

        Ok(())
    }
}
