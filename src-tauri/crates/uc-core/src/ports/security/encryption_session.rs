use crate::security::model::{EncryptionError, MasterKey};
use async_trait::async_trait;

#[async_trait]
pub trait EncryptionSessionPort: Send + Sync {
    /// Returns whether a MasterKey is currently available.
    async fn is_ready(&self) -> bool;

    /// Get a copy of current MasterKey.
    ///
    /// Notes:
    /// - Return a COPY (MasterKey is 32 bytes) to avoid lifetime & mutability complexity.
    /// - Implementation should zeroize internal storage when replaced/cleared.
    async fn get_master_key(&self) -> Result<MasterKey, EncryptionError>;

    /// Set or replace the MasterKey.
    ///
    /// Called by InitializeEncryption use case (and future rotation).
    async fn set_master_key(&self, master_key: MasterKey) -> Result<(), EncryptionError>;

    /// Clear MasterKey from memory.
    ///
    /// You might call this on app shutdown, or not call it at all—still useful for tests.
    async fn clear(&self) -> Result<(), EncryptionError>;
}
