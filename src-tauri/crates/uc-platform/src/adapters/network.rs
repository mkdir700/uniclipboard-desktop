//! Placeholder network port implementation
//! 占位符网络端口实现

use anyhow::Result;
use async_trait::async_trait;
use libp2p::PeerId;
use uc_core::network::{ClipboardMessage, ConnectedPeer, DiscoveredPeer, NetworkEvent};
use uc_core::ports::IdentityStorePort;
use uc_core::ports::NetworkPort;

use crate::identity_store::load_or_create_identity;

/// Placeholder network port implementation
/// 占位符网络端口实现
#[derive(Debug, Clone)]
pub struct PlaceholderNetworkPort {
    local_peer_id: String,
}

impl PlaceholderNetworkPort {
    pub fn new(identity_store: std::sync::Arc<dyn IdentityStorePort>) -> Result<Self> {
        let keypair = load_or_create_identity(identity_store.as_ref())
            .map_err(|e| anyhow::anyhow!("failed to load libp2p identity: {e}"))?;
        let local_peer_id = PeerId::from(keypair.public()).to_string();
        Ok(Self { local_peer_id })
    }

    pub fn local_peer_id(&self) -> &str {
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
        self.local_peer_id.clone()
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
