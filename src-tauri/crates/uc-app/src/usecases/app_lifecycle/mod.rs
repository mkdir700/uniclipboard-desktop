use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use super::{StartClipboardWatcher, StartNetworkAfterUnlock};

/// Coordinates application lifecycle readiness by orchestrating
/// the clipboard watcher, network runtime, and session ready emission.
pub struct AppLifecycleCoordinator {
    watcher: Arc<StartClipboardWatcher>,
    network: Arc<StartNetworkAfterUnlock>,
    emitter: Arc<dyn SessionReadyEmitter>,
}

/// Helper for constructing the coordinator with explicit dependency fields.
pub struct AppLifecycleCoordinatorDeps {
    pub watcher: Arc<StartClipboardWatcher>,
    pub network: Arc<StartNetworkAfterUnlock>,
    pub emitter: Arc<dyn SessionReadyEmitter>,
}

impl AppLifecycleCoordinator {
    /// Create a new coordinator instance.
    pub fn new(
        watcher: Arc<StartClipboardWatcher>,
        network: Arc<StartNetworkAfterUnlock>,
        emitter: Arc<dyn SessionReadyEmitter>,
    ) -> Self {
        Self {
            watcher,
            network,
            emitter,
        }
    }

    /// Construct a coordinator from dependency bundle.
    pub fn from_deps(deps: AppLifecycleCoordinatorDeps) -> Self {
        let AppLifecycleCoordinatorDeps {
            watcher,
            network,
            emitter,
        } = deps;

        Self::new(watcher, network, emitter)
    }

    /// Ensure the application lifecycle is ready by booting watcher,
    /// network, and emitting the ready event.
    pub async fn ensure_ready(&self) -> Result<()> {
        self.watcher.execute().await?;
        self.network.execute().await?;
        self.emitter.emit_ready().await?;
        Ok(())
    }
}

#[async_trait]
pub trait SessionReadyEmitter: Send + Sync {
    async fn emit_ready(&self) -> Result<()>;
}
