use crate::ids::{SessionId, SpaceId};

#[derive(Clone, Debug)]
pub struct SpaceAccessProofArtifact {
    pub pairing_session_id: SessionId,
    pub space_id: SpaceId,
    pub challenge_nonce: [u8; 32],
    pub proof_bytes: Vec<u8>,
}
