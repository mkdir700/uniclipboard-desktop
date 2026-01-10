// uc-infra/src/services/hashing/sha256_hasher.rs
use anyhow::Result;
use sha2::{Digest, Sha256};
use uc_core::ports::ContentHashPort;

pub struct Sha256Hasher;

impl ContentHashPort for Sha256Hasher {
    fn hash_bytes(&self, bytes: &[u8]) -> Result<String> {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let out = hasher.finalize();
        Ok(hex::encode(out))
    }
}
