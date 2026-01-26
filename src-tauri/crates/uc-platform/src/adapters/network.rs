//! Placeholder network port implementation
//! 占位符网络端口实现

use anyhow::Result;
use async_trait::async_trait;
use libp2p::PeerId;
use uc_core::network::{ClipboardMessage, ConnectedPeer, DiscoveredPeer, NetworkEvent};
use uc_core::ports::IdentityStorePort;
use uc_core::ports::{NetworkControlPort, NetworkPort};

use crate::identity_store::load_or_create_identity;

/// Placeholder network port implementation
/// 占位符网络端口实现
#[derive(Debug, Clone)]
pub struct PlaceholderNetworkPort {
    local_peer_id: PeerId,
}

impl PlaceholderNetworkPort {
    pub fn new(identity_store: std::sync::Arc<dyn IdentityStorePort>) -> Result<Self> {
        let keypair = load_or_create_identity(identity_store.as_ref())
            .map_err(|e| anyhow::anyhow!("failed to load libp2p identity: {e}"))?;
        let local_peer_id = PeerId::from(keypair.public());
        Ok(Self { local_peer_id })
    }

    pub fn local_peer_id(&self) -> &PeerId {
        &self.local_peer_id
    }
}

#[async_trait]
impl NetworkPort for PlaceholderNetworkPort {
    // === Clipboard operations ===

    async fn send_clipboard(&self, _peer_id: &str, _encrypted_data: Vec<u8>) -> Result<()> {
        Err(anyhow::anyhow!(
            "NetworkPort::send_clipboard not implemented yet"
        ))
    }

    async fn broadcast_clipboard(&self, _encrypted_data: Vec<u8>) -> Result<()> {
        Err(anyhow::anyhow!(
            "NetworkPort::broadcast_clipboard not implemented yet"
        ))
    }

    async fn subscribe_clipboard(&self) -> Result<tokio::sync::mpsc::Receiver<ClipboardMessage>> {
        let (_tx, rx) = tokio::sync::mpsc::channel(1);
        Ok(rx)
    }

    // === Peer operations ===

    async fn get_discovered_peers(&self) -> Result<Vec<DiscoveredPeer>> {
        Ok(Vec::new())
    }

    async fn get_connected_peers(&self) -> Result<Vec<ConnectedPeer>> {
        Ok(Vec::new())
    }

    fn local_peer_id(&self) -> String {
        self.local_peer_id.to_string()
    }

    // === Pairing operations ===

    async fn initiate_pairing(&self, _peer_id: String, _device_name: String) -> Result<String> {
        Err(anyhow::anyhow!(
            "NetworkPort::initiate_pairing not implemented yet"
        ))
    }

    async fn send_pin_response(&self, _session_id: String, _pin_match: bool) -> Result<()> {
        Err(anyhow::anyhow!(
            "NetworkPort::send_pin_response not implemented yet"
        ))
    }

    async fn send_pairing_rejection(&self, _session_id: String, _peer_id: String) -> Result<()> {
        Err(anyhow::anyhow!(
            "NetworkPort::send_pairing_rejection not implemented yet"
        ))
    }

    async fn accept_pairing(&self, _session_id: String) -> Result<()> {
        Err(anyhow::anyhow!(
            "NetworkPort::accept_pairing not implemented yet"
        ))
    }

    async fn unpair_device(&self, _peer_id: String) -> Result<()> {
        Err(anyhow::anyhow!(
            "NetworkPort::unpair_device not implemented yet"
        ))
    }

    // === Event operations ===

    async fn subscribe_events(&self) -> Result<tokio::sync::mpsc::Receiver<NetworkEvent>> {
        let (_tx, rx) = tokio::sync::mpsc::channel(1);
        Ok(rx)
    }
}

#[async_trait]
impl NetworkControlPort for PlaceholderNetworkPort {
    async fn start_network(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct TestIdentityStore {
        data: Mutex<Option<Vec<u8>>>,
    }

    impl IdentityStorePort for TestIdentityStore {
        fn load_identity(&self) -> Result<Option<Vec<u8>>, uc_core::ports::IdentityStoreError> {
            let guard = self.data.lock().expect("lock test identity store");
            Ok(guard.clone())
        }

        fn store_identity(
            &self,
            identity: &[u8],
        ) -> Result<(), uc_core::ports::IdentityStoreError> {
            let mut guard = self.data.lock().expect("lock test identity store");
            *guard = Some(identity.to_vec());
            Ok(())
        }
    }

    #[test]
    fn local_peer_id_returns_typed_peer_id() {
        let adapter = PlaceholderNetworkPort::new(Arc::new(TestIdentityStore::default()))
            .expect("create placeholder network port");

        let peer_id: &PeerId = adapter.local_peer_id();

        assert!(!peer_id.to_string().is_empty());
    }
}
