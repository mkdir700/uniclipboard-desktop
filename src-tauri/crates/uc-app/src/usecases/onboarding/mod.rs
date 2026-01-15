//! Onboarding use cases
//!
//! This module contains use cases for managing the onboarding flow,
//! which includes checking onboarding state, completing onboarding,
//! and initializing the onboarding process.

pub mod complete;
pub mod get_state;
pub mod initialize;

pub use complete::CompleteOnboarding;
pub use get_state::GetOnboardingState;
pub use initialize::InitializeOnboarding;

/// Data transfer object for onboarding state
#[derive(Debug, Clone, serde::Serialize)]
pub struct OnboardingStateDto {
    pub has_completed: bool,
    pub encryption_password_set: bool,
    pub device_registered: bool,
}
