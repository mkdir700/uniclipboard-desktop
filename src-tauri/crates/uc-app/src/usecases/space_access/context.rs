use uc_core::ids::SpaceId;
use uc_core::security::model::KeySlot;
use uc_core::security::space_access::SpaceAccessProofArtifact;
use uc_core::security::SecretString;

#[derive(Clone, Debug)]
pub struct SpaceAccessOffer {
    pub space_id: SpaceId,
    pub keyslot: KeySlot,
    pub nonce: [u8; 32],
}

#[derive(Clone, Debug)]
pub struct SpaceAccessJoinerOffer {
    pub space_id: SpaceId,
    pub keyslot_blob: Vec<u8>,
    pub challenge_nonce: [u8; 32],
}

#[derive(Default)]
pub struct SpaceAccessContext {
    pub prepared_offer: Option<SpaceAccessOffer>,
    pub joiner_offer: Option<SpaceAccessJoinerOffer>,
    pub joiner_passphrase: Option<SecretString>,
    pub proof_artifact: Option<SpaceAccessProofArtifact>,
    pub sponsor_peer_id: Option<String>,
}
