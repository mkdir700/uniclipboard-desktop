//! P2P runtime module
//!
//! Manages all P2P-related components including NetworkManager, PairingManager, and P2pSync.

use anyhow::Result;
use chrono::Utc;
use libp2p::identity::Keypair;
use log::error;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, RwLock};

use crate::api::encryption::get_unified_encryptor;
use crate::api::event::{
    P2PPairingCompleteEventData, P2PPairingFailedEventData, P2PPairingRequestEventData,
    P2PPeerConnectionEvent, P2PPinReadyEventData,
};
use crate::domain::pairing::PairedPeer;
use crate::infrastructure::p2p::pairing::{PairingCommand, PairingManager};
use crate::infrastructure::p2p::{ConnectedPeer, DiscoveredPeer, NetworkCommand};
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
    /// Peer storage for managing paired devices
    peer_storage: Arc<PeerStorage>,
    /// Local peer ID
    local_peer_id: String,
    /// Local device name
    device_name: String,
    /// Discovered peers (thread-safe for queries)
    discovered_peers: Arc<RwLock<HashMap<String, DiscoveredPeer>>>,
    /// Connected peers tracking (peer_id -> ConnectedPeer)
    connected_peers: Arc<RwLock<HashMap<String, ConnectedPeer>>>,
}

impl P2PRuntime {
    /// Create a new P2P runtime
    pub async fn new(device_name: String, app_handle: AppHandle) -> Result<Self> {
        // Create channels for network communication
        let (network_cmd_tx, network_cmd_rx) = mpsc::channel(100);
        let (network_event_tx, mut network_event_rx) = mpsc::channel(100);

        // Generate libp2p keypair
        let local_key = Keypair::generate_ed25519();
        let local_peer_id = libp2p::PeerId::from(local_key.public()).to_string();

        // Create channels for pairing manager
        let (pairing_cmd_tx, pairing_cmd_rx) = mpsc::channel(100);

        // Clone app_handle for the event loop
        let app_handle_for_events = app_handle.clone();

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
        let pairing_device_name = device_name.clone();
        tokio::spawn(async move {
            let pairing_manager = PairingManager::new(
                pairing_network_cmd_tx,
                pairing_event_tx,
                pairing_cmd_rx,
                pairing_device_name,
            );
            pairing_manager.run().await;
        });

        // Create PeerStorage (kept separate for pairing management)
        let peer_storage = Arc::new(PeerStorage::new().expect("Failed to create PeerStorage"));

        // Get the unified encryptor (must be initialized before this point)
        let encryptor = match get_unified_encryptor().await {
            Some(encryptor) => {
                log::info!("Unified encryptor initialized successfully");
                encryptor
            }
            None => {
                let err_msg =
                    "Unified encryptor not initialized. Please set encryption password first.";
                log::error!("{}", err_msg);
                return Err(anyhow::anyhow!(err_msg));
            }
        };

        // Clone device_name for P2pSync (original will be stored in P2PRuntime)
        let device_name_for_sync = device_name.clone();

        // Create P2pSync with unified encryptor
        let p2p_sync = Arc::new(Libp2pSync::new(
            network_cmd_tx.clone(),
            device_name_for_sync,
            local_peer_id.clone(),
            encryptor,
        ));

        let discovered_peers = Arc::new(RwLock::new(HashMap::new()));
        let connected_peers = Arc::new(RwLock::new(HashMap::new()));

        // Spawn event monitoring loop
        let pairing_cmd_tx_clone = pairing_cmd_tx.clone();
        let peer_storage_clone = peer_storage.clone();
        let _p2p_sync_clone = p2p_sync.clone();
        let discovered_peers_clone = discovered_peers.clone();
        let connected_peers_clone = connected_peers.clone();

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
                        session_id,
                        peer_id,
                        request,
                    } => {
                        let device_name = request.device_name.clone();

                        // Send to pairing manager
                        let _ = pairing_cmd_tx_clone
                            .send(PairingCommand::HandleRequest {
                                peer_id: peer_id.clone(),
                                request,
                            })
                            .await;

                        // Emit event to frontend
                        let event_data = P2PPairingRequestEventData {
                            session_id,
                            peer_id,
                            device_name: Some(device_name),
                        };
                        if let Err(e) =
                            app_handle_for_events.emit("p2p-pairing-request", event_data)
                        {
                            log::error!("Failed to emit p2p-pairing-request event: {:?}", e);
                        }
                    }
                    NetworkEvent::PairingPinReady {
                        session_id,
                        pin,
                        peer_device_name,
                    } => {
                        // Send to pairing manager
                        let _ = pairing_cmd_tx_clone
                            .send(PairingCommand::HandlePinReady {
                                session_id: session_id.clone(),
                                pin: pin.clone(),
                                peer_device_name: peer_device_name.clone(),
                            })
                            .await;

                        // Emit event to frontend
                        let event_data = P2PPinReadyEventData {
                            session_id,
                            pin,
                            peer_device_name,
                        };
                        if let Err(e) = app_handle_for_events.emit("p2p-pin-ready", event_data) {
                            log::error!("Failed to emit p2p-pin-ready event: {:?}", e);
                        }
                    }
                    NetworkEvent::PairingResponseReceived {
                        session_id,
                        peer_id,
                        response,
                    } => {
                        // Send to pairing manager for verification
                        let (tx, _rx) = tokio::sync::oneshot::channel();
                        let _ = pairing_cmd_tx_clone
                            .send(PairingCommand::HandleResponse {
                                session_id: session_id.clone(),
                                response,
                                peer_device_name: peer_id.clone(), // We'll use peer_id as device name placeholder
                                respond_to: tx,
                            })
                            .await;
                    }
                    NetworkEvent::PairingComplete {
                        session_id,
                        peer_id,
                        peer_device_name,
                    } => {
                        // Save paired peer to independent PeerStorage
                        let paired_peer = PairedPeer {
                            peer_id: peer_id.clone(),
                            device_name: peer_device_name.clone(),
                            shared_secret: vec![], // Empty - no longer using ECDH shared secret
                            paired_at: Utc::now(),
                            last_seen: Some(Utc::now()),
                            last_known_addresses: vec![],
                        };
                        if let Err(e) = peer_storage_clone.save_peer(paired_peer) {
                            log::error!("Failed to save paired peer {}: {}", peer_id, e);
                        } else {
                            log::info!(
                                "Saved paired peer: {} (device: {})",
                                peer_id,
                                peer_device_name
                            );
                        }

                        // Emit event to frontend
                        let event_data = P2PPairingCompleteEventData {
                            session_id,
                            peer_id,
                            device_name: peer_device_name,
                        };
                        if let Err(e) =
                            app_handle_for_events.emit("p2p-pairing-complete", event_data)
                        {
                            log::error!("Failed to emit p2p-pairing-complete event: {:?}", e);
                        }
                    }
                    NetworkEvent::PairingFailed { session_id, error } => {
                        // Emit event to frontend
                        let event_data = P2PPairingFailedEventData { session_id, error };
                        if let Err(e) = app_handle_for_events.emit("p2p-pairing-failed", event_data)
                        {
                            log::error!("Failed to emit p2p-pairing-failed event: {:?}", e);
                        }
                    }
                    NetworkEvent::ClipboardReceived(msg) => {
                        // Forward to P2pSync
                        if let Err(e) = _p2p_sync_clone.handle_incoming_message(msg).await {
                            log::warn!("Failed to handle incoming clipboard message: {}", e);
                        }
                    }
                    NetworkEvent::PeerConnected(connected) => {
                        // Update connected peers tracking
                        let mut peers = connected_peers_clone.write().await;
                        peers.insert(connected.peer_id.clone(), connected.clone());
                        log::info!(
                            "Peer connected: {} ({})",
                            connected.peer_id,
                            connected.device_name
                        );

                        // Emit event to frontend
                        let event_data = P2PPeerConnectionEvent {
                            peer_id: connected.peer_id.clone(),
                            device_name: Some(connected.device_name.clone()),
                            connected: true,
                        };
                        if let Err(e) =
                            app_handle_for_events.emit("p2p-peer-connection-changed", event_data)
                        {
                            log::error!(
                                "Failed to emit p2p-peer-connection-changed event: {:?}",
                                e
                            );
                        }
                    }
                    NetworkEvent::PeerDisconnected(peer_id) => {
                        // Get device name from connected peers before removing
                        let device_name = {
                            let mut peers = connected_peers_clone.write().await;
                            peers.remove(&peer_id).map(|p| p.device_name)
                        };
                        log::info!("Peer disconnected: {}", peer_id);

                        // Emit event to frontend
                        let event_data = P2PPeerConnectionEvent {
                            peer_id: peer_id.clone(),
                            device_name,
                            connected: false,
                        };
                        if let Err(e) =
                            app_handle_for_events.emit("p2p-peer-connection-changed", event_data)
                        {
                            log::error!(
                                "Failed to emit p2p-peer-connection-changed event: {:?}",
                                e
                            );
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
            peer_storage,
            local_peer_id,
            device_name,
            discovered_peers,
            connected_peers,
        })
    }

    /// Unpair a device
    pub fn unpair_peer(&self, peer_id: &str) -> Result<()> {
        // Use the independent peer_storage instead of p2p_sync.peer_storage()
        self.peer_storage.remove_peer(peer_id)?;
        Ok(())
    }

    /// Get all paired peers
    pub fn get_paired_peers(&self) -> Result<Vec<PairedPeer>> {
        self.peer_storage.get_all_peers()
    }

    /// Get the local peer ID
    pub fn local_peer_id(&self) -> &str {
        &self.local_peer_id
    }

    /// Get the local device name
    pub fn device_name(&self) -> &str {
        &self.device_name
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

    /// Get connected peers
    pub async fn connected_peers(&self) -> HashMap<String, ConnectedPeer> {
        self.connected_peers.read().await.clone()
    }

    /// Check if a specific peer is connected
    pub async fn is_peer_connected(&self, peer_id: &str) -> bool {
        self.connected_peers.read().await.contains_key(peer_id)
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
