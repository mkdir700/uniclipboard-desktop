use std::sync::Arc;

use uc_core::ports::OnboardingStatePort;

/// Use case for completing onboarding.
///
/// Marks the onboarding process as complete in the persistent state.
pub struct CompleteOnboarding {
    onboarding_state: Arc<dyn OnboardingStatePort>,
}

impl CompleteOnboarding {
    /// Create a new CompleteOnboarding use case from trait objects.
    pub fn new(onboarding_state: Arc<dyn OnboardingStatePort>) -> Self {
        Self { onboarding_state }
    }

    /// Create a new CompleteOnboarding use case from cloned Arc<dyn Port> references.
    ///
    /// This is a convenience method for the UseCases accessor pattern.
    pub fn from_ports(onboarding_state: Arc<dyn OnboardingStatePort>) -> Self {
        Self::new(onboarding_state)
    }

    /// Mark onboarding as complete.
    pub async fn execute(&self) -> anyhow::Result<()> {
        let mut state = self.onboarding_state.get_state().await?;
        state.has_completed = true;
        self.onboarding_state.set_state(&state).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[tokio::test]
    async fn test_execute_marks_onboarding_as_complete() {
        let mock = Arc::new(MockOnboardingStatePort::new(OnboardingState::default()));
        let use_case = CompleteOnboarding::new(mock.clone());

        assert!(!mock.get_state().await.unwrap().has_completed);

        use_case.execute().await.unwrap();

        assert!(mock.get_state().await.unwrap().has_completed);
    }

    #[tokio::test]
    async fn test_execute_preserves_other_state_fields() {
        let initial_state = OnboardingState {
            has_completed: false,
            encryption_password_set: true,
            device_registered: true,
        };
        let mock = Arc::new(MockOnboardingStatePort::new(initial_state));
        let use_case = CompleteOnboarding::new(mock.clone());

        use_case.execute().await.unwrap();

        let final_state = mock.get_state().await.unwrap();
        assert!(final_state.has_completed);
        assert!(final_state.encryption_password_set);
        assert!(final_state.device_registered);
    }

    #[tokio::test]
    async fn test_from_ports() {
        let mock = Arc::new(MockOnboardingStatePort::new(OnboardingState::default()))
            as Arc<dyn OnboardingStatePort>;
        let use_case = CompleteOnboarding::from_ports(mock.clone());

        use_case.execute().await.unwrap();

        assert!(mock.get_state().await.unwrap().has_completed);
    }

    #[tokio::test]
    async fn test_execute_when_already_completed() {
        let initial_state = OnboardingState {
            has_completed: true,
            ..Default::default()
        };
        let mock = Arc::new(MockOnboardingStatePort::new(initial_state));
        let use_case = CompleteOnboarding::new(mock.clone());

        use_case.execute().await.unwrap();

        assert!(mock.get_state().await.unwrap().has_completed);
    }
}
