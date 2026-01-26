use std::sync::Arc;
use tracing::{info, info_span, Instrument};
use uc_core::ports::NetworkControlPort;

use super::start_network::StartNetworkError;

/// Use case for starting the network runtime after unlock.
pub struct StartNetworkAfterUnlock {
    network_control: Arc<dyn NetworkControlPort>,
}

impl StartNetworkAfterUnlock {
    /// Create a new StartNetworkAfterUnlock use case.
    pub fn new(network_control: Arc<dyn NetworkControlPort>) -> Self {
        Self { network_control }
    }

    /// Create a new StartNetworkAfterUnlock use case from an Arc port.
    pub fn from_port(network_control: Arc<dyn NetworkControlPort>) -> Self {
        Self::new(network_control)
    }

    /// Execute the use case.
    pub async fn execute(&self) -> Result<(), StartNetworkError> {
        let span = info_span!("usecase.start_network_after_unlock.execute");

        async {
            info!("Requesting network start after unlock");
            self.network_control.start_network().await?;
            info!("Network started successfully after unlock");
            Ok(())
        }
        .instrument(span)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct MockNetworkControl {
        started: Arc<std::sync::atomic::AtomicBool>,
    }

    impl MockNetworkControl {
        fn new() -> Self {
            Self {
                started: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            }
        }

        fn was_started(&self) -> bool {
            self.started.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl NetworkControlPort for MockNetworkControl {
        async fn start_network(&self) -> anyhow::Result<()> {
            self.started
                .store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn start_network_after_unlock_invokes_network_control() {
        let control = Arc::new(MockNetworkControl::new());
        let use_case = StartNetworkAfterUnlock::new(control.clone());

        let result = use_case.execute().await;

        assert!(result.is_ok(), "start_network should succeed");
        assert!(control.was_started(), "network should be started");
    }
}
