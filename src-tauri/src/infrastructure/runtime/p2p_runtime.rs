//! P2P runtime module
//!
//! Manages all P2P-related components including NetworkManager, PairingManager, and P2pSync.

use anyhow::Result;
use libp2p::identity::Keypair;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use crate::config::Setting;
use crate::infrastructure::p2p::pairing::{PairingCommand, PairingManager};
use crate::infrastructure::p2p::{DiscoveredPeer, NetworkCommand};
use crate::infrastructure::storage::peer_storage::PeerStorage;
use crate::infrastructure::sync::Libp2pSync;

/// P2P runtime - manages all P2P components
pub struct P2PRuntime {
    /// Sender for network commands
    network_cmd_tx: mpsc::Sender<NetworkCommand>,
    /// Sender for pairing commands
    pairing_cmd_tx: mpsc::Sender<PairingCommand>,
    /// P2P sync instance
    p2p_sync: Arc<Libp2pSync>,
    /// Local peer ID
    local_peer_id: String,
    /// Discovered peers (thread-safe for queries)
    discovered_peers: Arc<RwLock<HashMap<String, DiscoveredPeer>>>,
    /// Configuration
    config: Arc<Setting>,
}

impl P2PRuntime {
    /// Create a new P2P runtime
    pub async fn new(device_name: String, config: Arc<Setting>) -> Result<Self> {
        // Create channels for network communication
        let (network_cmd_tx, network_cmd_rx) = mpsc::channel(100);
        let (network_event_tx, mut network_event_rx) = mpsc::channel(100);

        // Generate libp2p keypair
        let local_key = Keypair::generate_ed25519();
        let local_peer_id = libp2p::PeerId::from(local_key.public()).to_string();

        // Create channels for pairing manager
        let (pairing_cmd_tx, pairing_cmd_rx) = mpsc::channel(100);

        // Spawn NetworkManager task
        let network_event_tx_clone = network_event_tx.clone();
        let device_name_for_network = device_name.clone();
        tokio::spawn(async move {
            let mut network_manager = crate::infrastructure::p2p::NetworkManager::new(
                network_cmd_rx,
                network_event_tx_clone,
                local_key,
                device_name_for_network,
            )
            .await
            .expect("Failed to create NetworkManager");

            log::info!("Starting P2P NetworkManager");
            network_manager.run().await;
        });

        // Spawn PairingManager actor
        let pairing_network_cmd_tx = network_cmd_tx.clone();
        let pairing_event_tx = network_event_tx.clone();
        tokio::spawn(async move {
            let pairing_manager =
                PairingManager::new(pairing_network_cmd_tx, pairing_event_tx, pairing_cmd_rx);
            pairing_manager.run().await;
        });

        // Create PeerStorage
        let peer_storage = Arc::new(PeerStorage::new().expect("Failed to create PeerStorage"));

        // Create P2pSync
        let p2p_sync = Arc::new(Libp2pSync::new(
            network_cmd_tx.clone(),
            device_name,
            local_peer_id.clone(),
            peer_storage,
        ));

        let discovered_peers = Arc::new(RwLock::new(HashMap::new()));

        // Spawn event monitoring loop
        let pairing_cmd_tx_clone = pairing_cmd_tx.clone();
        let _p2p_sync_clone = p2p_sync.clone();
        let discovered_peers_clone = discovered_peers.clone();

        tokio::spawn(async move {
            use crate::infrastructure::p2p::NetworkEvent;

            while let Some(event) = network_event_rx.recv().await {
                match event {
                    NetworkEvent::PeerDiscovered(peer) => {
                        let mut peers = discovered_peers_clone.write().await;
                        peers.insert(peer.peer_id.clone(), peer);
                    }
                    NetworkEvent::PeerLost(peer_id) => {
                        let mut peers = discovered_peers_clone.write().await;
                        peers.remove(&peer_id);
                    }
                    NetworkEvent::PairingRequestReceived {
                        peer_id, request, ..
                    } => {
                        let _ = pairing_cmd_tx_clone
                            .send(PairingCommand::HandleRequest { peer_id, request })
                            .await;
                    }
                    NetworkEvent::PairingPinReady {
                        session_id,
                        pin,
                        peer_device_name,
                        peer_public_key,
                    } => {
                        let _ = pairing_cmd_tx_clone
                            .send(PairingCommand::HandlePinReady {
                                session_id,
                                pin,
                                peer_device_name,
                                peer_public_key,
                            })
                            .await;
                    }
                    NetworkEvent::ClipboardReceived(msg) => {
                        // Forward to P2pSync
                        if let Err(e) = _p2p_sync_clone.handle_incoming_message(msg).await {
                            log::warn!("Failed to handle incoming clipboard message: {}", e);
                        }
                    }
                    _ => {}
                }
            }
        });

        Ok(Self {
            network_cmd_tx,
            pairing_cmd_tx,
            p2p_sync,
            local_peer_id,
            discovered_peers,
            config,
        })
    }

    /// Unpair a device
    pub fn unpair_peer(&self, peer_id: &str) -> Result<()> {
        self.p2p_sync.peer_storage().remove_peer(peer_id)?;
        Ok(())
    }

    /// Get the local peer ID
    pub fn local_peer_id(&self) -> &str {
        &self.local_peer_id
    }

    /// Get discovered peers
    pub async fn discovered_peers(&self) -> Vec<DiscoveredPeer> {
        self.discovered_peers
            .read()
            .await
            .values()
            .cloned()
            .collect()
    }

    /// Get the P2P command sender
    pub fn network_cmd_tx(&self) -> mpsc::Sender<NetworkCommand> {
        self.network_cmd_tx.clone()
    }

    /// Get the pairing command sender
    pub fn pairing_cmd_tx(&self) -> mpsc::Sender<PairingCommand> {
        self.pairing_cmd_tx.clone()
    }

    /// Get the P2pSync instance
    pub fn p2p_sync(&self) -> Arc<Libp2pSync> {
        self.p2p_sync.clone()
    }
}
