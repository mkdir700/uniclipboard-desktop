use std::path::PathBuf;

use async_trait::async_trait;
use tokio::fs;

use uc_core::{
    ports::security::encryption_state::{EncryptionStateMarkerPort, EncryptionStatePort},
    security::state::{EncryptionState, EncryptionStateError},
};

const ENCRYPTION_STATE_FILE: &str = ".initialized_encryption";

pub struct EncryptionStateRepository {
    path: PathBuf,
}

impl EncryptionStateRepository {
    pub fn new(config_dir: PathBuf) -> Self {
        Self {
            path: config_dir.join(ENCRYPTION_STATE_FILE),
        }
    }
}

#[async_trait]
impl EncryptionStateMarkerPort for EncryptionStateRepository {
    async fn exists(&self) -> Result<bool, EncryptionStateError> {
        Ok(self.path.exists())
    }

    async fn create(&self) -> Result<(), EncryptionStateError> {
        fs::write(&self.path, "{}")
            .await
            .map_err(|e| EncryptionStateError::PersistError(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl EncryptionStatePort for EncryptionStateRepository {
    async fn load_state(&self) -> Result<EncryptionState, EncryptionStateError> {
        // TODO: 需要识别出 Initializing 的情况
        match self.exists().await? {
            true => Ok(EncryptionState::Initialized),
            false => Ok(EncryptionState::Uninitialized),
        }
    }

    async fn persist_initialized(&self) -> Result<(), EncryptionStateError> {
        self.create().await?;
        Ok(())
    }
}
