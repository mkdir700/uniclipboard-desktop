use keyring::Entry;
use uc_core::{
    ports::KeyringPort,
    security::model::{EncryptionError, Kek, KeyScope},
};

const SERVICE_NAME: &str = "UniClipboard";
const KEK_PREFIX: &str = "kek:v1:";

fn build_username(scope: &KeyScope) -> String {
    format!("{}{}", KEK_PREFIX, scope.to_identifier())
}

trait KeyringEntryOps {
    fn get_secret(&self) -> Result<Vec<u8>, keyring::Error>;
    fn set_secret(&self, secret: &[u8]) -> Result<(), keyring::Error>;
    fn delete_credential(&self) -> Result<(), keyring::Error>;
}

trait KeyringBackend {
    type Entry: KeyringEntryOps;
    fn new_entry(&self, service: &str, username: &str) -> Result<Self::Entry, keyring::Error>;
}

struct RealBackend;

struct RealEntry {
    inner: Entry,
}

impl KeyringEntryOps for RealEntry {
    fn get_secret(&self) -> Result<Vec<u8>, keyring::Error> {
        self.inner.get_secret()
    }

    fn set_secret(&self, secret: &[u8]) -> Result<(), keyring::Error> {
        self.inner.set_secret(secret)
    }

    fn delete_credential(&self) -> Result<(), keyring::Error> {
        self.inner.delete_credential()
    }
}

impl KeyringBackend for RealBackend {
    type Entry = RealEntry;

    fn new_entry(&self, service: &str, username: &str) -> Result<Self::Entry, keyring::Error> {
        Entry::new(service, username).map(|inner| RealEntry { inner })
    }
}

fn load_kek_with_backend<B: KeyringBackend>(
    backend: &B,
    scope: &KeyScope,
) -> Result<Kek, EncryptionError> {
    let entry = backend
        .new_entry(SERVICE_NAME, &build_username(scope))
        .map_err(|e| {
            EncryptionError::KeyringError(format!(
                "failed to access keyring entry: {}, scope may be invalid.",
                e
            ))
        })?;
    let secret = entry.get_secret().map_err(|e| match e {
        keyring::Error::NoEntry => EncryptionError::KeyNotFound,
        keyring::Error::PlatformFailure(msg) => EncryptionError::KeyringError(msg.to_string()),
        _ => EncryptionError::KeyringError("unknown error".into()),
    })?;
    let kek = Kek::from_bytes(&secret).map_err(|e| {
        EncryptionError::KeyringError(format!("invalid KEK material in keyring: {e}"))
    })?;
    Ok(kek)
}

fn store_kek_with_backend<B: KeyringBackend>(
    backend: &B,
    scope: &KeyScope,
    kek: &Kek,
) -> Result<(), EncryptionError> {
    let entry = backend
        .new_entry(SERVICE_NAME, &build_username(scope))
        .map_err(|e| {
            EncryptionError::KeyringError(format!(
                "failed to access keyring entry: {}, scope may be invalid.",
                e
            ))
        })?;
    entry
        .set_secret(&kek.0)
        .map_err(|e| EncryptionError::KeyringError(format!("failed to store KEK: {}", e)))?;
    Ok(())
}

fn delete_kek_with_backend<B: KeyringBackend>(
    backend: &B,
    scope: &KeyScope,
) -> Result<(), EncryptionError> {
    let entry = backend
        .new_entry(SERVICE_NAME, &build_username(scope))
        .map_err(|e| {
            EncryptionError::KeyringError(format!(
                "failed to access keyring entry: {}, scope may be invalid.",
                e
            ))
        })?;
    match entry.delete_credential() {
        Ok(_) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(EncryptionError::KeyringError(format!(
            "failed to delete KEK: {e}"
        ))),
    }
}

pub struct SystemKeyring {}

impl KeyringPort for SystemKeyring {
    fn load_kek(&self, scope: &KeyScope) -> Result<Kek, EncryptionError> {
        load_kek_with_backend(&RealBackend, scope)
    }
    /// Store KEK into OS keyring.
    ///
    /// Requirements:
    /// - Idempotent (overwrite if exists)
    /// - Atomic at keyring level
    fn store_kek(&self, scope: &KeyScope, kek: &Kek) -> Result<(), EncryptionError> {
        store_kek_with_backend(&RealBackend, scope, kek)
    }

    /// Delete KEK from OS keyring.
    ///
    /// Used in reset / unrecoverable error flows.
    fn delete_kek(&self, scope: &KeyScope) -> Result<(), EncryptionError> {
        delete_kek_with_backend(&RealBackend, scope)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, collections::HashMap, rc::Rc};

    #[derive(Default)]
    struct MockState {
        entries: HashMap<String, Vec<u8>>,
        new_error: Option<keyring::Error>,
        get_error: Option<keyring::Error>,
        set_error: Option<keyring::Error>,
        delete_error: Option<keyring::Error>,
    }

    #[derive(Clone, Default)]
    struct MockBackend {
        state: Rc<RefCell<MockState>>,
    }

    struct MockEntry {
        username: String,
        state: Rc<RefCell<MockState>>,
    }

    impl MockBackend {
        fn insert_secret(&self, username: &str, secret: Vec<u8>) {
            self.state
                .borrow_mut()
                .entries
                .insert(username.to_string(), secret);
        }

        fn set_new_error(&self, err: keyring::Error) {
            self.state.borrow_mut().new_error = Some(err);
        }

        fn set_get_error(&self, err: keyring::Error) {
            self.state.borrow_mut().get_error = Some(err);
        }

        fn set_set_error(&self, err: keyring::Error) {
            self.state.borrow_mut().set_error = Some(err);
        }

        fn set_delete_error(&self, err: keyring::Error) {
            self.state.borrow_mut().delete_error = Some(err);
        }
    }

    impl KeyringBackend for MockBackend {
        type Entry = MockEntry;

        fn new_entry(&self, _service: &str, username: &str) -> Result<Self::Entry, keyring::Error> {
            let err = self.state.borrow_mut().new_error.take();
            if let Some(err) = err {
                return Err(err);
            }
            Ok(MockEntry {
                username: username.to_string(),
                state: Rc::clone(&self.state),
            })
        }
    }

    impl KeyringEntryOps for MockEntry {
        fn get_secret(&self) -> Result<Vec<u8>, keyring::Error> {
            let err = self.state.borrow_mut().get_error.take();
            if let Some(err) = err {
                return Err(err);
            }
            let state = self.state.borrow();
            match state.entries.get(&self.username) {
                Some(value) => Ok(value.clone()),
                None => Err(keyring::Error::NoEntry),
            }
        }

        fn set_secret(&self, secret: &[u8]) -> Result<(), keyring::Error> {
            let err = self.state.borrow_mut().set_error.take();
            if let Some(err) = err {
                return Err(err);
            }
            self.state
                .borrow_mut()
                .entries
                .insert(self.username.clone(), secret.to_vec());
            Ok(())
        }

        fn delete_credential(&self) -> Result<(), keyring::Error> {
            let err = self.state.borrow_mut().delete_error.take();
            if let Some(err) = err {
                return Err(err);
            }
            let mut state = self.state.borrow_mut();
            if state.entries.remove(&self.username).is_some() {
                Ok(())
            } else {
                Err(keyring::Error::NoEntry)
            }
        }
    }

    #[test]
    fn build_username_includes_prefix_and_scope() {
        let scope = KeyScope {
            profile_id: "user123".to_string(),
        };
        let username = build_username(&scope);
        assert_eq!(username, "kek:v1:profile:user123");
    }

    #[test]
    fn load_missing_key_returns_not_found() {
        let scope = KeyScope {
            profile_id: "missing".to_string(),
        };
        let backend = MockBackend::default();
        match load_kek_with_backend(&backend, &scope) {
            Err(EncryptionError::KeyNotFound) => {}
            other => panic!("expected KeyNotFound, got: {:?}", other),
        }
    }

    #[test]
    fn store_and_load_roundtrip() {
        let scope = KeyScope {
            profile_id: "roundtrip".to_string(),
        };
        let backend = MockBackend::default();
        let kek = Kek([42u8; 32]);
        let result = store_kek_with_backend(&backend, &scope, &kek);
        if let Err(err) = result {
            panic!("store_kek failed: {err:?}");
        }
        let loaded = load_kek_with_backend(&backend, &scope).expect("load_kek failed");
        assert_eq!(loaded, kek);
    }

    #[test]
    fn delete_is_idempotent() {
        let scope = KeyScope {
            profile_id: "delete".to_string(),
        };
        let backend = MockBackend::default();
        let _ = delete_kek_with_backend(&backend, &scope);
        delete_kek_with_backend(&backend, &scope).expect("delete should be idempotent");
    }

    #[test]
    fn load_invalid_kek_material_returns_error() {
        let scope = KeyScope {
            profile_id: "invalid".to_string(),
        };
        let backend = MockBackend::default();
        let username = build_username(&scope);
        backend.insert_secret(&username, vec![1u8, 2u8, 3u8]);
        match load_kek_with_backend(&backend, &scope) {
            Err(EncryptionError::KeyringError(msg)) => {
                assert!(msg.contains("invalid KEK material"));
            }
            other => panic!("expected KeyringError, got: {:?}", other),
        }
    }

    #[test]
    fn load_maps_platform_failure() {
        let scope = KeyScope {
            profile_id: "platform-failure".to_string(),
        };
        let backend = MockBackend::default();
        backend.set_get_error(keyring::Error::PlatformFailure("boom".into()));
        match load_kek_with_backend(&backend, &scope) {
            Err(EncryptionError::KeyringError(msg)) => {
                assert!(msg.contains("boom"));
            }
            other => panic!("expected KeyringError, got: {:?}", other),
        }
    }

    #[test]
    fn store_maps_error() {
        let scope = KeyScope {
            profile_id: "store-failure".to_string(),
        };
        let backend = MockBackend::default();
        backend.set_set_error(keyring::Error::PlatformFailure("store-bad".into()));
        let kek = Kek([7u8; 32]);
        match store_kek_with_backend(&backend, &scope, &kek) {
            Err(EncryptionError::KeyringError(msg)) => {
                assert!(msg.contains("failed to store KEK"));
            }
            other => panic!("expected KeyringError, got: {:?}", other),
        }
    }

    #[test]
    fn delete_maps_error() {
        let scope = KeyScope {
            profile_id: "delete-failure".to_string(),
        };
        let backend = MockBackend::default();
        backend.set_delete_error(keyring::Error::PlatformFailure("delete-bad".into()));
        match delete_kek_with_backend(&backend, &scope) {
            Err(EncryptionError::KeyringError(msg)) => {
                assert!(msg.contains("failed to delete KEK"));
            }
            other => panic!("expected KeyringError, got: {:?}", other),
        }
    }

    #[test]
    fn entry_creation_error_maps_to_keyring_error() {
        let scope = KeyScope {
            profile_id: "new-failure".to_string(),
        };
        let backend = MockBackend::default();
        backend.set_new_error(keyring::Error::PlatformFailure("new-bad".into()));
        match load_kek_with_backend(&backend, &scope) {
            Err(EncryptionError::KeyringError(msg)) => {
                assert!(msg.contains("failed to access keyring entry"));
            }
            other => panic!("expected KeyringError, got: {:?}", other),
        }
    }
}
