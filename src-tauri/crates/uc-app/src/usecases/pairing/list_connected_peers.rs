use anyhow::Result;
use std::sync::Arc;

use uc_core::network::ConnectedPeer;
use uc_core::ports::NetworkPort;

pub struct ListConnectedPeers {
    network: Arc<dyn NetworkPort>,
}

impl ListConnectedPeers {
    pub fn new(network: Arc<dyn NetworkPort>) -> Self {
        Self { network }
    }

    pub async fn execute(&self) -> Result<Vec<ConnectedPeer>> {
        self.network
            .get_connected_peers()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list connected peers: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use tokio::sync::mpsc;
    use uc_core::network::{ClipboardMessage, DiscoveredPeer, NetworkEvent, PairingMessage};

    enum ConnectedOutcome {
        Ok(Vec<ConnectedPeer>),
        Err(String),
    }

    struct TestNetwork {
        outcome: ConnectedOutcome,
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
            match &self.outcome {
                ConnectedOutcome::Ok(peers) => Ok(peers.clone()),
                ConnectedOutcome::Err(message) => Err(anyhow::anyhow!(message.clone())),
            }
        }

        fn local_peer_id(&self) -> String {
            "peer-local".to_string()
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

    #[tokio::test]
    async fn returns_connected_peers_on_success() {
        let peers = vec![ConnectedPeer {
            peer_id: "peer-1".to_string(),
            device_name: "Desk".to_string(),
            connected_at: Utc::now(),
        }];

        let usecase = ListConnectedPeers::new(Arc::new(TestNetwork {
            outcome: ConnectedOutcome::Ok(peers.clone()),
        }));

        let result = usecase.execute().await.expect("list connected peers");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].peer_id, peers[0].peer_id);
        assert_eq!(result[0].device_name, peers[0].device_name);
    }

    #[tokio::test]
    async fn wraps_errors_with_context() {
        let usecase = ListConnectedPeers::new(Arc::new(TestNetwork {
            outcome: ConnectedOutcome::Err("boom".to_string()),
        }));

        let err = usecase.execute().await.expect_err("expected error");
        let message = err.to_string();
        assert!(message.contains("Failed to list connected peers"));
        assert!(message.contains("boom"));
    }
}
