//! Onboarding domain models
//!
//! This module defines the core domain models for the onboarding flow,
//! which manages the initial user setup experience including encryption
//! password initialization and device registration.

/// Onboarding flow state
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OnboardingState {
    /// Whether onboarding has been completed
    pub has_completed: bool,
    /// Whether encryption password has been set
    pub encryption_password_set: bool,
    /// Whether device has been registered (auto-registered)
    pub device_registered: bool,
}

impl Default for OnboardingState {
    fn default() -> Self {
        Self {
            has_completed: false,
            encryption_password_set: false,
            device_registered: false,
        }
    }
}
