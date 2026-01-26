//! 启动网络的用例

use tracing::{info, info_span, Instrument};
use uc_core::ports::NetworkControlPort;

/// Error type for network startup failures.
#[derive(Debug, thiserror::Error)]
pub enum StartNetworkError {
    #[error("Failed to start network: {0}")]
    StartFailed(String),
}

impl From<anyhow::Error> for StartNetworkError {
    fn from(err: anyhow::Error) -> Self {
        StartNetworkError::StartFailed(err.to_string())
    }
}

/// Use case for starting the network runtime.
pub struct StartNetwork {
    network_control: std::sync::Arc<dyn NetworkControlPort>,
}

impl StartNetwork {
    /// Create a new StartNetwork use case.
    pub fn new(network_control: std::sync::Arc<dyn NetworkControlPort>) -> Self {
        Self { network_control }
    }

    /// Create a new StartNetwork use case from an Arc port.
    pub fn from_port(network_control: std::sync::Arc<dyn NetworkControlPort>) -> Self {
        Self::new(network_control)
    }

    /// Execute the use case.
    pub async fn execute(&self) -> Result<(), StartNetworkError> {
        let span = info_span!("usecase.start_network.execute");

        async {
            info!("Requesting network start");
            self.network_control.start_network().await?;
            info!("Network started successfully");
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
    use std::sync::Arc;

    /// Mock NetworkControlPort
    struct MockNetworkControl {
        started: Arc<std::sync::atomic::AtomicBool>,
        should_fail: bool,
    }

    impl MockNetworkControl {
        fn new() -> Self {
            Self {
                started: Arc::new(std::sync::atomic::AtomicBool::new(false)),
                should_fail: false,
            }
        }

        fn fail_on_start() -> Self {
            Self {
                started: Arc::new(std::sync::atomic::AtomicBool::new(false)),
                should_fail: true,
            }
        }

        fn was_started(&self) -> bool {
            self.started.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl NetworkControlPort for MockNetworkControl {
        async fn start_network(&self) -> anyhow::Result<()> {
            if self.should_fail {
                return Err(anyhow::anyhow!("mock start failure"));
            }
            self.started
                .store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_start_network_succeeds() {
        let control = Arc::new(MockNetworkControl::new());
        let use_case = StartNetwork::new(control.clone());

        let result = use_case.execute().await;

        assert!(result.is_ok(), "start_network should succeed");
        assert!(control.was_started(), "network should be started");
    }

    #[tokio::test]
    async fn test_start_network_propagates_error() {
        let control = Arc::new(MockNetworkControl::fail_on_start());
        let use_case = StartNetwork::new(control);

        let result = use_case.execute().await;

        assert!(result.is_err(), "start_network should fail");
        let err = result.unwrap_err();
        assert!(matches!(err, StartNetworkError::StartFailed(_)));
    }

    #[tokio::test]
    async fn test_from_port_creates_use_case() {
        let control = Arc::new(MockNetworkControl::new());
        let use_case = StartNetwork::from_port(control.clone());

        let result = use_case.execute().await;

        assert!(result.is_ok(), "use_case created from_port should work");
    }
}
