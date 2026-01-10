//! PIN hashing module for secure pairing verification
//!
//! Uses Argon2id for secure password-based key derivation to prevent
//! brute force and dictionary attacks on pairing PINs.
//!
//! # Encoding Format
//!
//! PIN hashes are encoded as: `{version||salt||hash}`
//! - version: 1 byte (0x01 = Argon2id)
//! - salt: 16 bytes
//! - hash: 32 bytes
//! Total: 49 bytes

use anyhow::{anyhow, ensure, Result};
use argon2::{
    password_hash::{rand_core::OsRng, Salt, SaltString},
    Algorithm, Argon2, Params, Version,
};
use subtle::ConstantTimeEq;

/// Current version of the PIN hash encoding format
pub const HASH_VERSION: u8 = 0x01;

/// Size of the salt in bytes
pub const SALT_SIZE: usize = 16;

/// Size of the hash output in bytes
pub const HASH_SIZE: usize = 32;

/// Total size of the encoded hash (version + salt + hash)
pub const ENCODED_SIZE: usize = 1 + SALT_SIZE + HASH_SIZE;

/// Argon2id parameters for PIN hashing
///
/// These parameters are chosen to provide strong security while maintaining
/// reasonable performance on modern hardware.
///
/// - Memory: 64 MiB - provides protection against GPU/ASIC attacks
/// - Time: 3 iterations - slows down brute force attempts
/// - Parallelism: 4 lanes - utilizes multiple CPU cores
/// - Output: 32 bytes - matches AES-256 key size
fn get_argon_params() -> Params {
    // SAFETY: These constants are valid Argon2 parameters
    unsafe { Params::new(65536, 3, 4, None).unwrap_unchecked() }
}

/// A verified encoded PIN hash containing version, salt, and derived key
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncodedPinHash {
    pub version: u8,
    pub salt: [u8; SALT_SIZE],
    pub hash: [u8; HASH_SIZE],
}

impl EncodedPinHash {
    /// Encode the hash components into a byte vector
    pub fn encode(&self) -> Vec<u8> {
        let mut encoded = Vec::with_capacity(ENCODED_SIZE);
        encoded.push(self.version);
        encoded.extend_from_slice(&self.salt);
        encoded.extend_from_slice(&self.hash);
        encoded
    }

    /// Decode a byte vector into hash components
    pub fn decode(encoded: &[u8]) -> Result<Self> {
        ensure!(
            encoded.len() == ENCODED_SIZE,
            "Invalid encoded hash length: expected {}, got {}",
            ENCODED_SIZE,
            encoded.len()
        );

        let version = encoded[0];
        ensure!(
            version == HASH_VERSION,
            "Unsupported hash version: {} (supported: {})",
            version,
            HASH_VERSION
        );

        let mut salt = [0u8; SALT_SIZE];
        salt.copy_from_slice(&encoded[1..1 + SALT_SIZE]);

        let mut hash = [0u8; HASH_SIZE];
        hash.copy_from_slice(&encoded[1 + SALT_SIZE..]);

        Ok(Self {
            version,
            salt,
            hash,
        })
    }
}

/// Hash a PIN using Argon2id with a random salt
///
/// # Arguments
/// * `pin` - The PIN string to hash (typically 6 digits)
///
/// # Returns
/// An encoded hash containing version, salt, and derived key (49 bytes)
///
/// # Example
/// ```rust,ignore
/// let encoded = hash_pin("123456")?;
/// assert_eq!(encoded.len(), 49);
/// ```
pub fn hash_pin(pin: &str) -> Result<Vec<u8>> {
    // Generate a cryptographically random salt
    let salt_bytes = generate_salt();

    // Hash the PIN using Argon2id
    let hash = argon2id_hash(pin, &salt_bytes)?;

    // Encode as {version||salt||hash}
    let encoded = EncodedPinHash {
        version: HASH_VERSION,
        salt: salt_bytes,
        hash,
    }
    .encode();

    Ok(encoded)
}

/// Verify a PIN against an encoded hash
///
/// # Arguments
/// * `pin` - The PIN string to verify
/// * `encoded_hash` - The encoded hash (49 bytes from [`hash_pin`])
///
/// # Returns
/// - `Ok(true)` if the PIN matches
/// - `Ok(false)` if the PIN does not match
/// - `Err(...)` if the encoded hash is invalid
///
/// # Example
/// ```rust,ignore
/// let encoded = hash_pin("123456")?;
/// assert!(verify_pin("123456", &encoded)?);
/// assert!(!verify_pin("654321", &encoded)?);
/// ```
pub fn verify_pin(pin: &str, encoded_hash: &[u8]) -> Result<bool> {
    let decoded = EncodedPinHash::decode(encoded_hash)?;

    // Compute hash of the provided PIN with the same salt
    let computed = argon2id_hash(pin, &decoded.salt)?;

    // Constant-time comparison to prevent timing attacks
    let matches = computed.ct_eq(&decoded.hash).into();

    Ok(matches)
}

/// Generate a cryptographically random salt
fn generate_salt() -> [u8; SALT_SIZE] {
    let salt_string = SaltString::generate(&mut OsRng);
    let salt_str = salt_string.as_str();
    let salt_bytes = salt_str.as_bytes();
    let mut result = [0u8; SALT_SIZE];
    let copy_len = salt_bytes.len().min(SALT_SIZE);
    result[..copy_len].copy_from_slice(&salt_bytes[..copy_len]);
    result
}

/// Hash a PIN using Argon2id with the given salt
///
/// This is the core KDF operation. The salt should be unique per PIN
/// to prevent precomputation attacks (rainbow tables).
fn argon2id_hash(pin: &str, salt: &[u8; SALT_SIZE]) -> Result<[u8; HASH_SIZE]> {
    let mut output = [0u8; HASH_SIZE];

    // Use Argon2id with our chosen parameters
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, get_argon_params());

    // hash_password_into expects salt as &[u8]
    argon
        .hash_password_into(pin.as_bytes(), salt, &mut output)
        .map_err(|e| anyhow!("Argon2id hashing failed: {}", e))?;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_pin_produces_correct_length() {
        let encoded = hash_pin("123456").unwrap();
        assert_eq!(encoded.len(), ENCODED_SIZE);
    }

    #[test]
    fn test_hash_pin_is_deterministic_with_same_salt() {
        let salt = [1u8; SALT_SIZE];
        let hash1 = argon2id_hash("123456", &salt).unwrap();
        let hash2 = argon2id_hash("123456", &salt).unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_pin_changes_with_different_pin() {
        let encoded1 = hash_pin("123456").unwrap();
        let decoded1 = EncodedPinHash::decode(&encoded1).unwrap();

        let encoded2 = hash_pin("654321").unwrap();
        let decoded2 = EncodedPinHash::decode(&encoded2).unwrap();

        // Different PINs should produce different hashes even with different salts
        assert_ne!(decoded1.hash, decoded2.hash);
    }

    #[test]
    fn test_verify_pin_correct() {
        let encoded = hash_pin("123456").unwrap();
        assert!(verify_pin("123456", &encoded).unwrap());
    }

    #[test]
    fn test_verify_pin_incorrect() {
        let encoded = hash_pin("123456").unwrap();
        assert!(!verify_pin("654321", &encoded).unwrap());
    }

    #[test]
    fn test_verify_pin_rejects_wrong_length() {
        let encoded = hash_pin("123456").unwrap();
        let truncated = &encoded[..ENCODED_SIZE - 1];
        assert!(verify_pin("123456", truncated).is_err());
    }

    #[test]
    fn test_verify_pin_rejects_invalid_version() {
        let mut encoded = hash_pin("123456").unwrap();
        encoded[0] = 0xFF; // Invalid version
        assert!(verify_pin("123456", &encoded).is_err());
    }

    #[test]
    fn test_encoded_pin_hash_roundtrip() {
        let original = EncodedPinHash {
            version: HASH_VERSION,
            salt: [42u8; SALT_SIZE],
            hash: [99u8; HASH_SIZE],
        };

        let encoded = original.encode();
        let decoded = EncodedPinHash::decode(&encoded).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_salt_uniqueness() {
        let salts = std::iter::repeat_with(generate_salt)
            .take(100)
            .collect::<Vec<_>>();

        // All salts should be unique (with extremely high probability)
        let unique_salts: std::collections::HashSet<_> = salts.iter().collect();
        assert_eq!(unique_salts.len(), 100);
    }

    #[test]
    fn test_hash_pin_empty_string() {
        // Empty PIN should still hash correctly
        let encoded = hash_pin("").unwrap();
        assert!(verify_pin("", &encoded).unwrap());
    }
}
