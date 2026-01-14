//! Onboarding state port
//!
//! This port defines the contract for persisting and retrieving onboarding state.
//! Implementations are provided by the infrastructure layer (e.g., file-based storage).

use async_trait::async_trait;
use crate::onboarding::OnboardingState;

#[async_trait]
pub trait OnboardingStatePort: Send + Sync {
    /// Get current onboarding state
    async fn get_state(&self) -> anyhow::Result<OnboardingState>;

    /// Update onboarding state
    async fn set_state(&self, state: &OnboardingState) -> anyhow::Result<()>;

    /// Reset onboarding (for testing or re-onboarding)
    async fn reset(&self) -> anyhow::Result<()>;

    /// Check if onboarding is completed
    async fn is_completed(&self) -> anyhow::Result<bool> {
        Ok(self.get_state().await?.has_completed)
    }
}
