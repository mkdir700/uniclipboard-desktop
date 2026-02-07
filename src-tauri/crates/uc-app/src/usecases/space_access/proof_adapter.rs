use async_trait::async_trait;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;
use tokio::sync::Mutex;

use uc_core::ids::{SessionId, SpaceId};
use uc_core::ports::space::ProofPort;
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
}

impl HmacProofAdapter {
    pub fn new() -> Self {
        Self {
            key_cache: Mutex::new(HashMap::new()),
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
}
