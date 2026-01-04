//! Network port adapter - bridges P2P infrastructure to NetworkPort

use async_trait::async_trait;
use uc_core::network::{ClipboardMessage, ConnectedPeer, DiscoveredPeer, NetworkEvent};
use uc_core::ports::NetworkPort;

/// Adapter that wraps P2PRuntime to implement NetworkPort
///
/// TODO: This is a placeholder implementation. In Phase 3, we will implement
/// the actual adapter that wraps the existing P2PRuntime from the infrastructure layer.
pub struct P2PNetworkAdapter {
    _private: (),
}

impl P2PNetworkAdapter {
    /// Create a new P2PNetworkAdapter
    ///
    /// TODO: This will accept the actual P2PRuntime in Phase 3
    pub fn new() -> Self {
        Self { _private: () }
    }
}

#[async_trait]
impl NetworkPort for P2PNetworkAdapter {
    // === Clipboard operations ===

    async fn send_clipboard(&self, _peer_id: &str, _encrypted_data: Vec<u8>) -> anyhow::Result<()> {
        // TODO: Implement in Phase 3
        Ok(())
    }

    async fn broadcast_clipboard(&self, _encrypted_data: Vec<u8>) -> anyhow::Result<()> {
        // TODO: Implement in Phase 3
        Ok(())
    }

    async fn subscribe_clipboard(&self) -> anyhow::Result<tokio::sync::mpsc::Receiver<ClipboardMessage>> {
        // TODO: Implement in Phase 3
        let (_tx, rx) = tokio::sync::mpsc::channel(100);
        Ok(rx)
    }

    // === Peer operations ===

    async fn get_discovered_peers(&self) -> anyhow::Result<Vec<DiscoveredPeer>> {
        // TODO: Implement in Phase 3
        Ok(Vec::new())
    }

    async fn get_connected_peers(&self) -> anyhow::Result<Vec<ConnectedPeer>> {
        // TODO: Implement in Phase 3
        Ok(Vec::new())
    }

    fn local_peer_id(&self) -> String {
        // TODO: Implement in Phase 3
        "local-peer-id".to_string()
    }

    // === Pairing operations ===

    async fn initiate_pairing(&self, _peer_id: String, _device_name: String) -> anyhow::Result<String> {
        // TODO: Implement in Phase 3
        Ok("session-id".to_string())
    }

    async fn send_pin_response(&self, _session_id: String, _pin_match: bool) -> anyhow::Result<()> {
        // TODO: Implement in Phase 3
        Ok(())
    }

    async fn send_pairing_rejection(&self, _session_id: String, _peer_id: String) -> anyhow::Result<()> {
        // TODO: Implement in Phase 3
        Ok(())
    }

    async fn accept_pairing(&self, _session_id: String) -> anyhow::Result<()> {
        // TODO: Implement in Phase 3
        Ok(())
    }

    async fn unpair_device(&self, _peer_id: String) -> anyhow::Result<()> {
        // TODO: Implement in Phase 3
        Ok(())
    }

    // === Event operations ===

    async fn subscribe_events(&self) -> anyhow::Result<tokio::sync::mpsc::Receiver<NetworkEvent>> {
        // TODO: Implement in Phase 3
        let (_tx, rx) = tokio::sync::mpsc::channel(100);
        Ok(rx)
    }
}
