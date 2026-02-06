use uc_core::ids::SpaceId;
use uc_core::security::model::KeySlot;

#[derive(Clone, Debug)]
pub struct SpaceAccessOffer {
    pub space_id: SpaceId,
    pub keyslot: KeySlot,
    pub nonce: [u8; 32],
}

#[derive(Default)]
pub struct SpaceAccessContext {
    pub prepared_offer: Option<SpaceAccessOffer>,
}
