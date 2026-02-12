use async_trait::async_trait;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use uc_core::ids::{SessionId, SpaceId};
use uc_core::ports::space::ProofPort;
use uc_core::ports::EncryptionSessionPort;
use uc_core::security::model::MasterKey;
use uc_core::security::space_access::SpaceAccessProofArtifact;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ProofCacheKey {
    pairing_session_id: String,
    space_id: String,
    challenge_nonce: [u8; 32],
}

pub struct HmacProofAdapter {
    key_cache: Mutex<HashMap<ProofCacheKey, [u8; 32]>>,
    encryption_session: Option<Arc<dyn EncryptionSessionPort>>,
}

impl HmacProofAdapter {
    pub fn new() -> Self {
        Self {
            key_cache: Mutex::new(HashMap::new()),
            encryption_session: None,
        }
    }

    pub fn new_with_encryption_session(encryption_session: Arc<dyn EncryptionSessionPort>) -> Self {
        Self {
            key_cache: Mutex::new(HashMap::new()),
            encryption_session: Some(encryption_session),
        }
    }

    fn payload(
        pairing_session_id: &SessionId,
        space_id: &SpaceId,
        challenge_nonce: [u8; 32],
    ) -> Vec<u8> {
        let session = pairing_session_id.as_str().as_bytes();
        let space = space_id.as_ref().as_bytes();

        let mut payload =
            Vec::with_capacity(8 + session.len() + space.len() + challenge_nonce.len());
        payload.extend_from_slice(&(session.len() as u32).to_be_bytes());
        payload.extend_from_slice(session);
        payload.extend_from_slice(&(space.len() as u32).to_be_bytes());
        payload.extend_from_slice(space);
        payload.extend_from_slice(&challenge_nonce);
        payload
    }

    fn cache_key(
        pairing_session_id: &SessionId,
        space_id: &SpaceId,
        challenge_nonce: [u8; 32],
    ) -> ProofCacheKey {
        ProofCacheKey {
            pairing_session_id: pairing_session_id.as_str().to_string(),
            space_id: space_id.as_ref().to_string(),
            challenge_nonce,
        }
    }

    fn compute_hmac(
        pairing_session_id: &SessionId,
        space_id: &SpaceId,
        challenge_nonce: [u8; 32],
        master_key_bytes: &[u8],
    ) -> anyhow::Result<Vec<u8>> {
        let payload = Self::payload(pairing_session_id, space_id, challenge_nonce);
        let mut mac = HmacSha256::new_from_slice(master_key_bytes)?;
        mac.update(&payload);
        Ok(mac.finalize().into_bytes().to_vec())
    }
}

#[async_trait]
impl ProofPort for HmacProofAdapter {
    async fn build_proof(
        &self,
        pairing_session_id: &SessionId,
        space_id: &SpaceId,
        challenge_nonce: [u8; 32],
        master_key: &MasterKey,
    ) -> anyhow::Result<SpaceAccessProofArtifact> {
        let proof_bytes = Self::compute_hmac(
            pairing_session_id,
            space_id,
            challenge_nonce,
            master_key.as_bytes(),
        )?;

        let cache_key = Self::cache_key(pairing_session_id, space_id, challenge_nonce);
        self.key_cache.lock().await.insert(cache_key, master_key.0);

        Ok(SpaceAccessProofArtifact {
            pairing_session_id: pairing_session_id.clone(),
            space_id: space_id.clone(),
            challenge_nonce,
            proof_bytes,
        })
    }

    async fn verify_proof(
        &self,
        proof: &SpaceAccessProofArtifact,
        expected_nonce: [u8; 32],
    ) -> anyhow::Result<bool> {
        if proof.challenge_nonce != expected_nonce {
            return Ok(false);
        }

        let cache_key = Self::cache_key(
            &proof.pairing_session_id,
            &proof.space_id,
            proof.challenge_nonce,
        );
        let master_key = {
            let cache = self.key_cache.lock().await;
            cache.get(&cache_key).copied()
        };

        let master_key = if let Some(master_key) = master_key {
            Some(master_key)
        } else if let Some(encryption_session) = &self.encryption_session {
            match encryption_session.get_master_key().await {
                Ok(master_key) => {
                    let mut master_key_bytes = [0u8; 32];
                    master_key_bytes.copy_from_slice(master_key.as_bytes());
                    self.key_cache
                        .lock()
                        .await
                        .insert(cache_key, master_key_bytes);
                    Some(master_key_bytes)
                }
                Err(_) => None,
            }
        } else {
            None
        };

        let Some(master_key) = master_key else {
            return Ok(false);
        };

        let recomputed = Self::compute_hmac(
            &proof.pairing_session_id,
            &proof.space_id,
            proof.challenge_nonce,
            &master_key,
        )?;

        Ok(recomputed == proof.proof_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uc_core::security::model::EncryptionError;

    struct TestEncryptionSessionPort {
        master_key: Mutex<Option<MasterKey>>,
    }

    impl TestEncryptionSessionPort {
        fn with_master_key(master_key: MasterKey) -> Self {
            Self {
                master_key: Mutex::new(Some(master_key)),
            }
        }
    }

    #[async_trait]
    impl EncryptionSessionPort for TestEncryptionSessionPort {
        async fn is_ready(&self) -> bool {
            self.master_key.lock().await.is_some()
        }

        async fn get_master_key(&self) -> Result<MasterKey, EncryptionError> {
            self.master_key
                .lock()
                .await
                .clone()
                .ok_or_else(|| EncryptionError::NotInitialized)
        }

        async fn set_master_key(&self, master_key: MasterKey) -> Result<(), EncryptionError> {
            *self.master_key.lock().await = Some(master_key);
            Ok(())
        }

        async fn clear(&self) -> Result<(), EncryptionError> {
            *self.master_key.lock().await = None;
            Ok(())
        }
    }

    #[tokio::test]
    async fn build_and_verify_round_trip_succeeds() {
        let adapter = HmacProofAdapter::new();
        let session_id = SessionId::from("session-1");
        let space_id = SpaceId::from("space-1");
        let nonce = [7u8; 32];
        let master_key = MasterKey::from_bytes(&[11u8; 32]).expect("master key");

        let proof = adapter
            .build_proof(&session_id, &space_id, nonce, &master_key)
            .await
            .expect("build proof");

        let valid = adapter
            .verify_proof(&proof, nonce)
            .await
            .expect("verify proof");
        assert!(valid);
    }

    #[tokio::test]
    async fn verify_returns_false_for_tampered_proof() {
        let adapter = HmacProofAdapter::new();
        let session_id = SessionId::from("session-1");
        let space_id = SpaceId::from("space-1");
        let nonce = [9u8; 32];
        let master_key = MasterKey::from_bytes(&[22u8; 32]).expect("master key");

        let mut proof = adapter
            .build_proof(&session_id, &space_id, nonce, &master_key)
            .await
            .expect("build proof");
        if let Some(first) = proof.proof_bytes.first_mut() {
            *first ^= 0xFF;
        }

        let valid = adapter
            .verify_proof(&proof, nonce)
            .await
            .expect("verify proof");
        assert!(!valid);
    }

    #[tokio::test]
    async fn verify_returns_false_when_nonce_mismatch() {
        let adapter = HmacProofAdapter::new();
        let session_id = SessionId::from("session-1");
        let space_id = SpaceId::from("space-1");
        let nonce = [3u8; 32];
        let master_key = MasterKey::from_bytes(&[44u8; 32]).expect("master key");

        let proof = adapter
            .build_proof(&session_id, &space_id, nonce, &master_key)
            .await
            .expect("build proof");

        let valid = adapter
            .verify_proof(&proof, [8u8; 32])
            .await
            .expect("verify proof");
        assert!(!valid);
    }

    #[tokio::test]
    async fn verify_succeeds_with_encryption_session_fallback() {
        let builder = HmacProofAdapter::new();
        let session_id = SessionId::from("session-1");
        let space_id = SpaceId::from("space-1");
        let nonce = [4u8; 32];
        let master_key = MasterKey::from_bytes(&[55u8; 32]).expect("master key");

        let proof = builder
            .build_proof(&session_id, &space_id, nonce, &master_key)
            .await
            .expect("build proof");

        let verifier = HmacProofAdapter::new_with_encryption_session(Arc::new(
            TestEncryptionSessionPort::with_master_key(master_key),
        ));

        let valid = verifier
            .verify_proof(&proof, nonce)
            .await
            .expect("verify proof");
        assert!(valid);
    }
}
