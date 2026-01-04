//! StartApp use case - initializes and starts all application subsystems

use anyhow::Result;
use log::info;
use uc_core::clipboard::Payload;
use uc_core::device::Device;
use uc_core::network::{ClipboardMessage, NetworkEvent};
use uc_core::ports::{ClipboardPort, NetworkPort, StoragePort};
use std::sync::Arc;

/// StartApp use case - initializes and starts all application subsystems
pub struct StartApp<N, C, S>
where
    N: NetworkPort,
    C: ClipboardPort,
    S: StoragePort,
{
    network: Arc<N>,
    clipboard: Arc<C>,
    storage: Arc<S>,
}

impl<N, C, S> StartApp<N, C, S>
where
    N: NetworkPort,
    C: ClipboardPort,
    S: StoragePort,
{
    pub fn new(network: Arc<N>, clipboard: Arc<C>, storage: Arc<S>) -> Self {
        Self {
            network,
            clipboard,
            storage,
        }
    }

    /// Execute the startup sequence
    pub async fn execute(&self) -> Result<AppContext> {
        info!("Starting application...");

        // 1. Load config and ensure current device
        let current_device = self.ensure_current_device().await?;

        // 2. Start clipboard monitoring
        let clipboard_rx = self.clipboard.start_monitoring().await?;
        info!("Clipboard monitoring started");

        // 3. Subscribe to network events
        let network_event_rx = self.network.subscribe_events().await?;
        info!("Network event subscription started");

        // 4. Subscribe to remote clipboard messages
        let remote_clipboard_rx = self.network.subscribe_clipboard().await?;
        info!("Remote clipboard subscription started");

        Ok(AppContext {
            current_device,
            clipboard_rx,
            network_event_rx,
            remote_clipboard_rx,
        })
    }

    async fn ensure_current_device(&self) -> Result<Device> {
        match self.storage.get_current_device().await? {
            Some(device) => Ok(device),
            None => {
                info!("No current device found, registering...");
                // TODO: Implement device registration logic
                // This will be done when we integrate with the existing codebase
                Err(anyhow::anyhow!("Device registration not yet implemented"))
            }
        }
    }
}

/// Context returned after successful application startup
pub struct AppContext {
    pub current_device: Device,
    pub clipboard_rx: tokio::sync::mpsc::Receiver<Payload>,
    pub network_event_rx: tokio::sync::mpsc::Receiver<NetworkEvent>,
    pub remote_clipboard_rx: tokio::sync::mpsc::Receiver<ClipboardMessage>,
}
