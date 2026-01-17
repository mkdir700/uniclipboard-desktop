//! File-based onboarding state repository
//!
//! This module provides a file-based implementation of the OnboardingStatePort,
//! persisting onboarding state to a local JSON file in the application data directory.

use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uc_core::onboarding::OnboardingState;
use uc_core::ports::OnboardingStatePort;

pub const DEFAULT_ONBOARDING_STATE_FILE: &str = ".onboarding_state";

pub struct FileOnboardingStateRepository {
    state_file_path: PathBuf,
}

impl FileOnboardingStateRepository {
    /// Create repository with custom file path
    pub fn new(state_file_path: PathBuf) -> Self {
        Self { state_file_path }
    }

    /// Create repository with base dir and filename
    pub fn with_base_dir(base_dir: PathBuf, filename: impl Into<String>) -> Self {
        Self {
            state_file_path: base_dir.join(filename.into()),
        }
    }

    /// Create repository with defaults
    pub fn with_defaults(base_dir: PathBuf) -> Self {
        Self {
            state_file_path: base_dir.join(DEFAULT_ONBOARDING_STATE_FILE),
        }
    }

    async fn ensure_parent_dir(&self) -> anyhow::Result<()> {
        if let Some(parent) = self.state_file_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl OnboardingStatePort for FileOnboardingStateRepository {
    async fn get_state(&self) -> anyhow::Result<OnboardingState> {
        if !self.state_file_path.exists() {
            return Ok(OnboardingState::default());
        }

        self.ensure_parent_dir().await?;
        let content = fs::read_to_string(&self.state_file_path).await?;

        if content.trim().is_empty() {
            return Ok(OnboardingState::default());
        }

        let state: OnboardingState = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse onboarding state: {}", e))?;

        Ok(state)
    }

    async fn set_state(&self, state: &OnboardingState) -> anyhow::Result<()> {
        self.ensure_parent_dir().await?;

        let json = serde_json::to_string_pretty(state)
            .map_err(|e| anyhow::anyhow!("Failed to serialize onboarding state: {}", e))?;

        let mut file = fs::File::create(&self.state_file_path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create state file: {}", e))?;

        file.write_all(json.as_bytes())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to write state file: {}", e))?;

        file.sync_all()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to sync state file: {}", e))?;

        Ok(())
    }

    async fn reset(&self) -> anyhow::Result<()> {
        if self.state_file_path.exists() {
            fs::remove_file(&self.state_file_path).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_get_state_returns_default_when_file_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileOnboardingStateRepository::new(temp_dir.path().join("nonexistent.json"));

        let state = repo.get_state().await.unwrap();

        assert!(!state.has_completed);
        assert!(!state.encryption_password_set);
        assert!(!state.device_registered);
    }

    #[tokio::test]
    async fn test_set_state_and_get_state() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileOnboardingStateRepository::new(temp_dir.path().join("state.json"));

        let original_state = OnboardingState {
            has_completed: true,
            encryption_password_set: true,
            device_registered: true,
        };

        repo.set_state(&original_state).await.unwrap();
        let retrieved_state = repo.get_state().await.unwrap();

        assert_eq!(retrieved_state.has_completed, original_state.has_completed);
        assert_eq!(
            retrieved_state.encryption_password_set,
            original_state.encryption_password_set
        );
        assert_eq!(
            retrieved_state.device_registered,
            original_state.device_registered
        );
    }

    #[tokio::test]
    async fn test_reset_deletes_state_file() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileOnboardingStateRepository::new(temp_dir.path().join("state.json"));

        let state = OnboardingState {
            has_completed: true,
            encryption_password_set: true,
            device_registered: true,
        };

        repo.set_state(&state).await.unwrap();
        assert!(repo.get_state().await.unwrap().has_completed);

        repo.reset().await.unwrap();
        let reset_state = repo.get_state().await.unwrap();

        assert!(!reset_state.has_completed);
    }

    #[tokio::test]
    async fn test_is_completed() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileOnboardingStateRepository::new(temp_dir.path().join("state.json"));

        assert!(!repo.is_completed().await.unwrap());

        let state = OnboardingState {
            has_completed: true,
            encryption_password_set: false,
            device_registered: false,
        };
        repo.set_state(&state).await.unwrap();

        assert!(repo.is_completed().await.unwrap());
    }

    #[tokio::test]
    async fn test_with_defaults() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileOnboardingStateRepository::with_defaults(temp_dir.path().to_path_buf());

        let expected_path = temp_dir.path().join(DEFAULT_ONBOARDING_STATE_FILE);
        assert_eq!(repo.state_file_path, expected_path);
    }

    #[tokio::test]
    async fn test_with_base_dir() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileOnboardingStateRepository::with_base_dir(
            temp_dir.path().to_path_buf(),
            "custom_state.json",
        );

        let expected_path = temp_dir.path().join("custom_state.json");
        assert_eq!(repo.state_file_path, expected_path);
    }

    #[tokio::test]
    async fn test_empty_file_returns_default() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("empty.json");

        // Create an empty file
        fs::write(&state_file, "").await.unwrap();

        let repo = FileOnboardingStateRepository::new(state_file);
        let state = repo.get_state().await.unwrap();

        assert!(!state.has_completed);
        assert!(!state.encryption_password_set);
        assert!(!state.device_registered);
    }

    #[tokio::test]
    async fn test_invalid_json_returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("invalid.json");

        // Create a file with invalid JSON
        fs::write(&state_file, "{invalid json").await.unwrap();

        let repo = FileOnboardingStateRepository::new(state_file);
        let result = repo.get_state().await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to parse"));
    }

    #[tokio::test]
    async fn test_partial_state_update() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileOnboardingStateRepository::new(temp_dir.path().join("state.json"));

        // Set initial state
        let state1 = OnboardingState {
            has_completed: false,
            encryption_password_set: true,
            device_registered: false,
        };
        repo.set_state(&state1).await.unwrap();

        // Update to completion
        let mut state2 = repo.get_state().await.unwrap();
        state2.has_completed = true;
        state2.device_registered = true;
        repo.set_state(&state2).await.unwrap();

        let final_state = repo.get_state().await.unwrap();
        assert!(final_state.has_completed);
        assert!(final_state.encryption_password_set);
        assert!(final_state.device_registered);
    }
}
