//! Use case for checking if encryption is initialized
//! 检查加密是否已初始化的用例

use anyhow::Result;
use uc_core::ports::security::encryption_state::EncryptionStatePort;
use uc_core::security::state::EncryptionState;

/// Use case for checking encryption initialization status.
///
/// ## Behavior / 行为
/// - Loads encryption state from port
/// - Returns true if initialized, false otherwise
///
/// ## English
/// Checks whether encryption has been initialized by loading the
/// encryption state and comparing it to `EncryptionState::Initialized`.
pub struct IsEncryptionInitialized {
    encryption_state: std::sync::Arc<dyn EncryptionStatePort>,
}

impl IsEncryptionInitialized {
    /// Create a new IsEncryptionInitialized use case.
    pub fn new(encryption_state: std::sync::Arc<dyn EncryptionStatePort>) -> Self {
        Self { encryption_state }
    }

    /// Execute the use case.
    ///
    /// # Returns / 返回值
    /// - `Ok(true)` if encryption is initialized
    /// - `Ok(false)` if encryption is not initialized
    /// - `Err(e)` if loading state fails
    pub async fn execute(&self) -> Result<bool> {
        let state = self.encryption_state
            .load_state()
            .await?;
        Ok(state == EncryptionState::Initialized)
    }
}
