use async_trait::async_trait;

use crate::security::model::{
    EncryptedBlob, EncryptionAlgo, EncryptionError, KdfParams, Kek, MasterKey, Passphrase,
};

#[async_trait]
pub trait EncryptionPort: Send + Sync {
    // =========================
    // KDF / KEK
    // =========================

    /// Derive a Key Encryption Key (KEK) from user passphrase.
    ///
    /// Semantics:
    /// - Deterministic: same (passphrase, salt, params) => same KEK
    /// - NEVER returns MasterKey
    /// - Heavy operation (Argon2id / scrypt)
    async fn derive_kek(
        &self,
        passphrase: &Passphrase,
        salt: &[u8],
        kdf: &KdfParams,
    ) -> Result<Kek, EncryptionError>;

    // =========================
    // MasterKey wrapping
    // =========================

    /// Wrap (encrypt) a freshly generated MasterKey using KEK.
    ///
    /// Used only in:
    /// - InitializeEncryption
    /// - ChangePassphrase (future)
    async fn wrap_master_key(
        &self,
        kek: &Kek,
        master_key: &MasterKey,
        aead: EncryptionAlgo,
    ) -> Result<EncryptedBlob, EncryptionError>;

    /// Unwrap (decrypt) MasterKey using KEK.
    ///
    /// Failure mapping:
    /// - Wrong KEK        -> EncryptionError::WrongPassphrase
    /// - Corrupted blob  -> EncryptionError::CorruptedKeySlot
    async fn unwrap_master_key(
        &self,
        kek: &Kek,
        wrapped: &EncryptedBlob,
    ) -> Result<MasterKey, EncryptionError>;

    // =========================
    // Clipboard blob encryption
    // =========================

    /// Encrypt arbitrary plaintext blob using MasterKey.
    ///
    /// AAD MUST be provided by caller (repo / use case).
    async fn encrypt_blob(
        &self,
        master_key: &MasterKey,
        plaintext: &[u8],
        aad: &[u8],
        aead: EncryptionAlgo,
    ) -> Result<EncryptedBlob, EncryptionError>;

    /// Decrypt blob using MasterKey.
    ///
    /// Failure mapping:
    /// - Wrong key or wrong AAD -> EncryptionError::CorruptedBlob
    async fn decrypt_blob(
        &self,
        master_key: &MasterKey,
        encrypted: &EncryptedBlob,
        aad: &[u8],
    ) -> Result<Vec<u8>, EncryptionError>;
}
