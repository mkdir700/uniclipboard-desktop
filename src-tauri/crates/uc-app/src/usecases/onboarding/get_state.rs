use std::sync::Arc;

use uc_core::ports::security::encryption_state::EncryptionStatePort;
use uc_core::ports::OnboardingStatePort;
use uc_core::security::state::EncryptionState;

use super::OnboardingStateDto;

/// Use case for getting current onboarding state.
///
/// Returns the current state of onboarding including:
/// - Whether onboarding is completed
/// - Whether encryption password is initialized
/// - Whether device is registered
pub struct GetOnboardingState {
    onboarding_state: Arc<dyn OnboardingStatePort>,
    encryption_state: Arc<dyn EncryptionStatePort>,
}

impl GetOnboardingState {
    /// Create a new GetOnboardingState use case from trait objects.
    pub fn new(
        onboarding_state: Arc<dyn OnboardingStatePort>,
        encryption_state: Arc<dyn EncryptionStatePort>,
    ) -> Self {
        Self {
            onboarding_state,
            encryption_state,
        }
    }

    /// Create a new GetOnboardingState use case from cloned Arc<dyn Port> references.
    ///
    /// This is a convenience method for the UseCases accessor pattern.
    pub fn from_ports(
        onboarding_state: Arc<dyn OnboardingStatePort>,
        encryption_state: Arc<dyn EncryptionStatePort>,
    ) -> Self {
        Self::new(onboarding_state, encryption_state)
    }

    /// Get current onboarding state.
    pub async fn execute(&self) -> anyhow::Result<OnboardingStateDto> {
        let onboarding_state = self.onboarding_state.get_state().await?;

        // Check if encryption is initialized
        let encryption_state = self
            .encryption_state
            .load_state()
            .await
            .unwrap_or(EncryptionState::Uninitialized);
        let encryption_initialized = encryption_state == EncryptionState::Initialized;

        Ok(OnboardingStateDto {
            has_completed: onboarding_state.has_completed,
            encryption_password_set: encryption_initialized,
            device_registered: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use uc_core::onboarding::OnboardingState;

    struct MockOnboardingStatePort {
        state: std::sync::Mutex<OnboardingState>,
    }

    impl MockOnboardingStatePort {
        fn new(state: OnboardingState) -> Self {
            Self {
                state: std::sync::Mutex::new(state),
            }
        }
    }

    #[async_trait::async_trait]
    impl OnboardingStatePort for MockOnboardingStatePort {
        async fn get_state(&self) -> anyhow::Result<OnboardingState> {
            Ok(self.state.lock().unwrap().clone())
        }

        async fn set_state(&self, state: &OnboardingState) -> anyhow::Result<()> {
            *self.state.lock().unwrap() = state.clone();
            Ok(())
        }

        async fn reset(&self) -> anyhow::Result<()> {
            *self.state.lock().unwrap() = OnboardingState::default();
            Ok(())
        }
    }

    struct MockEncryptionStatePort {
        state: std::sync::Mutex<EncryptionState>,
    }

    impl MockEncryptionStatePort {
        fn new(state: EncryptionState) -> Self {
            Self {
                state: std::sync::Mutex::new(state),
            }
        }
    }

    #[async_trait::async_trait]
    impl EncryptionStatePort for MockEncryptionStatePort {
        async fn load_state(
            &self,
        ) -> Result<EncryptionState, uc_core::security::state::EncryptionStateError> {
            Ok(*self.state.lock().unwrap())
        }

        async fn persist_initialized(
            &self,
        ) -> Result<(), uc_core::security::state::EncryptionStateError> {
            *self.state.lock().unwrap() = EncryptionState::Initialized;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_execute_returns_default_when_no_state() {
        let onboarding_mock = Arc::new(MockOnboardingStatePort::new(OnboardingState::default()));
        let encryption_mock =
            Arc::new(MockEncryptionStatePort::new(EncryptionState::Uninitialized));

        let use_case = GetOnboardingState::new(onboarding_mock, encryption_mock);
        let result = use_case.execute().await.unwrap();

        assert!(!result.has_completed);
        assert!(!result.encryption_password_set);
        assert!(result.device_registered); // Always true
    }

    #[tokio::test]
    async fn test_execute_when_onboarding_completed() {
        let onboarding_state = OnboardingState {
            has_completed: true,
            ..Default::default()
        };
        let onboarding_mock = Arc::new(MockOnboardingStatePort::new(onboarding_state));
        let encryption_mock =
            Arc::new(MockEncryptionStatePort::new(EncryptionState::Uninitialized));

        let use_case = GetOnboardingState::new(onboarding_mock, encryption_mock);
        let result = use_case.execute().await.unwrap();

        assert!(result.has_completed);
        assert!(!result.encryption_password_set);
        assert!(result.device_registered);
    }

    #[tokio::test]
    async fn test_execute_when_encryption_initialized() {
        let onboarding_mock = Arc::new(MockOnboardingStatePort::new(OnboardingState::default()));
        let encryption_mock = Arc::new(MockEncryptionStatePort::new(EncryptionState::Initialized));

        let use_case = GetOnboardingState::new(onboarding_mock, encryption_mock);
        let result = use_case.execute().await.unwrap();

        assert!(!result.has_completed);
        assert!(result.encryption_password_set);
        assert!(result.device_registered);
    }

    #[tokio::test]
    async fn test_execute_when_all_completed() {
        let onboarding_state = OnboardingState {
            has_completed: true,
            ..Default::default()
        };
        let onboarding_mock = Arc::new(MockOnboardingStatePort::new(onboarding_state));
        let encryption_mock = Arc::new(MockEncryptionStatePort::new(EncryptionState::Initialized));

        let use_case = GetOnboardingState::new(onboarding_mock, encryption_mock);
        let result = use_case.execute().await.unwrap();

        assert!(result.has_completed);
        assert!(result.encryption_password_set);
        assert!(result.device_registered);
    }

    #[tokio::test]
    async fn test_from_ports() {
        let onboarding_mock = Arc::new(MockOnboardingStatePort::new(OnboardingState::default()))
            as Arc<dyn OnboardingStatePort>;
        let encryption_mock = Arc::new(MockEncryptionStatePort::new(EncryptionState::Uninitialized))
            as Arc<dyn EncryptionStatePort>;

        let use_case =
            GetOnboardingState::from_ports(onboarding_mock.clone(), encryption_mock.clone());
        let result = use_case.execute().await.unwrap();

        assert!(!result.has_completed);
        assert!(!result.encryption_password_set);
        assert!(result.device_registered);
    }

    #[tokio::test]
    async fn test_device_registered_always_true() {
        // Test that device_registered is always true regardless of state
        let onboarding_mock = Arc::new(MockOnboardingStatePort::new(OnboardingState {
            device_registered: false, // This field is ignored
            ..Default::default()
        }));
        let encryption_mock =
            Arc::new(MockEncryptionStatePort::new(EncryptionState::Uninitialized));

        let use_case = GetOnboardingState::new(onboarding_mock, encryption_mock);
        let result = use_case.execute().await.unwrap();

        // device_registered should still be true in the DTO
        assert!(result.device_registered);
    }
}
