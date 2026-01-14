//! Simplified Onboarding Command (Temporary Implementation)
//!
//! This is a minimal implementation to bridge the gap during architecture migration.
//! The full implementation will be migrated to the new hexagonal architecture.

use serde::{Deserialize, Serialize};

/// Onboarding status information
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OnboardingStatus {
    pub has_completed: bool,
    pub vault_initialized: bool,
    pub device_registered: bool,
    pub encryption_password_set: bool,
}

/// Check onboarding status (simplified implementation)
///
/// NOTE: This is a temporary minimal implementation during architecture migration.
/// It always returns default values since the legacy dependencies are not yet
/// integrated with the new architecture.
#[tauri::command]
pub async fn check_onboarding_status() -> Result<OnboardingStatus, String> {
    // TODO: Integrate with new architecture's use cases
    // For now, return default values to allow the app to start
    Ok(OnboardingStatus {
        has_completed: false,
        vault_initialized: false,
        device_registered: false,
        encryption_password_set: false,
    })
}
