use async_trait::async_trait;
use uc_core::{
    ports::{KeyMaterialPort, KeyringPort},
    security::model::{EncryptionError, Kek, KeyScope, KeySlot, KeySlotFile},
};

use crate::fs::key_slot_store::KeySlotStore;

pub struct DefaultKeyMaterialService<KR, KS> {
    keyring: KR,
    keyslot_store: KS,
}

#[async_trait]
impl<KR, KS> KeyMaterialPort for DefaultKeyMaterialService<KR, KS>
where
    KR: KeyringPort,
    KS: KeySlotStore,
{
    async fn load_kek(&self, scope: &KeyScope) -> Result<Kek, EncryptionError> {
        self.keyring.load_kek(scope)
    }

    async fn store_kek(&self, scope: &KeyScope, kek: &Kek) -> Result<(), EncryptionError> {
        self.keyring.store_kek(scope, kek)
    }

    async fn delete_kek(&self, scope: &KeyScope) -> Result<(), EncryptionError> {
        self.keyring.delete_kek(scope)
    }

    async fn load_keyslot(&self, scope: &KeyScope) -> Result<KeySlot, EncryptionError> {
        let file = self.keyslot_store.load().await?;
        if &file.scope != scope {
            return Err(EncryptionError::KeyMaterialCorrupt);
        }
        Ok(file.into())
    }

    async fn store_keyslot(&self, keyslot: &KeySlot) -> Result<(), EncryptionError> {
        let file = KeySlotFile::try_from(keyslot).map_err(|_| EncryptionError::CorruptedKeySlot)?;
        self.keyslot_store.store(&file).await
    }

    async fn delete_keyslot(&self, scope: &KeyScope) -> Result<(), EncryptionError> {
        let file = self.keyslot_store.load().await?;
        if &file.scope != scope {
            return Err(EncryptionError::KeyMaterialCorrupt);
        }
        self.keyslot_store.delete().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};
    use uc_core::security::model::{
        EncryptionAlgo, EncryptionFormatVersion, KdfParams, KeyScope, KeySlotVersion,
        WrappedMasterKey,
    };

    struct TestKeyringState {
        load_result: Option<Result<Kek, EncryptionError>>,
        store_result: Option<Result<(), EncryptionError>>,
        delete_result: Option<Result<(), EncryptionError>>,
        load_scope: Option<KeyScope>,
        store_scope: Option<KeyScope>,
        store_kek: Option<Kek>,
        delete_scope: Option<KeyScope>,
    }

    #[derive(Clone)]
    struct TestKeyring {
        state: Arc<Mutex<TestKeyringState>>,
    }

    impl TestKeyring {
        fn new() -> (Self, Arc<Mutex<TestKeyringState>>) {
            let state = Arc::new(Mutex::new(TestKeyringState {
                load_result: None,
                store_result: None,
                delete_result: None,
                load_scope: None,
                store_scope: None,
                store_kek: None,
                delete_scope: None,
            }));
            (
                Self {
                    state: state.clone(),
                },
                state,
            )
        }
    }

    impl KeyringPort for TestKeyring {
        fn load_kek(&self, scope: &KeyScope) -> Result<Kek, EncryptionError> {
            let mut state = self.state.lock().expect("lock keyring state");
            state.load_scope = Some(scope.clone());
            state
                .load_result
                .take()
                .unwrap_or(Err(EncryptionError::KeyMaterialCorrupt))
        }

        fn store_kek(&self, scope: &KeyScope, kek: &Kek) -> Result<(), EncryptionError> {
            let mut state = self.state.lock().expect("lock keyring state");
            state.store_scope = Some(scope.clone());
            state.store_kek = Some(kek.clone());
            state.store_result.take().unwrap_or(Ok(()))
        }

        fn delete_kek(&self, scope: &KeyScope) -> Result<(), EncryptionError> {
            let mut state = self.state.lock().expect("lock keyring state");
            state.delete_scope = Some(scope.clone());
            state.delete_result.take().unwrap_or(Ok(()))
        }
    }

    struct TestKeySlotStoreState {
        load_result: Option<Result<KeySlotFile, EncryptionError>>,
        store_result: Option<Result<(), EncryptionError>>,
        delete_result: Option<Result<(), EncryptionError>>,
        stored_slot: Option<KeySlotFile>,
        delete_called: bool,
    }

    #[derive(Clone)]
    struct TestKeySlotStore {
        state: Arc<Mutex<TestKeySlotStoreState>>,
    }

    impl TestKeySlotStore {
        fn new() -> (Self, Arc<Mutex<TestKeySlotStoreState>>) {
            let state = Arc::new(Mutex::new(TestKeySlotStoreState {
                load_result: None,
                store_result: None,
                delete_result: None,
                stored_slot: None,
                delete_called: false,
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
    impl KeySlotStore for TestKeySlotStore {
        async fn load(&self) -> Result<KeySlotFile, EncryptionError> {
            let mut state = self.state.lock().expect("lock keyslot store state");
            state
                .load_result
                .take()
                .unwrap_or(Err(EncryptionError::KeyMaterialCorrupt))
        }

        async fn store(&self, slot: &KeySlotFile) -> Result<(), EncryptionError> {
            let mut state = self.state.lock().expect("lock keyslot store state");
            state.stored_slot = Some(slot.clone());
            state.store_result.take().unwrap_or(Ok(()))
        }

        async fn delete(&self) -> Result<(), EncryptionError> {
            let mut state = self.state.lock().expect("lock keyslot store state");
            state.delete_called = true;
            state.delete_result.take().unwrap_or(Ok(()))
        }
    }

    fn sample_scope(profile_id: &str) -> KeyScope {
        KeyScope {
            profile_id: profile_id.to_string(),
        }
    }

    fn sample_kek() -> Kek {
        Kek([7u8; 32])
    }

    fn sample_keyslot(scope: KeyScope) -> KeySlot {
        KeySlot {
            version: KeySlotVersion::V1,
            scope,
            kdf: KdfParams::for_initialization(),
            salt: vec![1u8; 32],
            wrapped_master_key: Some(WrappedMasterKey {
                blob: uc_core::security::model::EncryptedBlob {
                    version: EncryptionFormatVersion::V1,
                    aead: EncryptionAlgo::XChaCha20Poly1305,
                    nonce: vec![1u8; 24],
                    ciphertext: vec![2u8; 32],
                    aad_fingerprint: None,
                },
            }),
        }
    }

    #[tokio::test]
    async fn load_kek_delegates_to_keyring() {
        let (keyring, state) = TestKeyring::new();
        let (keyslot_store, _) = TestKeySlotStore::new();
        let service = DefaultKeyMaterialService {
            keyring,
            keyslot_store,
        };
        let scope = sample_scope("profile-1");
        let kek = sample_kek();

        state.lock().expect("lock keyring state").load_result = Some(Ok(kek.clone()));

        let loaded = service.load_kek(&scope).await.expect("load kek");

        assert_eq!(loaded, kek);
        let guard = state.lock().expect("lock keyring state");
        assert_eq!(guard.load_scope, Some(scope));
    }

    #[tokio::test]
    async fn store_kek_delegates_to_keyring() {
        let (keyring, state) = TestKeyring::new();
        let (keyslot_store, _) = TestKeySlotStore::new();
        let service = DefaultKeyMaterialService {
            keyring,
            keyslot_store,
        };
        let scope = sample_scope("profile-2");
        let kek = sample_kek();

        state.lock().expect("lock keyring state").store_result = Some(Ok(()));

        service.store_kek(&scope, &kek).await.expect("store kek");

        let guard = state.lock().expect("lock keyring state");
        assert_eq!(guard.store_scope, Some(scope));
        assert_eq!(guard.store_kek, Some(kek));
    }

    #[tokio::test]
    async fn delete_kek_delegates_to_keyring() {
        let (keyring, state) = TestKeyring::new();
        let (keyslot_store, _) = TestKeySlotStore::new();
        let service = DefaultKeyMaterialService {
            keyring,
            keyslot_store,
        };
        let scope = sample_scope("profile-3");

        state.lock().expect("lock keyring state").delete_result = Some(Ok(()));

        service.delete_kek(&scope).await.expect("delete kek");

        let guard = state.lock().expect("lock keyring state");
        assert_eq!(guard.delete_scope, Some(scope));
    }

    #[tokio::test]
    async fn load_keyslot_rejects_scope_mismatch() {
        let (keyring, _) = TestKeyring::new();
        let (keyslot_store, state) = TestKeySlotStore::new();
        let service = DefaultKeyMaterialService {
            keyring,
            keyslot_store,
        };
        let scope = sample_scope("profile-a");
        let file = KeySlotFile::try_from(&sample_keyslot(sample_scope("profile-b"))).unwrap();

        state.lock().expect("lock keyslot state").load_result = Some(Ok(file));

        let err = service
            .load_keyslot(&scope)
            .await
            .expect_err("scope mismatch");

        assert!(matches!(err, EncryptionError::KeyMaterialCorrupt));
    }

    #[tokio::test]
    async fn load_keyslot_returns_keyslot_on_match() {
        let (keyring, _) = TestKeyring::new();
        let (keyslot_store, state) = TestKeySlotStore::new();
        let service = DefaultKeyMaterialService {
            keyring,
            keyslot_store,
        };
        let scope = sample_scope("profile-ok");
        let keyslot = sample_keyslot(scope.clone());
        let file = KeySlotFile::try_from(&keyslot).unwrap();

        state.lock().expect("lock keyslot state").load_result = Some(Ok(file));

        let loaded = service.load_keyslot(&scope).await.expect("load keyslot");

        assert_eq!(loaded, keyslot);
    }

    #[tokio::test]
    async fn store_keyslot_persists_file_representation() {
        let (keyring, _) = TestKeyring::new();
        let (keyslot_store, state) = TestKeySlotStore::new();
        let service = DefaultKeyMaterialService {
            keyring,
            keyslot_store,
        };
        let keyslot = sample_keyslot(sample_scope("profile-store"));

        state.lock().expect("lock keyslot state").store_result = Some(Ok(()));

        service
            .store_keyslot(&keyslot)
            .await
            .expect("store keyslot");

        let guard = state.lock().expect("lock keyslot state");
        assert_eq!(
            guard.stored_slot,
            Some(KeySlotFile::try_from(&keyslot).unwrap())
        );
    }

    #[tokio::test]
    async fn delete_keyslot_rejects_scope_mismatch_without_delete() {
        let (keyring, _) = TestKeyring::new();
        let (keyslot_store, state) = TestKeySlotStore::new();
        let service = DefaultKeyMaterialService {
            keyring,
            keyslot_store,
        };
        let scope = sample_scope("profile-x");
        let file = KeySlotFile::try_from(&sample_keyslot(sample_scope("profile-y"))).unwrap();

        state.lock().expect("lock keyslot state").load_result = Some(Ok(file));

        let err = service
            .delete_keyslot(&scope)
            .await
            .expect_err("scope mismatch");

        assert!(matches!(err, EncryptionError::KeyMaterialCorrupt));
        let guard = state.lock().expect("lock keyslot state");
        assert!(!guard.delete_called);
    }

    #[tokio::test]
    async fn delete_keyslot_deletes_on_match() {
        let (keyring, _) = TestKeyring::new();
        let (keyslot_store, state) = TestKeySlotStore::new();
        let service = DefaultKeyMaterialService {
            keyring,
            keyslot_store,
        };
        let scope = sample_scope("profile-del");
        let file = KeySlotFile::try_from(&sample_keyslot(scope.clone())).unwrap();

        {
            let mut guard = state.lock().expect("lock keyslot state");
            guard.load_result = Some(Ok(file));
            guard.delete_result = Some(Ok(()));
        }

        service
            .delete_keyslot(&scope)
            .await
            .expect("delete keyslot");

        let guard = state.lock().expect("lock keyslot state");
        assert!(guard.delete_called);
    }
}
