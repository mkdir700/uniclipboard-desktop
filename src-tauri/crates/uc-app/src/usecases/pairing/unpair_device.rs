use anyhow::Result;
use std::sync::Arc;

use uc_core::ports::{NetworkPort, PairedDeviceRepositoryPort};
use uc_core::PeerId;

pub struct UnpairDevice {
    network: Arc<dyn NetworkPort>,
    repo: Arc<dyn PairedDeviceRepositoryPort>,
}

impl UnpairDevice {
    pub fn new(network: Arc<dyn NetworkPort>, repo: Arc<dyn PairedDeviceRepositoryPort>) -> Self {
        Self { network, repo }
    }

    pub async fn execute(&self, peer_id: String) -> Result<()> {
        let peer = PeerId::from(peer_id.as_str());
        self.repo
            .delete(&peer)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete paired device: {}", e))?;
        self.network
            .unpair_device(peer_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to unpair device: {}", e))?;
        Ok(())
    }
}
