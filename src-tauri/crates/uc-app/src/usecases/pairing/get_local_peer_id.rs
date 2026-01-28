use std::sync::Arc;

use uc_core::ports::NetworkPort;

pub struct GetLocalPeerId {
    network: Arc<dyn NetworkPort>,
}

impl GetLocalPeerId {
    pub fn new(network: Arc<dyn NetworkPort>) -> Self {
        Self { network }
    }

    pub fn execute(&self) -> String {
        self.network.local_peer_id()
    }
}
