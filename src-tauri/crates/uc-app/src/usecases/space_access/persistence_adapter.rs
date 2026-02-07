use std::sync::Arc;

use async_trait::async_trait;

use uc_core::ids::{PeerId, SpaceId};
use uc_core::network::PairingState;
use uc_core::ports::paired_device_repository::PairedDeviceRepositoryPort;
use uc_core::ports::security::encryption_state::EncryptionStatePort;
use uc_core::ports::space::PersistencePort;

pub struct SpaceAccessPersistenceAdapter {
    encryption_state: Arc<dyn EncryptionStatePort>,
    paired_device_repo: Arc<dyn PairedDeviceRepositoryPort>,
}

impl SpaceAccessPersistenceAdapter {
    pub fn new(
        encryption_state: Arc<dyn EncryptionStatePort>,
        paired_device_repo: Arc<dyn PairedDeviceRepositoryPort>,
    ) -> Self {
        Self {
            encryption_state,
            paired_device_repo,
        }
    }
}

#[async_trait]
impl PersistencePort for SpaceAccessPersistenceAdapter {
    async fn persist_joiner_access(&mut self, _space_id: &SpaceId) -> anyhow::Result<()> {
        self.encryption_state.persist_initialized().await?;
        Ok(())
    }

    async fn persist_sponsor_access(
        &mut self,
        _space_id: &SpaceId,
        peer_id: &str,
    ) -> anyhow::Result<()> {
        self.paired_device_repo
            .set_state(&PeerId::from(peer_id), PairingState::Trusted)
            .await?;
        Ok(())
    }
}
