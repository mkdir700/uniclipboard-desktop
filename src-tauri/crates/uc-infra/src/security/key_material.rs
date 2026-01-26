use async_trait::async_trait;
use std::sync::Arc;
use uc_core::{
    ports::{KeyMaterialPort, SecureStoragePort},
    security::model::{EncryptionError, Kek, KeyScope, KeySlot, KeySlotFile},
};

use crate::fs::key_slot_store::KeySlotStore;

pub struct DefaultKeyMaterialService {
    secure_storage: Arc<dyn SecureStoragePort>,
    keyslot_store: Arc<dyn KeySlotStore>,
}

impl DefaultKeyMaterialService {
    /// Create a new key material service
    /// 创建新的密钥材料服务
    pub fn new(
        secure_storage: Arc<dyn SecureStoragePort>,
        keyslot_store: Arc<dyn KeySlotStore>,
    ) -> Self {
        Self {
            secure_storage,
            keyslot_store,
        }
    }
}

fn kek_key(scope: &KeyScope) -> String {
    format!("kek:v1:{}", scope.to_identifier())
}

fn map_storage_error(err: uc_core::ports::SecureStorageError) -> EncryptionError {
    use uc_core::ports::SecureStorageError as StorageError;
    match err {
        StorageError::PermissionDenied(_) => EncryptionError::PermissionDenied,
        StorageError::Corrupt(_) => EncryptionError::KeyMaterialCorrupt,
        StorageError::Unavailable(msg) | StorageError::Other(msg) => {
            EncryptionError::KeyringError(msg)
        }
    }
}

#[async_trait]
impl KeyMaterialPort for DefaultKeyMaterialService {
    async fn load_kek(&self, scope: &KeyScope) -> Result<Kek, EncryptionError> {
        let key = kek_key(scope);
        let secret = self
            .secure_storage
            .get(&key)
            .map_err(map_storage_error)?
            .ok_or(EncryptionError::KeyNotFound)?;
        Kek::from_bytes(&secret)
            .map_err(|e| EncryptionError::KeyringError(format!("invalid KEK material: {e}")))
    }

    async fn store_kek(&self, scope: &KeyScope, kek: &Kek) -> Result<(), EncryptionError> {
        let key = kek_key(scope);
        self.secure_storage
            .set(&key, &kek.0)
            .map_err(map_storage_error)
    }

    async fn delete_kek(&self, scope: &KeyScope) -> Result<(), EncryptionError> {
        let key = kek_key(scope);
        self.secure_storage.delete(&key).map_err(map_storage_error)
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
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use uc_core::ports::SecureStorageError;
    use uc_core::security::model::{
        EncryptionAlgo, EncryptionFormatVersion, KdfParams, KeyScope, KeySlotVersion,
        WrappedMasterKey,
    };

    struct TestSecureStorageState {
        data: HashMap<String, Vec<u8>>,
        get_result: Option<Result<Option<Vec<u8>>, SecureStorageError>>,
        set_result: Option<Result<(), SecureStorageError>>,
        delete_result: Option<Result<(), SecureStorageError>>,
        get_key: Option<String>,
        set_key: Option<String>,
        set_value: Option<Vec<u8>>,
        delete_key: Option<String>,
    }

    #[derive(Clone)]
    struct TestSecureStorage {
        state: Arc<Mutex<TestSecureStorageState>>,
    }

    impl TestSecureStorage {
        fn new() -> (Self, Arc<Mutex<TestSecureStorageState>>) {
            let state = Arc::new(Mutex::new(TestSecureStorageState {
                data: HashMap::new(),
                get_result: None,
                set_result: None,
                delete_result: None,
                get_key: None,
                set_key: None,
                set_value: None,
                delete_key: None,
            }));
            (
                Self {
                    state: state.clone(),
                },
                state,
            )
        }
    }

    impl SecureStoragePort for TestSecureStorage {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, SecureStorageError> {
            let mut state = self.state.lock().expect("lock secure storage state");
            state.get_key = Some(key.to_string());
            if let Some(result) = state.get_result.take() {
                return result;
            }
            Ok(state.data.get(key).cloned())
        }

        fn set(&self, key: &str, value: &[u8]) -> Result<(), SecureStorageError> {
            let mut state = self.state.lock().expect("lock secure storage state");
            state.set_key = Some(key.to_string());
            state.set_value = Some(value.to_vec());
            if let Some(result) = state.set_result.take() {
                return result;
            }
            state.data.insert(key.to_string(), value.to_vec());
            Ok(())
        }

        fn delete(&self, key: &str) -> Result<(), SecureStorageError> {
            let mut state = self.state.lock().expect("lock secure storage state");
            state.delete_key = Some(key.to_string());
            if let Some(result) = state.delete_result.take() {
                return result;
            }
            state.data.remove(key);
            Ok(())
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
    async fn load_kek_reads_from_secure_storage() {
        let (storage, state) = TestSecureStorage::new();
        let (keyslot_store, _) = TestKeySlotStore::new();
        let service = DefaultKeyMaterialService::new(
            Arc::new(storage) as Arc<dyn SecureStoragePort>,
            Arc::new(keyslot_store) as Arc<dyn KeySlotStore>,
        );
        let scope = sample_scope("profile-1");
        let kek = sample_kek();
        let key = kek_key(&scope);

        {
            let mut guard = state.lock().expect("lock secure storage state");
            guard.data.insert(key.clone(), kek.0.to_vec());
        }

        let loaded = service.load_kek(&scope).await.expect("load kek");
        assert_eq!(loaded, kek);

        let guard = state.lock().expect("lock secure storage state");
        assert_eq!(guard.get_key, Some(key));
    }

    #[tokio::test]
    async fn store_kek_writes_to_secure_storage() {
        let (storage, state) = TestSecureStorage::new();
        let (keyslot_store, _) = TestKeySlotStore::new();
        let service = DefaultKeyMaterialService::new(
            Arc::new(storage) as Arc<dyn SecureStoragePort>,
            Arc::new(keyslot_store) as Arc<dyn KeySlotStore>,
        );
        let scope = sample_scope("profile-2");
        let kek = sample_kek();
        let key = kek_key(&scope);

        service.store_kek(&scope, &kek).await.expect("store kek");

        let guard = state.lock().expect("lock secure storage state");
        assert_eq!(guard.set_key, Some(key));
        assert_eq!(guard.set_value, Some(kek.0.to_vec()));
    }

    #[tokio::test]
    async fn delete_kek_writes_to_secure_storage() {
        let (storage, state) = TestSecureStorage::new();
        let (keyslot_store, _) = TestKeySlotStore::new();
        let service = DefaultKeyMaterialService::new(
            Arc::new(storage) as Arc<dyn SecureStoragePort>,
            Arc::new(keyslot_store) as Arc<dyn KeySlotStore>,
        );
        let scope = sample_scope("profile-3");
        let key = kek_key(&scope);

        service.delete_kek(&scope).await.expect("delete kek");

        let guard = state.lock().expect("lock secure storage state");
        assert_eq!(guard.delete_key, Some(key));
    }

    #[tokio::test]
    async fn load_keyslot_rejects_scope_mismatch() {
        let (storage, _) = TestSecureStorage::new();
        let (keyslot_store, state) = TestKeySlotStore::new();
        let service = DefaultKeyMaterialService::new(
            Arc::new(storage) as Arc<dyn SecureStoragePort>,
            Arc::new(keyslot_store) as Arc<dyn KeySlotStore>,
        );
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
        let (storage, _) = TestSecureStorage::new();
        let (keyslot_store, state) = TestKeySlotStore::new();
        let service = DefaultKeyMaterialService::new(
            Arc::new(storage) as Arc<dyn SecureStoragePort>,
            Arc::new(keyslot_store) as Arc<dyn KeySlotStore>,
        );
        let scope = sample_scope("profile-ok");
        let keyslot = sample_keyslot(scope.clone());
        let file = KeySlotFile::try_from(&keyslot).unwrap();

        state.lock().expect("lock keyslot state").load_result = Some(Ok(file));

        let loaded = service.load_keyslot(&scope).await.expect("load keyslot");

        assert_eq!(loaded, keyslot);
    }

    #[tokio::test]
    async fn store_keyslot_persists_file_representation() {
        let (storage, _) = TestSecureStorage::new();
        let (keyslot_store, state) = TestKeySlotStore::new();
        let service = DefaultKeyMaterialService::new(
            Arc::new(storage) as Arc<dyn SecureStoragePort>,
            Arc::new(keyslot_store) as Arc<dyn KeySlotStore>,
        );
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
        let (storage, _) = TestSecureStorage::new();
        let (keyslot_store, state) = TestKeySlotStore::new();
        let service = DefaultKeyMaterialService::new(
            Arc::new(storage) as Arc<dyn SecureStoragePort>,
            Arc::new(keyslot_store) as Arc<dyn KeySlotStore>,
        );
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
        let (storage, _) = TestSecureStorage::new();
        let (keyslot_store, state) = TestKeySlotStore::new();
        let service = DefaultKeyMaterialService::new(
            Arc::new(storage) as Arc<dyn SecureStoragePort>,
            Arc::new(keyslot_store) as Arc<dyn KeySlotStore>,
        );
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
