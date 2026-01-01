use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Wrapper for sensitive byte arrays that redacts in Debug output
#[derive(Clone, Serialize, Deserialize)]
pub struct SecretBytes(Vec<u8>);

impl SecretBytes {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

impl std::fmt::Debug for SecretBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SecretBytes")
            .field(&format!("[{} bytes]", self.0.len()))
            .finish()
    }
}

impl From<Vec<u8>> for SecretBytes {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl AsRef<[u8]> for SecretBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

pub struct Encryptor {
    cipher: Aes256Gcm,
}

impl Encryptor {
    // pub fn new() -> Self {
    //     let key = Aes256Gcm::generate_key(&mut OsRng);
    //     let cipher = Aes256Gcm::new(&key);
    //     Self { cipher }
    // }

    pub fn from_key(key: &[u8; 32]) -> Self {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        Self { cipher }
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("Failed to encrypt: {}", e))?;
        Ok([nonce.as_slice(), &ciphertext].concat())
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        if ciphertext.len() < 12 {
            return Err(anyhow::anyhow!("Invalid ciphertext length"));
        }
        let nonce = Nonce::from_slice(&ciphertext[..12]);
        self.cipher
            .decrypt(nonce, &ciphertext[12..])
            .map_err(|e| anyhow::anyhow!("Failed to decrypt: {}", e))
    }
}
