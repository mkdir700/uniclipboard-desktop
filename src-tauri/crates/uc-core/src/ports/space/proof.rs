use crate::security::space_access::SpaceAccessProofArtifact;
use crate::{
    ids::{SessionId, SpaceId},
    security::MasterKey,
};

#[async_trait::async_trait]
pub trait ProofPort: Send {
    async fn build_proof(
        &self,
        pairing_session_id: &SessionId,
        space_id: &SpaceId,
        challenge_nonce: [u8; 32],
        master_key: &MasterKey,
    ) -> anyhow::Result<SpaceAccessProofArtifact>;

    async fn verify_proof(
        &self,
        proof: &SpaceAccessProofArtifact,
        expected_nonce: [u8; 32],
    ) -> anyhow::Result<bool>;
}
