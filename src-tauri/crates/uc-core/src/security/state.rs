use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EncryptionStateError {
    #[error("Failed to load encryption state: {0}")]
    LoadError(String),
    #[error("Failed to persist encryption state: {0}")]
    PersistError(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncryptionState {
    Uninitialized,
    Initializing,
    Initialized,
}
