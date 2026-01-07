use async_trait::async_trait;

use crate::security::state::{EncryptionState, EncryptionStateError};

#[async_trait]
pub trait EncryptionStatePort: Send + Sync {
    async fn load_state(&self) -> Result<EncryptionState, EncryptionStateError>;
    async fn persist_initialized(&self) -> Result<(), EncryptionStateError>;
}

#[async_trait]
pub trait EncryptionStateMarkerPort: Send + Sync {
    async fn exists(&self) -> Result<bool, EncryptionStateError>;
    async fn create(&self) -> Result<(), EncryptionStateError>;
}
