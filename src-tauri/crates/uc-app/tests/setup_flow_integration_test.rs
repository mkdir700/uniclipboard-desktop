use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use tempfile::TempDir;
use uc_app::usecases::{
    AppLifecycleCoordinator, AppLifecycleCoordinatorDeps, InitializeEncryption, LifecycleEvent,
    LifecycleEventEmitter, LifecycleState, LifecycleStatusPort, MarkSetupComplete,
    PairingConfig, PairingOrchestrator, SessionReadyEmitter, SetupOrchestrator,
    StartClipboardWatcher, StartNetworkAfterUnlock,
};
use uc_app::usecases::space_access::SpaceAccessOrchestrator;
use uc_core::network::{DiscoveredPeer, PairedDevice, PairingState};
use uc_core::ports::network_control::NetworkControlPort;
use uc_core::ports::security::key_scope::{KeyScopePort, ScopeError};
use uc_core::ports::security::secure_storage::{SecureStorageError, SecureStoragePort};
use uc_core::ports::watcher_control::{WatcherControlError, WatcherControlPort};
use uc_core::ports::{
    DiscoveryPort, EncryptionSessionPort, PairedDeviceRepositoryError, PairedDeviceRepositoryPort,
    SetupStatusPort,
};
use uc_core::security::model::KeyScope;
use uc_core::setup::SetupState;
use uc_core::PeerId;
use uc_infra::fs::key_slot_store::JsonKeySlotStore;
use uc_infra::security::{
    DefaultKeyMaterialService, EncryptionRepository, FileEncryptionStateRepository,
    InMemoryEncryptionSession,
};
use uc_infra::setup_status::FileSetupStatusRepository;

#[derive(Default)]
struct InMemorySecureStorage {
    data: Mutex<HashMap<String, Vec<u8>>>,
}

impl SecureStoragePort for InMemorySecureStorage {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, SecureStorageError> {
        Ok(self.data.lock().unwrap().get(key).cloned())
    }

    fn set(&self, key: &str, value: &[u8]) -> Result<(), SecureStorageError> {
        self.data
            .lock()
            .unwrap()
            .insert(key.to_string(), value.to_vec());
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<(), SecureStorageError> {
        self.data.lock().unwrap().remove(key);
        Ok(())
    }
}

struct TestKeyScope {
    scope: KeyScope,
}

impl Default for TestKeyScope {
    fn default() -> Self {
        Self {
            scope: KeyScope {
                profile_id: "default".to_string(),
            },
        }
    }
}

#[async_trait::async_trait]
impl KeyScopePort for TestKeyScope {
    async fn current_scope(&self) -> Result<KeyScope, ScopeError> {
        Ok(self.scope.clone())
    }
}

struct MockWatcherControl;

#[async_trait]
impl WatcherControlPort for MockWatcherControl {
    async fn start_watcher(&self) -> Result<(), WatcherControlError> {
        Ok(())
    }

    async fn stop_watcher(&self) -> Result<(), WatcherControlError> {
        Ok(())
    }
}

struct MockNetworkControl;

#[async_trait]
impl NetworkControlPort for MockNetworkControl {
    async fn start_network(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

struct MockSessionReadyEmitter;

#[async_trait]
impl SessionReadyEmitter for MockSessionReadyEmitter {
    async fn emit_ready(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

struct MockLifecycleStatus;

#[async_trait]
impl LifecycleStatusPort for MockLifecycleStatus {
    async fn set_state(&self, _state: LifecycleState) -> anyhow::Result<()> {
        Ok(())
    }

    async fn get_state(&self) -> LifecycleState {
        LifecycleState::Idle
    }
}

struct MockLifecycleEventEmitter;

#[async_trait]
impl LifecycleEventEmitter for MockLifecycleEventEmitter {
    async fn emit_lifecycle_event(&self, _event: LifecycleEvent) -> anyhow::Result<()> {
        Ok(())
    }
}

struct NoopPairedDeviceRepository;

#[async_trait]
impl PairedDeviceRepositoryPort for NoopPairedDeviceRepository {
    async fn get_by_peer_id(
        &self,
        _peer_id: &PeerId,
    ) -> Result<Option<PairedDevice>, PairedDeviceRepositoryError> {
        Ok(None)
    }

    async fn list_all(&self) -> Result<Vec<PairedDevice>, PairedDeviceRepositoryError> {
        Ok(Vec::new())
    }

    async fn upsert(
        &self,
        _device: PairedDevice,
    ) -> Result<(), PairedDeviceRepositoryError> {
        Ok(())
    }

    async fn set_state(
        &self,
        _peer_id: &PeerId,
        _state: PairingState,
    ) -> Result<(), PairedDeviceRepositoryError> {
        Ok(())
    }

    async fn update_last_seen(
        &self,
        _peer_id: &PeerId,
        _last_seen_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), PairedDeviceRepositoryError> {
        Ok(())
    }

    async fn delete(
        &self,
        _peer_id: &PeerId,
    ) -> Result<(), PairedDeviceRepositoryError> {
        Ok(())
    }
}

struct NoopDiscoveryPort;

#[async_trait]
impl DiscoveryPort for NoopDiscoveryPort {
    async fn list_discovered_peers(&self) -> anyhow::Result<Vec<DiscoveredPeer>> {
        Ok(Vec::new())
    }
}

fn build_mock_lifecycle() -> Arc<AppLifecycleCoordinator> {
    Arc::new(AppLifecycleCoordinator::from_deps(
        AppLifecycleCoordinatorDeps {
            watcher: Arc::new(StartClipboardWatcher::new(Arc::new(MockWatcherControl))),
            network: Arc::new(StartNetworkAfterUnlock::new(Arc::new(MockNetworkControl))),
            announcer: None,
            emitter: Arc::new(MockSessionReadyEmitter),
            status: Arc::new(MockLifecycleStatus),
            lifecycle_emitter: Arc::new(MockLifecycleEventEmitter),
        },
    ))
}

fn build_pairing_orchestrator() -> Arc<PairingOrchestrator> {
    let repo = Arc::new(NoopPairedDeviceRepository);
    let (orchestrator, _rx) = PairingOrchestrator::new(
        PairingConfig::default(),
        repo,
        "test-device".to_string(),
        "test-device-id".to_string(),
        "test-peer-id".to_string(),
        vec![],
    );
    Arc::new(orchestrator)
}

fn build_space_access_orchestrator() -> Arc<SpaceAccessOrchestrator> {
    Arc::new(SpaceAccessOrchestrator::new())
}

fn build_discovery_port() -> Arc<dyn DiscoveryPort> {
    Arc::new(NoopDiscoveryPort)
}

#[tokio::test]
async fn create_space_flow_marks_setup_complete_and_persists_state() {
    let temp_dir = TempDir::new().expect("temp dir");
    let vault_dir = temp_dir.path().join("vault");
    std::fs::create_dir_all(&vault_dir).expect("create vault dir");

    let keyslot_store = Arc::new(JsonKeySlotStore::new(vault_dir.clone()));
    let secure_storage = Arc::new(InMemorySecureStorage::default());
    let key_material = Arc::new(DefaultKeyMaterialService::new(
        secure_storage,
        keyslot_store,
    ));

    let encryption = Arc::new(EncryptionRepository);
    let key_scope = Arc::new(TestKeyScope::default());
    let encryption_state = Arc::new(FileEncryptionStateRepository::new(vault_dir.clone()));
    let encryption_session = Arc::new(InMemoryEncryptionSession::new());

    let initialize_encryption = Arc::new(InitializeEncryption::from_ports(
        encryption,
        key_material,
        key_scope,
        encryption_state,
        encryption_session.clone(),
    ));

    let setup_status = Arc::new(FileSetupStatusRepository::with_defaults(vault_dir.clone()));
    let mark_setup_complete = Arc::new(MarkSetupComplete::new(setup_status.clone()));
    let orchestrator = SetupOrchestrator::new(
        initialize_encryption,
        mark_setup_complete,
        setup_status.clone(),
        build_mock_lifecycle(),
        build_pairing_orchestrator(),
        build_space_access_orchestrator(),
        build_discovery_port(),
    );

    orchestrator.new_space().await.expect("new space");
    let state = orchestrator
        .submit_passphrase("secret".to_string(), "secret".to_string())
        .await
        .expect("submit passphrase");

    assert_eq!(state, SetupState::Completed);

    let status = setup_status.get_status().await.expect("get status");
    assert!(status.has_completed);
    assert!(encryption_session.is_ready().await);
    assert!(vault_dir.join(".initialized_encryption").exists());
}
