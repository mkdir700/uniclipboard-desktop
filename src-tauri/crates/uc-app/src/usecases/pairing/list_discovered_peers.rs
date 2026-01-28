use anyhow::Result;
use std::sync::Arc;

use uc_core::network::DiscoveredPeer;
use uc_core::ports::NetworkPort;

pub struct ListDiscoveredPeers {
    network: Arc<dyn NetworkPort>,
}

impl ListDiscoveredPeers {
    pub fn new(network: Arc<dyn NetworkPort>) -> Self {
        Self { network }
    }

    pub async fn execute(&self) -> Result<Vec<DiscoveredPeer>> {
        self.network
            .get_discovered_peers()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list discovered peers: {}", e))
    }
}
