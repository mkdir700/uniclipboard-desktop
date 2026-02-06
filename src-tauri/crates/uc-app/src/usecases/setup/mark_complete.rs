use std::sync::Arc;

use uc_core::ports::SetupStatusPort;

/// Use case for marking setup as complete.
///
/// This updates the persistent setup completion flag.
pub struct MarkSetupComplete {
    setup_status: Arc<dyn SetupStatusPort>,
}

impl MarkSetupComplete {
    /// Create a new MarkSetupComplete use case from trait objects.
    pub fn new(setup_status: Arc<dyn SetupStatusPort>) -> Self {
        Self { setup_status }
    }

    /// Create a new MarkSetupComplete use case from cloned Arc<dyn Port> references.
    pub fn from_ports(setup_status: Arc<dyn SetupStatusPort>) -> Self {
        Self::new(setup_status)
    }

    pub async fn execute(&self) -> anyhow::Result<()> {
        let mut status = self.setup_status.get_status().await?;
        status.has_completed = true;
        self.setup_status.set_status(&status).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uc_core::setup::SetupStatus;

    struct MockSetupStatusPort {
        status: std::sync::Mutex<SetupStatus>,
    }

    impl MockSetupStatusPort {
        fn new(status: SetupStatus) -> Self {
            Self {
                status: std::sync::Mutex::new(status),
            }
        }
    }

    #[async_trait::async_trait]
    impl SetupStatusPort for MockSetupStatusPort {
        async fn get_status(&self) -> anyhow::Result<SetupStatus> {
            Ok(self.status.lock().unwrap().clone())
        }

        async fn set_status(&self, status: &SetupStatus) -> anyhow::Result<()> {
            *self.status.lock().unwrap() = status.clone();
            Ok(())
        }
    }

    #[tokio::test]
    async fn mark_setup_complete_sets_has_completed() {
        let mock = Arc::new(MockSetupStatusPort::new(SetupStatus::default()));
        let use_case = MarkSetupComplete::new(mock.clone());

        assert!(!mock.get_status().await.unwrap().has_completed);

        use_case.execute().await.unwrap();

        assert!(mock.get_status().await.unwrap().has_completed);
    }
}
