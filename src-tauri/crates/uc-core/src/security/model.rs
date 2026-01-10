//! Security / Encryption domain models.
//!
//! This module contains pure domain models used by encryption-related use cases.
//! It does NOT implement crypto algorithms, filesystem, keyring, etc.
//!
//! Design: WrappedMasterKey only.
//! - Passphrase -> KDF -> KEK (Key Encryption Key)
//! - KEK unwraps MasterKey (DEK)
//! - MasterKey encrypts/decrypts clipboard blobs

use chrono::{DateTime, Utc};
use rand::{rngs::OsRng, TryRngCore};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Versions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeySlotVersion {
    V1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncryptionFormatVersion {
    V1,
}

/// Algorithms
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KdfAlgorithm {
    Argon2id,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncryptionAlgo {
    /// Only supported XChaCha20-Poly1305 for now
    XChaCha20Poly1305,
}

impl fmt::Display for EncryptionAlgo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            EncryptionAlgo::XChaCha20Poly1305 => "xchacha20-poly1305",
        };
        write!(f, "{}", s)
    }
}

/// KDF params
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KdfParams {
    pub alg: KdfAlgorithm,
    pub params: KdfParamsV1,
}

impl KdfParams {
    pub fn for_initialization() -> Self {
        Self {
            alg: KdfAlgorithm::Argon2id,
            params: KdfParamsV1::default(),
        }
    }

    pub fn salt_len(&self) -> usize {
        match self.alg {
            KdfAlgorithm::Argon2id => 16,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KdfParamsV1 {
    /// Argon2id (example semantics):
    /// - mem_kib: memory cost in KiB
    /// - iters: time cost (iterations)
    /// - parallelism: lanes/threads
    ///
    /// Scrypt: you may reinterpret these or introduce a separate struct in V2.
    pub mem_kib: u32,
    pub iters: u32,
    pub parallelism: u32,
}

impl Default for KdfParamsV1 {
    fn default() -> Self {
        Self {
            mem_kib: 128 * 1024, // 128 MB
            iters: 3,
            parallelism: 4,
        }
    }
}

/// KeySlot (persistent; no passphrase, no plaintext keys)
/// =========================
///
/// KeySlot persists the parameters required to derive KEK from a passphrase,
///j and stores a wrapped (encrypted) MasterKey.
///
/// Unlock logic:
/// 1) derive KEK from passphrase + salt + kdf params
/// 2) unwrap MasterKey from wrapped_master_key
/// 3) store MasterKey in session
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeySlot {
    pub version: KeySlotVersion,

    pub scope: KeyScope,

    pub kdf: KdfParams,

    pub salt: Vec<u8>,

    /// MasterKey encrypted (wrapped) by KEK.
    pub wrapped_master_key: Option<WrappedMasterKey>,
}

impl KeySlot {
    pub fn draft_v1(scope: KeyScope) -> Result<Self, EncryptionError> {
        let kdf = KdfParams::for_initialization();
        let mut salt = vec![0u8; kdf.salt_len()];
        OsRng
            .try_fill_bytes(&mut salt)
            .map_err(|_| EncryptionError::CryptoFailure)?;

        Ok(Self {
            version: KeySlotVersion::V1,
            scope,
            kdf,
            salt,
            wrapped_master_key: None,
        })
    }

    pub fn finalize(self, wrapped_master_key: WrappedMasterKey) -> Self {
        Self {
            wrapped_master_key: Some(wrapped_master_key),
            ..self
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyScope {
    /// Profile ID (user profile)
    pub profile_id: String,
}

impl KeyScope {
    pub fn to_identifier(&self) -> String {
        format!("profile:{}", self.profile_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WrappedMasterKey {
    pub blob: EncryptedBlob,
}

/// Encrypted blob container (for disk storage / wrapped key)
/// =========================
///
/// This is a generic AEAD container used for:
/// - wrapping/unwrapping the MasterKey (KEK encrypts MasterKey)
/// - encrypting/decrypting clipboard blobs (MasterKey encrypts plaintext)
///
/// IMPORTANT:
/// - nonce length depends on algorithm
///   - XChaCha20-Poly1305: 24 bytes
///   - AES-256-GCM: 12 bytes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptedBlob {
    pub version: EncryptionFormatVersion,
    pub aead: EncryptionAlgo,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,

    /// Optional: store a short hash/fingerprint of AAD (NOT the AAD itself)
    /// to help debugging "wrong AAD" vs "wrong key" scenarios.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aad_fingerprint: Option<Vec<u8>>,
}

/// Secrets (newtypes)
/// =========================

/// The data-encryption key (DEK) used to encrypt clipboard blobs.
///
/// - 32 bytes is suitable for XChaCha20-Poly1305 / AES-256-GCM keys.
/// - Do NOT implement Serialize/Deserialize.
/// - Consider adding `zeroize` to wipe on drop in adapters.
#[derive(Clone, PartialEq, Eq)]
pub struct MasterKey(pub [u8; 32]);

impl fmt::Debug for MasterKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MasterKey([REDACTED])")
    }
}

impl MasterKey {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn generate() -> Result<Self, EncryptionError> {
        let mut buf = [0u8; Self::LEN];
        OsRng
            .try_fill_bytes(&mut buf)
            .map_err(|_| EncryptionError::CryptoFailure)?;
        Self::from_bytes(&buf)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, EncryptionError> {
        if bytes.len() != Self::LEN {
            return Err(EncryptionError::InvalidParameter(format!(
                "invalid MasterKey length: expected {}, got {}",
                Self::LEN,
                bytes.len()
            )));
        }
        let mut mk_bytes = [0u8; Self::LEN];
        mk_bytes.copy_from_slice(bytes);
        Ok(MasterKey(mk_bytes))
    }
}

/// Passphrase provided by user. Only used to derive KEK inside use cases.
/// Avoid storing this beyond the unlock/initialize flow.
#[derive(Clone)]
pub struct Passphrase(pub String);

impl fmt::Debug for Passphrase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Passphrase([REDACTED])")
    }
}

impl Passphrase {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

/// The key-encryption key (KEK) derived from passphrase via KDF.
/// KEK is used ONLY to wrap/unwrap the MasterKey.
///
/// Keep KEK ephemeral (avoid long-lived storage).
#[derive(Clone, PartialEq, Eq)]
pub struct Kek(pub [u8; 32]);

impl fmt::Debug for Kek {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Kek([REDACTED])")
    }
}

impl Kek {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, EncryptionError> {
        if bytes.len() != Self::LEN {
            return Err(EncryptionError::InvalidParameter(format!(
                "invalid KEK length: expected {}, got {}",
                Self::LEN,
                bytes.len()
            )));
        }
        let mut kek_bytes = [0u8; Self::LEN];
        kek_bytes.copy_from_slice(bytes);
        Ok(Kek(kek_bytes))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum KeySlotConvertError {
    #[error("wrapped master key is missing")]
    MissingWrappedMasterKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeySlotFile {
    pub version: KeySlotVersion,
    pub scope: KeyScope,
    pub kdf: KdfParams,
    pub salt: Vec<u8>,
    pub wrapped_master_key: EncryptedBlob,

    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

impl TryFrom<&KeySlot> for KeySlotFile {
    type Error = KeySlotConvertError;

    fn try_from(ks: &KeySlot) -> Result<Self, Self::Error> {
        let wrapped_master_key = ks
            .wrapped_master_key
            .clone()
            .ok_or(KeySlotConvertError::MissingWrappedMasterKey)?;

        Ok(KeySlotFile {
            version: ks.version,
            scope: ks.scope.clone(),
            kdf: ks.kdf.clone(),
            salt: ks.salt.clone(),
            wrapped_master_key: wrapped_master_key.blob.clone(),
            created_at: None,
            updated_at: None,
        })
    }
}

impl From<KeySlotFile> for KeySlot {
    fn from(ksf: KeySlotFile) -> Self {
        KeySlot {
            version: ksf.version,
            scope: ksf.scope,
            kdf: ksf.kdf,
            salt: ksf.salt,
            wrapped_master_key: Some(WrappedMasterKey {
                blob: ksf.wrapped_master_key,
            }),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("encryption is not initialized")]
    NotInitialized,

    #[error("encryption is locked")]
    Locked,

    #[error("wrong passphrase")]
    WrongPassphrase,

    #[error("unsupported keyslot version")]
    UnsupportedKeySlotVersion,

    #[error("unsupported blob format version")]
    UnsupportedBlobVersion,

    #[error("corrupted keyslot data")]
    CorruptedKeySlot,

    #[error("corrupted encrypted blob")]
    CorruptedBlob,

    #[error("internal crypto failure")]
    CryptoFailure,

    #[error("invalid key")]
    InvalidKey,

    #[error("invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("KDF operation failed")]
    KdfFailed,

    #[error("unsupported KDF algorithm")]
    UnsupportedKdfAlgorithm,

    #[error("encryption failed")]
    EncryptFailed,
    /// Keyring / Key Material errors

    #[error("key material not found")]
    KeyNotFound, // keyring 或 keyslot 缺失

    #[error("key material is corrupt")]
    KeyMaterialCorrupt, // keyslot 或 keyring 内容损坏/长度不对/反序列化失败

    #[error("other encryption error: {0}")]
    KeyringError(String),

    #[error("permission denied for key material access")]
    PermissionDenied, // keyring 权限/系统拒绝

    #[error("I/O failure during key material access")]
    IoFailure, // 文件/DB IO

    #[error("unsupported version for key material")]
    UnsupportedVersion, // keyslot/blob 版本不支持
}

impl EncryptedBlob {
    pub fn validate_basic(&self) -> Result<(), EncryptionError> {
        match (self.aead.clone(), self.nonce.len()) {
            (EncryptionAlgo::XChaCha20Poly1305, 24) => {}
            (alg, n) => {
                return Err(EncryptionError::InvalidParameter(format!(
                    "invalid nonce length for {:?}: {}",
                    alg, n
                )));
            }
        }

        if self.ciphertext.is_empty() {
            return Err(EncryptionError::InvalidParameter(
                "ciphertext is empty".into(),
            ));
        }

        match self.version {
            EncryptionFormatVersion::V1 => {}
        }

        Ok(())
    }
}

/// Helpers for KEK/MasterKey conversions (optional, keep domain clean).
impl MasterKey {
    pub const LEN: usize = 32;
}

impl Kek {
    pub const LEN: usize = 32;
}
