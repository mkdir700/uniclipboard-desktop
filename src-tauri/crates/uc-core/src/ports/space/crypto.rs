use crate::ids::SpaceId;
use crate::security::{MasterKey, SecretString};

use crate::security::KeySlot;

#[async_trait::async_trait]
pub trait CryptoPort: Send + Sync {
    async fn generate_nonce32(&self) -> [u8; 32];
    async fn export_keyslot_blob(&self, space_id: &SpaceId) -> anyhow::Result<KeySlot>;
    async fn derive_master_key_from_keyslot(
        &self,
        keyslot_blob: &[u8],
        passphrase: SecretString,
    ) -> anyhow::Result<MasterKey>;
}
