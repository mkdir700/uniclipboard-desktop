use anyhow::Result;
use async_trait::async_trait;

use crate::network::DiscoveredPeer;

/// Discovery port providing access to discovered peers.
#[async_trait]
pub trait DiscoveryPort: Send + Sync {
    /// List peers discovered via the underlying transport (e.g. mDNS).
    async fn list_discovered_peers(&self) -> Result<Vec<DiscoveredPeer>>;
}
