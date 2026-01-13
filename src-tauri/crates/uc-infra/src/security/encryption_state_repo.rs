//! File-based encryption state repository
//! 基于文件的加密状态仓库

use anyhow::Result;
use std::path::PathBuf;
use tokio::fs;
use uc_core::ports::security::encryption_state::EncryptionStatePort;
use uc_core::security::state::{EncryptionState, EncryptionStateError};

const ENCRYPTION_STATE_FILE: &str = ".initialized_encryption";

/// File-based encryption state repository
pub struct FileEncryptionStateRepository {
    state_file: PathBuf,
}

impl FileEncryptionStateRepository {
    pub fn new(config_dir: PathBuf) -> Self {
        let state_file = config_dir.join(ENCRYPTION_STATE_FILE);
        Self { state_file }
    }
}

#[async_trait::async_trait]
impl EncryptionStatePort for FileEncryptionStateRepository {
    async fn load_state(&self) -> Result<EncryptionState, EncryptionStateError> {
        if self.state_file.exists() {
            Ok(EncryptionState::Initialized)
        } else {
            Ok(EncryptionState::Uninitialized)
        }
    }

    async fn persist_initialized(&self) -> Result<(), EncryptionStateError> {
        fs::write(&self.state_file, b"1")
            .await
            .map_err(|e| EncryptionStateError::PersistError(e.to_string()))?;
        Ok(())
    }
}
