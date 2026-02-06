//! File-based setup status repository
//!
//! This module provides a file-based implementation of the SetupStatusPort,
//! persisting setup status to a local JSON file in the application data directory.

use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uc_core::ports::SetupStatusPort;
use uc_core::setup::SetupStatus;

pub const DEFAULT_SETUP_STATUS_FILE: &str = ".setup_status";

pub struct FileSetupStatusRepository {
    status_file_path: PathBuf,
}

impl FileSetupStatusRepository {
    /// Create repository with custom file path
    pub fn new(status_file_path: PathBuf) -> Self {
        Self { status_file_path }
    }

    /// Create repository with base dir and filename
    pub fn with_base_dir(base_dir: PathBuf, filename: impl Into<String>) -> Self {
        Self {
            status_file_path: base_dir.join(filename.into()),
        }
    }

    /// Create repository with defaults
    pub fn with_defaults(base_dir: PathBuf) -> Self {
        Self {
            status_file_path: base_dir.join(DEFAULT_SETUP_STATUS_FILE),
        }
    }

    async fn ensure_parent_dir(&self) -> anyhow::Result<()> {
        if let Some(parent) = self.status_file_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl SetupStatusPort for FileSetupStatusRepository {
    async fn get_status(&self) -> anyhow::Result<SetupStatus> {
        if !self.status_file_path.exists() {
            return Ok(SetupStatus::default());
        }

        self.ensure_parent_dir().await?;
        let content = fs::read_to_string(&self.status_file_path).await?;

        if content.trim().is_empty() {
            return Ok(SetupStatus::default());
        }

        let status: SetupStatus = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse setup status: {e}"))?;

        Ok(status)
    }

    async fn set_status(&self, status: &SetupStatus) -> anyhow::Result<()> {
        self.ensure_parent_dir().await?;

        let json = serde_json::to_string_pretty(status)
            .map_err(|e| anyhow::anyhow!("Failed to serialize setup status: {e}"))?;

        let mut file = fs::File::create(&self.status_file_path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create status file: {e}"))?;

        file.write_all(json.as_bytes())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to write status file: {e}"))?;

        file.sync_all()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to sync status file: {e}"))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn get_status_returns_default_when_file_missing() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileSetupStatusRepository::new(temp_dir.path().join("missing.json"));

        let status = repo.get_status().await.unwrap();

        assert!(!status.has_completed);
    }

    #[tokio::test]
    async fn set_status_then_get_status_round_trip() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileSetupStatusRepository::new(temp_dir.path().join("status.json"));

        let status = SetupStatus {
            has_completed: true,
        };

        repo.set_status(&status).await.unwrap();
        let stored = repo.get_status().await.unwrap();

        assert_eq!(stored, status);
    }

    #[tokio::test]
    async fn empty_file_returns_default() {
        let temp_dir = TempDir::new().unwrap();
        let status_file = temp_dir.path().join("empty.json");

        fs::write(&status_file, "").await.unwrap();

        let repo = FileSetupStatusRepository::new(status_file);
        let status = repo.get_status().await.unwrap();

        assert!(!status.has_completed);
    }

    #[tokio::test]
    async fn invalid_json_returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let status_file = temp_dir.path().join("invalid.json");

        fs::write(&status_file, "{invalid json").await.unwrap();

        let repo = FileSetupStatusRepository::new(status_file);
        let result = repo.get_status().await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to parse"));
    }

    #[tokio::test]
    async fn with_defaults_uses_expected_path() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileSetupStatusRepository::with_defaults(temp_dir.path().to_path_buf());

        let expected_path = temp_dir.path().join(DEFAULT_SETUP_STATUS_FILE);
        assert_eq!(repo.status_file_path, expected_path);
    }
}
