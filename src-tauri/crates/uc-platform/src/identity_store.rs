use libp2p::identity::Keypair;
use uc_core::ports::{IdentityStoreError, IdentityStorePort};

const SERVICE_NAME: &str = "UniClipboard";
const IDENTITY_USERNAME: &str = "libp2p-identity:v1";

trait KeyringEntryOps {
    fn get_secret(&self) -> Result<Vec<u8>, keyring::Error>;
    fn set_secret(&self, secret: &[u8]) -> Result<(), keyring::Error>;
}

trait KeyringBackend {
    type Entry: KeyringEntryOps;
    fn new_entry(&self, service: &str, username: &str) -> Result<Self::Entry, keyring::Error>;
}

struct RealBackend;

struct RealEntry {
    inner: keyring::Entry,
}

impl KeyringEntryOps for RealEntry {
    fn get_secret(&self) -> Result<Vec<u8>, keyring::Error> {
        self.inner.get_secret()
    }

    fn set_secret(&self, secret: &[u8]) -> Result<(), keyring::Error> {
        self.inner.set_secret(secret)
    }
}

impl KeyringBackend for RealBackend {
    type Entry = RealEntry;

    fn new_entry(&self, service: &str, username: &str) -> Result<Self::Entry, keyring::Error> {
        keyring::Entry::new(service, username).map(|inner| RealEntry { inner })
    }
}

fn load_identity_with_backend<B: KeyringBackend>(
    backend: &B,
) -> Result<Option<Vec<u8>>, IdentityStoreError> {
    let entry = backend
        .new_entry(SERVICE_NAME, IDENTITY_USERNAME)
        .map_err(|e| IdentityStoreError::Store(format!("failed to access keyring entry: {e}")))?;

    match entry.get_secret() {
        Ok(secret) => Ok(Some(secret)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(keyring::Error::PlatformFailure(msg)) => {
            Err(IdentityStoreError::Store(msg.to_string()))
        }
        Err(e) => Err(IdentityStoreError::Store(format!(
            "failed to load identity from keyring: {e}"
        ))),
    }
}

fn store_identity_with_backend<B: KeyringBackend>(
    backend: &B,
    identity: &[u8],
) -> Result<(), IdentityStoreError> {
    let entry = backend
        .new_entry(SERVICE_NAME, IDENTITY_USERNAME)
        .map_err(|e| IdentityStoreError::Store(format!("failed to access keyring entry: {e}")))?;
    entry
        .set_secret(identity)
        .map_err(|e| IdentityStoreError::Store(format!("failed to store identity: {e}")))?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct SystemIdentityStore;

impl SystemIdentityStore {
    pub fn new() -> Self {
        Self
    }
}

impl IdentityStorePort for SystemIdentityStore {
    fn load_identity(&self) -> Result<Option<Vec<u8>>, IdentityStoreError> {
        load_identity_with_backend(&RealBackend)
    }

    fn store_identity(&self, identity: &[u8]) -> Result<(), IdentityStoreError> {
        store_identity_with_backend(&RealBackend, identity)
    }
}

pub fn load_or_create_identity(
    store: &dyn IdentityStorePort,
) -> Result<Keypair, IdentityStoreError> {
    if let Some(bytes) = store.load_identity()? {
        let keypair = Keypair::from_protobuf_encoding(&bytes).map_err(|e| {
            IdentityStoreError::Corrupt(format!("failed to decode identity keypair: {e}"))
        })?;
        Ok(keypair)
    } else {
        let keypair = Keypair::generate_ed25519();
        let bytes = keypair.to_protobuf_encoding().map_err(|e| {
            IdentityStoreError::Store(format!("failed to encode identity keypair: {e}"))
        })?;
        store.store_identity(&bytes)?;
        Ok(keypair)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MockState {
        entries: HashMap<String, Vec<u8>>,
        new_error: Option<keyring::Error>,
        get_error: Option<keyring::Error>,
        set_error: Option<keyring::Error>,
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
            if let Some(err) = self.state.borrow_mut().get_error.take() {
                return Err(err);
            }
            self.state
                .borrow()
                .entries
                .get(&self.username)
                .cloned()
                .ok_or(keyring::Error::NoEntry)
        }

        fn set_secret(&self, secret: &[u8]) -> Result<(), keyring::Error> {
            if let Some(err) = self.state.borrow_mut().set_error.take() {
                return Err(err);
            }
            self.state
                .borrow_mut()
                .entries
                .insert(self.username.clone(), secret.to_vec());
            Ok(())
        }
    }

    #[derive(Default)]
    struct MemoryIdentityStore {
        data: Mutex<Option<Vec<u8>>>,
    }

    impl IdentityStorePort for MemoryIdentityStore {
        fn load_identity(&self) -> Result<Option<Vec<u8>>, IdentityStoreError> {
            let guard = self.data.lock().expect("lock memory store");
            Ok(guard.clone())
        }

        fn store_identity(&self, identity: &[u8]) -> Result<(), IdentityStoreError> {
            let mut guard = self.data.lock().expect("lock memory store");
            *guard = Some(identity.to_vec());
            Ok(())
        }
    }

    #[test]
    fn load_identity_returns_none_when_missing() {
        let backend = MockBackend::default();
        let result = load_identity_with_backend(&backend).expect("load should succeed");
        assert!(result.is_none());
    }

    #[test]
    fn load_identity_returns_bytes_when_present() {
        let backend = MockBackend::default();
        backend.insert_secret(IDENTITY_USERNAME, vec![1u8, 2u8]);
        let result = load_identity_with_backend(&backend).expect("load should succeed");
        assert_eq!(result, Some(vec![1u8, 2u8]));
    }

    #[test]
    fn store_identity_persists_secret() {
        let backend = MockBackend::default();
        store_identity_with_backend(&backend, &[9u8, 8u8]).expect("store should succeed");
        let result = load_identity_with_backend(&backend).expect("load should succeed");
        assert_eq!(result, Some(vec![9u8, 8u8]));
    }

    #[test]
    fn load_or_create_identity_is_stable() {
        let store = MemoryIdentityStore::default();
        let first = load_or_create_identity(&store).expect("first load should succeed");
        let second = load_or_create_identity(&store).expect("second load should succeed");

        let first_id = libp2p::PeerId::from(first.public()).to_string();
        let second_id = libp2p::PeerId::from(second.public()).to_string();

        assert_eq!(first_id, second_id, "peer id should remain stable");
    }

    #[test]
    fn load_or_create_identity_rejects_corrupt_bytes() {
        let store = MemoryIdentityStore::default();
        store
            .store_identity(&[1u8, 2u8, 3u8])
            .expect("store should succeed");

        let result = load_or_create_identity(&store);
        assert!(matches!(result, Err(IdentityStoreError::Corrupt(_))));
    }

    #[test]
    fn load_identity_maps_backend_errors() {
        let backend = MockBackend::default();
        backend.set_get_error(keyring::Error::PlatformFailure("no access".into()));

        let result = load_identity_with_backend(&backend);
        assert!(matches!(result, Err(IdentityStoreError::Store(_))));
    }

    #[test]
    fn store_identity_maps_backend_errors() {
        let backend = MockBackend::default();
        backend.set_set_error(keyring::Error::PlatformFailure("no access".into()));

        let result = store_identity_with_backend(&backend, &[7u8, 7u8]);
        assert!(matches!(result, Err(IdentityStoreError::Store(_))));
    }

    #[test]
    fn load_identity_maps_entry_creation_error() {
        let backend = MockBackend::default();
        backend.set_new_error(keyring::Error::PlatformFailure("no entry".into()));

        let result = load_identity_with_backend(&backend);
        assert!(matches!(result, Err(IdentityStoreError::Store(_))));
    }
}
