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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use tokio::sync::mpsc;
    use uc_core::network::{
        ClipboardMessage, ConnectedPeer, DiscoveredPeer, NetworkEvent, PairingMessage,
    };

    struct TestNetwork {
        peer_id: String,
    }

    #[async_trait]
    impl NetworkPort for TestNetwork {
        async fn send_clipboard(
            &self,
            _peer_id: &str,
            _encrypted_data: Vec<u8>,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        async fn broadcast_clipboard(&self, _encrypted_data: Vec<u8>) -> anyhow::Result<()> {
            Ok(())
        }

        async fn subscribe_clipboard(&self) -> anyhow::Result<mpsc::Receiver<ClipboardMessage>> {
            let (_tx, rx) = mpsc::channel(1);
            Ok(rx)
        }

        async fn get_discovered_peers(&self) -> anyhow::Result<Vec<DiscoveredPeer>> {
            Ok(Vec::new())
        }

        async fn get_connected_peers(&self) -> anyhow::Result<Vec<ConnectedPeer>> {
            Ok(Vec::new())
        }

        fn local_peer_id(&self) -> String {
            self.peer_id.clone()
        }

        async fn announce_device_name(&self, _device_name: String) -> anyhow::Result<()> {
            Ok(())
        }

        async fn open_pairing_session(
            &self,
            _peer_id: String,
            _session_id: String,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        async fn send_pairing_on_session(
            &self,
            _session_id: String,
            _message: PairingMessage,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        async fn close_pairing_session(
            &self,
            _session_id: String,
            _reason: Option<String>,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        async fn unpair_device(&self, _peer_id: String) -> anyhow::Result<()> {
            Ok(())
        }

        async fn subscribe_events(&self) -> anyhow::Result<mpsc::Receiver<NetworkEvent>> {
            let (_tx, rx) = mpsc::channel(1);
            Ok(rx)
        }
    }

    #[test]
    fn returns_local_peer_id_from_network() {
        let usecase = GetLocalPeerId::new(Arc::new(TestNetwork {
            peer_id: "peer-123".to_string(),
        }));

        let peer_id = usecase.execute();
        assert_eq!(peer_id, "peer-123");
    }
}
