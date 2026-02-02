use super::protocol::{SpaceAccessOffer, SpaceAccessProof, SpaceAccessResult};
use crate::ids::SpaceId;

pub enum SpaceAccessAction {
    // Sponsor side
    SendOffer(SpaceAccessOffer),

    // Joiner side
    DeriveSpaceKeyFromKeyslot {
        keyslot_blob: Vec<u8>,
        passphrase: String,
    },
    SendProof(SpaceAccessProof),

    // Sponsor verify
    VerifyProof {
        proof: SpaceAccessProof,
    },

    // Result
    SendResult(SpaceAccessResult),

    // Persistence hooks (交给 uc-app/infras)
    PersistJoinerKeyslot {
        space_id: SpaceId,
        keyslot_blob: Vec<u8>,
    },
    PersistSponsorPairedDevice {
        space_id: SpaceId,
        peer_id: String,
    },

    // housekeeping
    StartTimer {
        ttl_secs: u64,
    },
    StopTimer,
}
