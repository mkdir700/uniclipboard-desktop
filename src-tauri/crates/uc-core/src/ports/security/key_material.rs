use crate::security::model::{EncryptionError, Kek, KeyScope, KeySlot};
use async_trait::async_trait;

#[async_trait]
pub trait KeyMaterialPort: Send + Sync {
    // -------- KEK (keyring) --------

    /// Load KEK from system keyring.
    /// - Err(KeyNotFound) if missing
    async fn load_kek(&self, scope: &KeyScope) -> Result<Kek, EncryptionError>;

    /// Store KEK into system keyring.
    /// - Should overwrite if exists (idempotent)
    async fn store_kek(&self, scope: &KeyScope, kek: &Kek) -> Result<(), EncryptionError>;

    /// Optional but useful for reset flows
    async fn delete_kek(&self, scope: &KeyScope) -> Result<(), EncryptionError>;

    // -------- KeySlot (disk/db) --------

    /// Load KeySlot from disk/db.
    /// - Err(KeyNotFound) if missing
    async fn load_keyslot(&self, scope: &KeyScope) -> Result<KeySlot, EncryptionError>;

    /// Store KeySlot to disk/db.
    /// - Should be atomic (write temp then rename / transaction)
    async fn store_keyslot(&self, keyslot: &KeySlot) -> Result<(), EncryptionError>;

    /// Optional reset flow
    async fn delete_keyslot(&self, scope: &KeyScope) -> Result<(), EncryptionError>;
}

#[cfg(test)]
mockall::mock! {
    pub KeyMaterial {}

    #[async_trait]
    impl KeyMaterialPort for KeyMaterial {
        async fn load_kek(&self, scope: &KeyScope) -> Result<Kek, EncryptionError>;
        async fn store_kek(&self, scope: &KeyScope, kek: &Kek) -> Result<(), EncryptionError>;
        async fn delete_kek(&self, scope: &KeyScope) -> Result<(), EncryptionError>;
        async fn load_keyslot(&self, scope: &KeyScope) -> Result<KeySlot, EncryptionError>;
        async fn store_keyslot(&self, keyslot: &KeySlot) -> Result<(), EncryptionError>;
        async fn delete_keyslot(&self, scope: &KeyScope) -> Result<(), EncryptionError>;
    }
}
