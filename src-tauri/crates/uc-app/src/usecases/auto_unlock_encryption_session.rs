//! Auto-unlock encryption session on startup.
//!
//! This use case loads the MasterKey from persisted keyslot + KEK
//! and sets it in the EncryptionSessionPort for transparent encryption.

use std::sync::Arc;
use tracing::{info_span, info, Instrument};

use uc_core::{
    ports::{
        security::{
            encryption_state::EncryptionStatePort,
            key_scope::KeyScopePort,
        },
        EncryptionPort, EncryptionSessionPort, KeyMaterialPort,
    },
    security::{
        model::EncryptionError,
        state::EncryptionState,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum AutoUnlockError {
    #[error("encryption state check failed: {0}")]
    StateCheckFailed(String),

    #[error("key scope resolution failed: {0}")]
    ScopeFailed(String),

    #[error("failed to load keyslot: {0}")]
    KeySlotLoadFailed(#[source] EncryptionError),

    #[error("failed to load KEK from keyring: {0}")]
    KekLoadFailed(#[source] EncryptionError),

    #[error("keyslot has no wrapped master key")]
    MissingWrappedMasterKey,

    #[error("failed to unwrap master key: {0}")]
    UnwrapFailed(#[source] EncryptionError),

    #[error("failed to set master key in session: {0}")]
    SessionSetFailed(#[source] EncryptionError),
}

/// Use case for automatically unlocking encryption session on startup.
///
/// ## Behavior
///
/// - If encryption is **Uninitialized**: Returns `Ok(false)` (not unlocked, but not an error)
/// - If encryption is **Initialized**: Attempts to load and set MasterKey, returns `Ok(true)` on success
/// - Any failure during unlock returns an error
pub struct AutoUnlockEncryptionSession {
    encryption_state: Arc<dyn EncryptionStatePort>,
    key_scope: Arc<dyn KeyScopePort>,
    key_material: Arc<dyn KeyMaterialPort>,
    encryption: Arc<dyn EncryptionPort>,
    encryption_session: Arc<dyn EncryptionSessionPort>,
}

impl AutoUnlockEncryptionSession {
    pub fn new(
        encryption_state: Arc<dyn EncryptionStatePort>,
        key_scope: Arc<dyn KeyScopePort>,
        key_material: Arc<dyn KeyMaterialPort>,
        encryption: Arc<dyn EncryptionPort>,
        encryption_session: Arc<dyn EncryptionSessionPort>,
    ) -> Self {
        Self {
            encryption_state,
            key_scope,
            key_material,
            encryption,
            encryption_session,
        }
    }

    pub fn from_ports(
        encryption_state: Arc<dyn EncryptionStatePort>,
        key_scope: Arc<dyn KeyScopePort>,
        key_material: Arc<dyn KeyMaterialPort>,
        encryption: Arc<dyn EncryptionPort>,
        encryption_session: Arc<dyn EncryptionSessionPort>,
    ) -> Self {
        Self::new(encryption_state, key_scope, key_material, encryption, encryption_session)
    }

    /// Execute the auto-unlock flow.
    ///
    /// # Returns
    ///
    /// - `Ok(true)` - Session unlocked successfully
    /// - `Ok(false)` - Encryption not initialized (no unlock needed)
    /// - `Err(_)` - Unlock failed
    pub async fn execute(&self) -> Result<bool, AutoUnlockError> {
        let span = info_span!("usecase.auto_unlock_encryption_session.execute");

        async {
            info!("Checking encryption state for auto-unlock");

            // 1. Check encryption state
            let state = self.encryption_state.load_state().await
                .map_err(|e| AutoUnlockError::StateCheckFailed(e.to_string()))?;

            if state == EncryptionState::Uninitialized {
                info!("Encryption not initialized, skipping auto-unlock");
                return Ok(false);
            }

            info!("Encryption initialized, attempting auto-unlock");

            // 2. Get key scope
            let scope = self.key_scope.current_scope().await
                .map_err(|e| AutoUnlockError::ScopeFailed(e.to_string()))?;

            // 3. Load keyslot
            let keyslot = self.key_material.load_keyslot(&scope).await
                .map_err(AutoUnlockError::KeySlotLoadFailed)?;

            // 4. Get wrapped master key
            let wrapped_master_key = keyslot.wrapped_master_key
                .ok_or(AutoUnlockError::MissingWrappedMasterKey)?;

            // 5. Load KEK from keyring
            let kek = self.key_material.load_kek(&scope).await
                .map_err(AutoUnlockError::KekLoadFailed)?;

            // 6. Unwrap master key
            let master_key = self.encryption.unwrap_master_key(&kek, &wrapped_master_key.blob).await
                .map_err(AutoUnlockError::UnwrapFailed)?;

            // 7. Set master key in session
            self.encryption_session.set_master_key(master_key).await
                .map_err(AutoUnlockError::SessionSetFailed)?;

            info!("Auto-unlock completed successfully");
            Ok(true)
        }
        .instrument(span)
        .await
    }
}

#[cfg(test)]
mod tests {
    // Tests will be added in a separate task
}
