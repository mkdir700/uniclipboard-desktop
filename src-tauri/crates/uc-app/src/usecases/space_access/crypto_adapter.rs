use std::sync::Arc;

use async_trait::async_trait;
use rand::rngs::OsRng;
use rand::RngCore;
use tracing::{debug, error, info, info_span, warn, Instrument};

use uc_core::ids::SpaceId;
use uc_core::ports::security::encryption_state::EncryptionStatePort;
use uc_core::ports::security::key_scope::{KeyScopePort, ScopeError};
use uc_core::ports::space::CryptoPort;
use uc_core::ports::{EncryptionPort, EncryptionSessionPort, KeyMaterialPort};
use uc_core::security::model::{
    EncryptionAlgo, EncryptionError, KeySlot, MasterKey, Passphrase, WrappedMasterKey,
};
use uc_core::security::state::{EncryptionState, EncryptionStateError};
use uc_core::security::SecretString;

use super::SpaceAccessCryptoFactory;

#[derive(Debug, thiserror::Error)]
pub enum SpaceAccessCryptoError {
    #[error("encryption is already initialized")]
    AlreadyInitialized,
    #[error("failed to resolve key scope")]
    ScopeFailed(#[from] ScopeError),
    #[error("encryption failed: {0}")]
    EncryptionFailed(#[from] EncryptionError),
    #[error("failed to persist encryption state")]
    StatePersistenceFailed(#[from] EncryptionStateError),
}

pub struct SpaceAccessCryptoAdapter {
    passphrase: SecretString,
    encryption: Arc<dyn EncryptionPort>,
    key_material: Arc<dyn KeyMaterialPort>,
    key_scope: Arc<dyn KeyScopePort>,
    encryption_state: Arc<dyn EncryptionStatePort>,
    encryption_session: Arc<dyn EncryptionSessionPort>,
}

impl SpaceAccessCryptoAdapter {
    pub fn new(
        passphrase: SecretString,
        encryption: Arc<dyn EncryptionPort>,
        key_material: Arc<dyn KeyMaterialPort>,
        key_scope: Arc<dyn KeyScopePort>,
        encryption_state: Arc<dyn EncryptionStatePort>,
        encryption_session: Arc<dyn EncryptionSessionPort>,
    ) -> Self {
        Self {
            passphrase,
            encryption,
            key_material,
            key_scope,
            encryption_state,
            encryption_session,
        }
    }
}

pub struct DefaultSpaceAccessCryptoFactory {
    encryption: Arc<dyn EncryptionPort>,
    key_material: Arc<dyn KeyMaterialPort>,
    key_scope: Arc<dyn KeyScopePort>,
    encryption_state: Arc<dyn EncryptionStatePort>,
    encryption_session: Arc<dyn EncryptionSessionPort>,
}

impl DefaultSpaceAccessCryptoFactory {
    pub fn new(
        encryption: Arc<dyn EncryptionPort>,
        key_material: Arc<dyn KeyMaterialPort>,
        key_scope: Arc<dyn KeyScopePort>,
        encryption_state: Arc<dyn EncryptionStatePort>,
        encryption_session: Arc<dyn EncryptionSessionPort>,
    ) -> Self {
        Self {
            encryption,
            key_material,
            key_scope,
            encryption_state,
            encryption_session,
        }
    }
}

impl SpaceAccessCryptoFactory for DefaultSpaceAccessCryptoFactory {
    fn build(&self, passphrase: SecretString) -> Box<dyn CryptoPort> {
        Box::new(SpaceAccessCryptoAdapter::new(
            passphrase,
            self.encryption.clone(),
            self.key_material.clone(),
            self.key_scope.clone(),
            self.encryption_state.clone(),
            self.encryption_session.clone(),
        ))
    }
}

#[async_trait]
impl CryptoPort for SpaceAccessCryptoAdapter {
    async fn generate_nonce32(&self) -> [u8; 32] {
        let mut nonce = [0u8; 32];
        OsRng.fill_bytes(&mut nonce);
        nonce
    }

    async fn export_keyslot_blob(&self, _space_id: &SpaceId) -> anyhow::Result<KeySlot> {
        let span = info_span!("usecase.space_access.export_keyslot_blob");
        async {
            info!("Starting new space keyslot creation");

            let state = self.encryption_state.load_state().await?;
            debug!(state = ?state, "Loaded encryption state");
            if state == EncryptionState::Initialized {
                return Err(SpaceAccessCryptoError::AlreadyInitialized.into());
            }

            let scope = self.key_scope.current_scope().await?;
            debug!(scope = %scope.to_identifier(), "Got key scope");

            let keyslot_draft = KeySlot::draft_v1(scope.clone())?;
            debug!("Keyslot draft created");

            let passphrase = Passphrase(self.passphrase.expose().to_string());
            let kek = self
                .encryption
                .derive_kek(&passphrase, &keyslot_draft.salt, &keyslot_draft.kdf)
                .await?;
            debug!("KEK derived");

            let master_key = MasterKey::generate()?;
            debug!("Master key generated");

            let blob = self
                .encryption
                .wrap_master_key(&kek, &master_key, EncryptionAlgo::XChaCha20Poly1305)
                .await?;
            debug!("Master key wrapped");

            let keyslot = keyslot_draft.finalize(WrappedMasterKey { blob });

            if let Err(e) = self.key_material.store_kek(&scope, &kek).await {
                error!(error = %e, "store_kek failed");
                return Err(e.into());
            }

            if let Err(e) = self.key_material.store_keyslot(&keyslot).await {
                error!(error = %e, "store_keyslot failed");
                if let Err(err) = self.key_material.delete_keyslot(&scope).await {
                    warn!(error = %err, "rollback delete_keyslot failed");
                }
                if let Err(err) = self.key_material.delete_kek(&scope).await {
                    warn!(error = %err, "rollback delete_kek failed");
                }
                return Err(e.into());
            }

            if let Err(e) = self.encryption_session.set_master_key(master_key).await {
                error!(error = %e, "set_master_key failed");
                if let Err(err) = self.key_material.delete_keyslot(&scope).await {
                    warn!(error = %err, "rollback delete_keyslot failed");
                }
                if let Err(err) = self.key_material.delete_kek(&scope).await {
                    warn!(error = %err, "rollback delete_kek failed");
                }
                return Err(e.into());
            }

            if let Err(e) = self.encryption_state.persist_initialized().await {
                error!(error = %e, "persist_initialized failed");
                if let Err(err) = self.encryption_session.clear().await {
                    warn!(error = %err, "rollback clear master key failed");
                }
                if let Err(err) = self.key_material.delete_keyslot(&scope).await {
                    warn!(error = %err, "rollback delete_keyslot failed");
                }
                if let Err(err) = self.key_material.delete_kek(&scope).await {
                    warn!(error = %err, "rollback delete_kek failed");
                }
                return Err(e.into());
            }

            info!("New space keyslot stored");
            Ok(keyslot)
        }
        .instrument(span)
        .await
    }

    async fn derive_master_key_from_keyslot(
        &self,
        keyslot_blob: &[u8],
        passphrase: SecretString,
    ) -> anyhow::Result<MasterKey> {
        let span = info_span!("usecase.space_access.derive_master_key_from_keyslot");
        async {
            info!("Deriving master key from keyslot blob");

            let keyslot: KeySlot = serde_json::from_slice(keyslot_blob)
                .map_err(|_| EncryptionError::CorruptedKeySlot)?;
            let scope = keyslot.scope.clone();
            debug!(scope = %scope.to_identifier(), "Parsed keyslot from blob");

            let wrapped_master_key = keyslot
                .wrapped_master_key
                .as_ref()
                .ok_or(EncryptionError::CorruptedKeySlot)?;

            let passphrase = Passphrase(passphrase.expose().to_string());
            let kek = self
                .encryption
                .derive_kek(&passphrase, &keyslot.salt, &keyslot.kdf)
                .await?;
            debug!("KEK derived from passphrase and keyslot");

            if let Err(e) = self.key_material.store_kek(&scope, &kek).await {
                error!(error = %e, "store_kek failed");
                return Err(e.into());
            }

            if let Err(e) = self.key_material.store_keyslot(&keyslot).await {
                error!(error = %e, "store_keyslot failed");
                if let Err(err) = self.key_material.delete_keyslot(&scope).await {
                    warn!(error = %err, "rollback delete_keyslot failed");
                }
                if let Err(err) = self.key_material.delete_kek(&scope).await {
                    warn!(error = %err, "rollback delete_kek failed");
                }
                return Err(e.into());
            }

            let master_key = match self
                .encryption
                .unwrap_master_key(&kek, &wrapped_master_key.blob)
                .await
            {
                Ok(master_key) => master_key,
                Err(e) => {
                    error!(error = %e, "unwrap_master_key failed");
                    if let Err(err) = self.key_material.delete_keyslot(&scope).await {
                        warn!(error = %err, "rollback delete_keyslot failed");
                    }
                    if let Err(err) = self.key_material.delete_kek(&scope).await {
                        warn!(error = %err, "rollback delete_kek failed");
                    }
                    return Err(e.into());
                }
            };
            debug!("Master key unwrapped");

            if let Err(e) = self
                .encryption_session
                .set_master_key(master_key.clone())
                .await
            {
                error!(error = %e, "set_master_key failed");
                if let Err(err) = self.key_material.delete_keyslot(&scope).await {
                    warn!(error = %err, "rollback delete_keyslot failed");
                }
                if let Err(err) = self.key_material.delete_kek(&scope).await {
                    warn!(error = %err, "rollback delete_kek failed");
                }
                return Err(e.into());
            }

            if let Err(e) = self.encryption_state.persist_initialized().await {
                error!(error = %e, "persist_initialized failed");
                if let Err(err) = self.encryption_session.clear().await {
                    warn!(error = %err, "rollback clear master key failed");
                }
                if let Err(err) = self.key_material.delete_keyslot(&scope).await {
                    warn!(error = %err, "rollback delete_keyslot failed");
                }
                if let Err(err) = self.key_material.delete_kek(&scope).await {
                    warn!(error = %err, "rollback delete_kek failed");
                }
                return Err(e.into());
            }

            info!("Master key derivation completed");
            Ok(master_key)
        }
        .instrument(span)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};
    use uc_core::security::model::{
        EncryptedBlob, EncryptionAlgo, EncryptionError, EncryptionFormatVersion, KdfParams, Kek,
        KeyScope,
    };

    struct EncryptionPortState {
        unwrapped_master_key: Option<MasterKey>,
    }

    struct TestEncryptionPort {
        state: Arc<Mutex<EncryptionPortState>>,
    }

    impl TestEncryptionPort {
        fn new(unwrapped_master_key: Option<MasterKey>) -> (Self, Arc<Mutex<EncryptionPortState>>) {
            let state = Arc::new(Mutex::new(EncryptionPortState {
                unwrapped_master_key,
            }));
            (
                Self {
                    state: state.clone(),
                },
                state,
            )
        }
    }

    #[async_trait]
    impl EncryptionPort for TestEncryptionPort {
        async fn derive_kek(
            &self,
            _passphrase: &Passphrase,
            _salt: &[u8],
            _kdf: &KdfParams,
        ) -> Result<Kek, EncryptionError> {
            Ok(Kek([3u8; 32]))
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
                ciphertext: vec![1u8; 32],
                aad_fingerprint: None,
            })
        }

        async fn unwrap_master_key(
            &self,
            _kek: &Kek,
            _wrapped: &EncryptedBlob,
        ) -> Result<MasterKey, EncryptionError> {
            let guard = self.state.lock().expect("lock encryption port state");
            match &guard.unwrapped_master_key {
                Some(master_key) => Ok(master_key.clone()),
                None => Err(EncryptionError::KeyMaterialCorrupt),
            }
        }

        async fn encrypt_blob(
            &self,
            _master_key: &MasterKey,
            _plaintext: &[u8],
            _aad: &[u8],
            _aead: EncryptionAlgo,
        ) -> Result<EncryptedBlob, EncryptionError> {
            Err(EncryptionError::KeyMaterialCorrupt)
        }

        async fn decrypt_blob(
            &self,
            _master_key: &MasterKey,
            _encrypted: &EncryptedBlob,
            _aad: &[u8],
        ) -> Result<Vec<u8>, EncryptionError> {
            Err(EncryptionError::KeyMaterialCorrupt)
        }
    }

    struct TestKeyScopePort;

    #[async_trait]
    impl KeyScopePort for TestKeyScopePort {
        async fn current_scope(&self) -> Result<KeyScope, ScopeError> {
            Ok(KeyScope {
                profile_id: "profile-test".to_string(),
            })
        }
    }

    struct KeyMaterialState {
        store_kek_called: bool,
        store_keyslot_called: bool,
        store_kek_error: Option<EncryptionError>,
        delete_kek_called: bool,
        delete_keyslot_called: bool,
        store_keyslot_error: Option<EncryptionError>,
    }

    struct TestKeyMaterialPort {
        state: Arc<Mutex<KeyMaterialState>>,
    }

    impl TestKeyMaterialPort {
        fn new(
            store_kek_error: Option<EncryptionError>,
            store_keyslot_error: Option<EncryptionError>,
        ) -> (Self, Arc<Mutex<KeyMaterialState>>) {
            let state = Arc::new(Mutex::new(KeyMaterialState {
                store_kek_called: false,
                store_keyslot_called: false,
                store_kek_error,
                delete_kek_called: false,
                delete_keyslot_called: false,
                store_keyslot_error,
            }));
            (
                Self {
                    state: state.clone(),
                },
                state,
            )
        }
    }

    #[async_trait]
    impl KeyMaterialPort for TestKeyMaterialPort {
        async fn load_kek(&self, _scope: &KeyScope) -> Result<Kek, EncryptionError> {
            Err(EncryptionError::KeyNotFound)
        }

        async fn store_kek(&self, _scope: &KeyScope, _kek: &Kek) -> Result<(), EncryptionError> {
            let mut guard = self.state.lock().expect("lock key material state");
            guard.store_kek_called = true;
            match guard.store_kek_error.take() {
                Some(error) => Err(error),
                None => Ok(()),
            }
        }

        async fn delete_kek(&self, _scope: &KeyScope) -> Result<(), EncryptionError> {
            let mut guard = self.state.lock().expect("lock key material state");
            guard.delete_kek_called = true;
            Ok(())
        }

        async fn load_keyslot(&self, _scope: &KeyScope) -> Result<KeySlot, EncryptionError> {
            Err(EncryptionError::KeyNotFound)
        }

        async fn store_keyslot(&self, _keyslot: &KeySlot) -> Result<(), EncryptionError> {
            let mut guard = self.state.lock().expect("lock key material state");
            guard.store_keyslot_called = true;
            match guard.store_keyslot_error.take() {
                Some(error) => Err(error),
                None => Ok(()),
            }
        }

        async fn delete_keyslot(&self, _scope: &KeyScope) -> Result<(), EncryptionError> {
            let mut guard = self.state.lock().expect("lock key material state");
            guard.delete_keyslot_called = true;
            Ok(())
        }
    }

    struct EncryptionStateState {
        persist_initialized_called: bool,
        persist_initialized_error: Option<EncryptionStateError>,
    }

    struct TestEncryptionStatePort {
        state: Arc<Mutex<EncryptionStateState>>,
    }

    impl TestEncryptionStatePort {
        fn new(
            persist_initialized_error: Option<EncryptionStateError>,
        ) -> (Self, Arc<Mutex<EncryptionStateState>>) {
            let state = Arc::new(Mutex::new(EncryptionStateState {
                persist_initialized_called: false,
                persist_initialized_error,
            }));
            (
                Self {
                    state: state.clone(),
                },
                state,
            )
        }
    }

    #[async_trait]
    impl EncryptionStatePort for TestEncryptionStatePort {
        async fn load_state(&self) -> Result<EncryptionState, EncryptionStateError> {
            Ok(EncryptionState::Uninitialized)
        }

        async fn persist_initialized(&self) -> Result<(), EncryptionStateError> {
            let mut guard = self.state.lock().expect("lock encryption state");
            guard.persist_initialized_called = true;
            match guard.persist_initialized_error.take() {
                Some(error) => Err(error),
                None => Ok(()),
            }
        }
    }

    struct EncryptionSessionState {
        set_master_key_called: bool,
        clear_called: bool,
        set_master_key_error: Option<EncryptionError>,
    }

    struct TestEncryptionSessionPort {
        state: Arc<Mutex<EncryptionSessionState>>,
    }

    impl TestEncryptionSessionPort {
        fn new(
            set_master_key_error: Option<EncryptionError>,
        ) -> (Self, Arc<Mutex<EncryptionSessionState>>) {
            let state = Arc::new(Mutex::new(EncryptionSessionState {
                set_master_key_called: false,
                clear_called: false,
                set_master_key_error,
            }));
            (
                Self {
                    state: state.clone(),
                },
                state,
            )
        }
    }

    #[async_trait]
    impl EncryptionSessionPort for TestEncryptionSessionPort {
        async fn is_ready(&self) -> bool {
            false
        }

        async fn get_master_key(&self) -> Result<MasterKey, EncryptionError> {
            Err(EncryptionError::KeyNotFound)
        }

        async fn set_master_key(&self, _master_key: MasterKey) -> Result<(), EncryptionError> {
            let mut guard = self.state.lock().expect("lock encryption session");
            guard.set_master_key_called = true;
            match guard.set_master_key_error.take() {
                Some(error) => Err(error),
                None => Ok(()),
            }
        }

        async fn clear(&self) -> Result<(), EncryptionError> {
            let mut guard = self.state.lock().expect("lock encryption session");
            guard.clear_called = true;
            Ok(())
        }
    }

    #[tokio::test]
    async fn space_access_keychain_rollback_on_keyslot_failure() {
        let (encryption, _) = TestEncryptionPort::new(None);
        let (key_material, state) =
            TestKeyMaterialPort::new(None, Some(EncryptionError::IoFailure));
        let (encryption_state, _) = TestEncryptionStatePort::new(None);
        let (encryption_session, _) = TestEncryptionSessionPort::new(None);
        let adapter = SpaceAccessCryptoAdapter::new(
            SecretString::from("passphrase"),
            Arc::new(encryption),
            Arc::new(key_material),
            Arc::new(TestKeyScopePort),
            Arc::new(encryption_state),
            Arc::new(encryption_session),
        );

        let result = adapter.export_keyslot_blob(&SpaceId::new()).await;

        assert!(result.is_err());
        let guard = state.lock().expect("lock key material state");
        assert!(guard.delete_kek_called, "expected KEK rollback");
        assert!(guard.delete_keyslot_called, "expected keyslot cleanup");
    }

    #[tokio::test]
    async fn derive_master_key_from_keyslot_succeeds_and_persists_state() {
        let expected_master_key = MasterKey::from_bytes(&[9u8; 32]).expect("valid master key");
        let (encryption, _) = TestEncryptionPort::new(Some(expected_master_key.clone()));
        let (key_material, key_material_state) = TestKeyMaterialPort::new(None, None);
        let (encryption_state, encryption_state_state) = TestEncryptionStatePort::new(None);
        let (encryption_session, encryption_session_state) = TestEncryptionSessionPort::new(None);

        let keyslot = KeySlot::draft_v1(KeyScope {
            profile_id: "profile-joiner".to_string(),
        })
        .expect("draft keyslot")
        .finalize(WrappedMasterKey {
            blob: EncryptedBlob {
                version: EncryptionFormatVersion::V1,
                aead: EncryptionAlgo::XChaCha20Poly1305,
                nonce: vec![0u8; 24],
                ciphertext: vec![1u8; 32],
                aad_fingerprint: None,
            },
        });
        let keyslot_blob = serde_json::to_vec(&keyslot).expect("serialize keyslot");

        let adapter = SpaceAccessCryptoAdapter::new(
            SecretString::from("unused"),
            Arc::new(encryption),
            Arc::new(key_material),
            Arc::new(TestKeyScopePort),
            Arc::new(encryption_state),
            Arc::new(encryption_session),
        );

        let result = adapter
            .derive_master_key_from_keyslot(&keyslot_blob, SecretString::from("joiner-pass"))
            .await;

        assert!(result.is_ok(), "expected key derivation success");
        assert_eq!(
            result.expect("master key").as_bytes(),
            expected_master_key.as_bytes()
        );

        let km_guard = key_material_state.lock().expect("lock key material state");
        assert!(km_guard.store_kek_called, "expected KEK to be stored");
        assert!(
            km_guard.store_keyslot_called,
            "expected keyslot to be stored"
        );
        drop(km_guard);

        let session_guard = encryption_session_state
            .lock()
            .expect("lock encryption session state");
        assert!(
            session_guard.set_master_key_called,
            "expected session master key to be set"
        );
        drop(session_guard);

        let state_guard = encryption_state_state
            .lock()
            .expect("lock encryption state state");
        assert!(
            state_guard.persist_initialized_called,
            "expected encryption initialization to be persisted"
        );
    }

    #[tokio::test]
    async fn derive_master_key_from_keyslot_rolls_back_when_persist_initialized_fails() {
        let expected_master_key = MasterKey::from_bytes(&[8u8; 32]).expect("valid master key");
        let (encryption, _) = TestEncryptionPort::new(Some(expected_master_key));
        let (key_material, key_material_state) = TestKeyMaterialPort::new(None, None);
        let (encryption_state, _) =
            TestEncryptionStatePort::new(Some(EncryptionStateError::PersistError("boom".into())));
        let (encryption_session, encryption_session_state) = TestEncryptionSessionPort::new(None);

        let keyslot = KeySlot::draft_v1(KeyScope {
            profile_id: "profile-joiner".to_string(),
        })
        .expect("draft keyslot")
        .finalize(WrappedMasterKey {
            blob: EncryptedBlob {
                version: EncryptionFormatVersion::V1,
                aead: EncryptionAlgo::XChaCha20Poly1305,
                nonce: vec![0u8; 24],
                ciphertext: vec![2u8; 32],
                aad_fingerprint: None,
            },
        });
        let keyslot_blob = serde_json::to_vec(&keyslot).expect("serialize keyslot");

        let adapter = SpaceAccessCryptoAdapter::new(
            SecretString::from("unused"),
            Arc::new(encryption),
            Arc::new(key_material),
            Arc::new(TestKeyScopePort),
            Arc::new(encryption_state),
            Arc::new(encryption_session),
        );

        let result = adapter
            .derive_master_key_from_keyslot(&keyslot_blob, SecretString::from("joiner-pass"))
            .await;

        assert!(result.is_err(), "expected derive failure");
        let km_guard = key_material_state.lock().expect("lock key material state");
        assert!(km_guard.delete_kek_called, "expected KEK rollback");
        assert!(km_guard.delete_keyslot_called, "expected keyslot rollback");
        drop(km_guard);

        let session_guard = encryption_session_state
            .lock()
            .expect("lock encryption session state");
        assert!(
            session_guard.clear_called,
            "expected encryption session clear rollback"
        );
    }
}
