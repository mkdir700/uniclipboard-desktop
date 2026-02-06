use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use uc_app::usecases::{
    AppLifecycleCoordinator, AppLifecycleCoordinatorDeps, LifecycleEvent, LifecycleEventEmitter,
    LifecycleState, LifecycleStatusPort, SessionReadyEmitter, StartClipboardWatcher,
    StartNetworkAfterUnlock,
};
use uc_core::ports::network_control::NetworkControlPort;
use uc_core::ports::watcher_control::{WatcherControlError, WatcherControlPort};

struct TestMocks {
    watcher_calls: Arc<AtomicUsize>,
    network_calls: Arc<AtomicUsize>,
    emitted_events: Arc<Mutex<Vec<String>>>,
    status_states: Arc<Mutex<Vec<LifecycleState>>>,
    lifecycle_events: Arc<Mutex<Vec<LifecycleEvent>>>,
}

fn test_fixtures() -> (TestMocks, AppLifecycleCoordinator) {
    let watcher_calls = Arc::new(AtomicUsize::new(0));
    let network_calls = Arc::new(AtomicUsize::new(0));
    let emitted_events = Arc::new(Mutex::new(Vec::new()));
    let status_states = Arc::new(Mutex::new(Vec::new()));
    let lifecycle_events = Arc::new(Mutex::new(Vec::new()));

    let watcher_control = Arc::new(MockWatcherControl {
        calls: watcher_calls.clone(),
    });
    let watcher = Arc::new(StartClipboardWatcher::new(watcher_control));

    let network_control = Arc::new(MockNetworkControl {
        calls: network_calls.clone(),
    });
    let network = Arc::new(StartNetworkAfterUnlock::new(network_control));

    let emitter = Arc::new(MockSessionReadyEmitter {
        events: emitted_events.clone(),
    }) as Arc<dyn SessionReadyEmitter>;

    let status = Arc::new(MockLifecycleStatus {
        states: status_states.clone(),
    }) as Arc<dyn LifecycleStatusPort>;

    let lifecycle_emitter = Arc::new(MockLifecycleEventEmitter {
        events: lifecycle_events.clone(),
    }) as Arc<dyn LifecycleEventEmitter>;

    let coordinator = AppLifecycleCoordinator::from_deps(AppLifecycleCoordinatorDeps {
        watcher,
        network,
        emitter,
        status,
        lifecycle_emitter,
    });

    (
        TestMocks {
            watcher_calls,
            network_calls,
            emitted_events,
            status_states,
            lifecycle_events,
        },
        coordinator,
    )
}

struct MockWatcherControl {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl WatcherControlPort for MockWatcherControl {
    async fn start_watcher(&self) -> Result<(), WatcherControlError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    async fn stop_watcher(&self) -> Result<(), WatcherControlError> {
        Ok(())
    }
}

struct MockNetworkControl {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl NetworkControlPort for MockNetworkControl {
    async fn start_network(&self) -> anyhow::Result<()> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

struct MockSessionReadyEmitter {
    events: Arc<Mutex<Vec<String>>>,
}

#[async_trait]
impl SessionReadyEmitter for MockSessionReadyEmitter {
    async fn emit_ready(&self) -> anyhow::Result<()> {
        let mut guard = self.events.lock().await;
        guard.push("ready".to_string());
        Ok(())
    }
}

struct MockLifecycleStatus {
    states: Arc<Mutex<Vec<LifecycleState>>>,
}

#[async_trait]
impl LifecycleStatusPort for MockLifecycleStatus {
    async fn set_state(&self, state: LifecycleState) -> anyhow::Result<()> {
        let mut guard = self.states.lock().await;
        guard.push(state);
        Ok(())
    }

    async fn get_state(&self) -> LifecycleState {
        let guard = self.states.lock().await;
        guard.last().cloned().unwrap_or(LifecycleState::Idle)
    }
}

struct MockLifecycleEventEmitter {
    events: Arc<Mutex<Vec<LifecycleEvent>>>,
}

#[async_trait]
impl LifecycleEventEmitter for MockLifecycleEventEmitter {
    async fn emit_lifecycle_event(&self, event: LifecycleEvent) -> anyhow::Result<()> {
        let mut guard = self.events.lock().await;
        guard.push(event);
        Ok(())
    }
}

#[tokio::test]
async fn coordinator_starts_watcher_network_and_emits_ready() {
    let (mocks, coordinator) = test_fixtures();

    let result = coordinator.ensure_ready().await;

    assert!(result.is_ok(), "coordinator should return Ok");
    assert_eq!(mocks.watcher_calls.load(Ordering::SeqCst), 1);
    assert_eq!(mocks.network_calls.load(Ordering::SeqCst), 1);
    assert_eq!(mocks.emitted_events.lock().await.len(), 1);

    // Verify lifecycle state transitions: Pending → Ready
    let states = mocks.status_states.lock().await;
    assert_eq!(states.len(), 2);
    assert_eq!(states[0], LifecycleState::Pending);
    assert_eq!(states[1], LifecycleState::Ready);

    // Verify lifecycle event emitted
    let events = mocks.lifecycle_events.lock().await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0], LifecycleEvent::Ready);
}
