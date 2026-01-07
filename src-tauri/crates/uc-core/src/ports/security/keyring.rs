use crate::security::model::EncryptionError;
use crate::security::model::Kek;
use crate::security::model::KeyScope;

pub trait KeyringPort: Send + Sync {
    /// Load KEK from OS keyring.
    ///
    /// Error semantics:
    /// - KeyNotFound        : item does not exist
    /// - PermissionDenied  : keyring not accessible
    /// - KeyMaterialCorrupt: data exists but invalid
    fn load_kek(&self, scope: &KeyScope) -> Result<Kek, EncryptionError>;

    /// Store KEK into OS keyring.
    ///
    /// Requirements:
    /// - Idempotent (overwrite if exists)
    /// - Atomic at keyring level
    fn store_kek(&self, scope: &KeyScope, kek: &Kek) -> Result<(), EncryptionError>;

    /// Delete KEK from OS keyring.
    ///
    /// Used in reset / unrecoverable error flows.
    fn delete_kek(&self, scope: &KeyScope) -> Result<(), EncryptionError>;
}

#[cfg(test)]
mockall::mock! {
    pub Keyring {}

    impl KeyringPort for Keyring {
        fn load_kek(&self, scope: &KeyScope) -> Result<Kek, EncryptionError>;
        fn store_kek(&self, scope: &KeyScope, kek: &Kek) -> Result<(), EncryptionError>;
        fn delete_kek(&self, scope: &KeyScope) -> Result<(), EncryptionError>;
    }
}

