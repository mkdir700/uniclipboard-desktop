//! P2P service - High-level P2P functionality
//!
//! This service provides a simplified interface to P2P operations,
//! removing the complexity of command channels and Actor patterns.

use crate::api::event::{
    P2PPairingCompleteEventData, P2PPairingFailedEventData, P2PPairingRequestEventData,
    P2PPeerConnectionEvent, P2PPinReadyEventData,
};
use crate::application::device_service;
use crate::domain::pairing::PairedPeer;
use crate::error::{AppError, Result};
use crate::infrastructure::p2p::{
    ConnectedPeer, DiscoveredPeer, NetworkCommand, NetworkEvent, NetworkManager,
};
use crate::infrastructure::storage::peer_storage::PeerStorage;
use chrono::Utc;
use libp2p::identity::Keypair;
use log::{error, info, warn};
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::AppHandle;
use tokio::sync::{mpsc, RwLock};

/// PIN length for pairing verification
const PIN_LENGTH: usize = 6;

/// Active pairing session state
#[derive(Clone)]
struct PairingSession {
    pub session_id: String,
    pub peer_id: String,
    pub local_device_name: String,
    pub peer_device_name: String,
    pub created_at: chrono::DateTime<Utc>,
    pub is_initiator: bool,
    pub peer_device_id: Option<String>,
    pin: Option<String>,
}

impl PairingSession {
    pub fn new(
        session_id: String,
        peer_id: String,
        local_device_name: String,
        peer_device_name: String,
        is_initiator: bool,
    ) -> Self {
        Self {
            session_id,
            peer_id,
            local_device_name,
            peer_device_name,
            created_at: Utc::now(),
            is_initiator,
            peer_device_id: None,
            pin: None,
        }
    }

    pub fn set_pin(&mut self, pin: String) {
        self.pin = Some(pin);
    }

    pub fn get_pin(&self) -> Option<&str> {
        self.pin.as_deref()
    }

    pub fn clear_pin(&mut self) {
        self.pin = None;
    }

    pub fn is_expired(&self) -> bool {
        let elapsed = Utc::now() - self.created_at;
        elapsed.num_seconds() > 300 // 5 minutes
    }
}

/// Local device info
#[derive(Clone, Debug)]
pub struct LocalDeviceInfo {
    pub peer_id: String,
    pub device_name: String,
}

/// Paired peer with connection status
#[derive(Clone, Debug)]
pub struct PairedPeerWithStatus {
    pub peer_id: String,
    pub device_name: String,
    pub paired_at: String,
    pub last_seen: Option<String>,
    pub last_known_addresses: Vec<String>,
    pub connected: bool,
}

/// P2P service - unified management of all P2P functionality
///
/// Design principles:
/// 1. Single responsibility: focused on P2P network and pairing management
/// 2. Simple interface: direct method calls, no command channel indirection
/// 3. Clear errors: use AppError for unified error handling
/// 4. Event-driven: keep Tauri event system for frontend notifications
pub struct P2PService {
    /// libp2p local Peer ID (changes each restart)
    local_peer_id: String,
    /// Device name
    device_name: String,
    /// 6-digit device ID (from database, stable)
    device_id: String,
    /// NetworkManager command sender
    network_cmd_tx: mpsc::Sender<NetworkCommand>,
    /// Pairing session storage (session_id -> session)
    pairing_sessions: Arc<RwLock<HashMap<String, PairingSession>>>,
    /// Discovered peers
    discovered_peers: Arc<RwLock<HashMap<String, DiscoveredPeer>>>,
    /// Connected peers
    connected_peers: Arc<RwLock<HashMap<String, ConnectedPeer>>>,
    /// Paired device storage
    peer_storage: Arc<PeerStorage>,
    /// Tauri AppHandle (for emitting events)
    app_handle: AppHandle,
}

impl P2PService {
    /// Create a new P2P service instance
    pub async fn new(
        device_name: String,
        app_handle: AppHandle,
    ) -> Result<Self> {
        // Create channels for network communication
        let (network_cmd_tx, network_cmd_rx) = mpsc::channel(100);
        let (network_event_tx, network_event_rx) = mpsc::channel(100);

        // Generate libp2p keypair
        let local_key = Keypair::generate_ed25519();
        let local_peer_id = libp2p::PeerId::from(local_key.public()).to_string();

        // Get our 6-digit device ID from database
        let device_manager = device_service::get_device_manager();
        let our_device_id = match device_manager.get_current_device() {
            Ok(Some(device)) => device.id,
            Ok(None) => {
                warn!("No current device found in database, using fallback ID");
                "000000".to_string()
            }
            Err(e) => {
                error!("Failed to get current device: {}", e);
                "000000".to_string()
            }
        };

        // Clone values for NetworkManager task
        let device_name_for_network = device_name.clone();
        let our_device_id_for_network = our_device_id.clone();

        // Spawn NetworkManager task
        tokio::spawn(async move {
            let mut network_manager = match NetworkManager::new(
                network_cmd_rx,
                network_event_tx.clone(),
                local_key,
                device_name_for_network,
                our_device_id_for_network,
            )
            .await
            {
                Ok(manager) => manager,
                Err(e) => {
                    error!("Failed to create NetworkManager: {}", e);
                    return;
                }
            };

            info!("Starting P2P NetworkManager");
            network_manager.run().await;
        });

        // Initialize storage and state
        let peer_storage = Arc::new(
            PeerStorage::new()
                .map_err(|e| AppError::internal(format!("Failed to create PeerStorage: {}", e)))?,
        );
        let pairing_sessions = Arc::new(RwLock::new(HashMap::new()));
        let discovered_peers = Arc::new(RwLock::new(HashMap::new()));
        let connected_peers = Arc::new(RwLock::new(HashMap::new()));

        // Clone for event handler
        let service_for_events = Self {
            local_peer_id: local_peer_id.clone(),
            device_name: device_name.clone(),
            device_id: our_device_id.clone(),
            network_cmd_tx: network_cmd_tx.clone(),
            pairing_sessions: pairing_sessions.clone(),
            discovered_peers: discovered_peers.clone(),
            connected_peers: connected_peers.clone(),
            peer_storage: peer_storage.clone(),
            app_handle: app_handle.clone(),
        };

        // Spawn event handling task
        tokio::spawn(async move {
            service_for_events
                .handle_network_events(network_event_rx)
                .await;
        });

        Ok(Self {
            local_peer_id,
            device_name,
            device_id: our_device_id,
            network_cmd_tx,
            pairing_sessions,
            discovered_peers,
            connected_peers,
            peer_storage,
            app_handle,
        })
    }

    // ========== Device Discovery ==========

    /// Get local device info
    pub fn local_device_info(&self) -> LocalDeviceInfo {
        LocalDeviceInfo {
            peer_id: self.local_peer_id.clone(),
            device_name: self.device_name.clone(),
        }
    }

    /// Get local peer ID
    pub fn local_peer_id(&self) -> &str {
        &self.local_peer_id
    }

    /// Get local device name
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

    // ========== Pairing ==========

    /// Generate a new session ID
    fn generate_session_id(&self) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time is before UNIX_EPOCH")
            .as_millis();
        let mut rng = rand::rng();
        format!("{}-{}", timestamp, rng.random::<u32>())
    }

    /// Generate a random PIN for pairing verification
    fn generate_pin(&self) -> String {
        let mut rng = rand::rng();
        (0..PIN_LENGTH)
            .map(|_| rng.random_range(0..10).to_string())
            .collect()
    }

    /// Initiate pairing (initiator side)
    pub async fn initiate_pairing(&self, peer_id: String) -> Result<String> {
        let session_id = self.generate_session_id();

        // Create session
        let session = PairingSession::new(
            session_id.clone(),
            peer_id.clone(),
            self.device_name.clone(),
            "Unknown".to_string(), // Will be updated in handle_pin_ready
            true, // is_initiator
        );

        self.pairing_sessions
            .write()
            .await
            .insert(session_id.clone(), session);

        // Send pairing request
        use crate::infrastructure::p2p::protocol::{PairingMessage, ProtocolMessage};

        let request = crate::infrastructure::p2p::protocol::PairingRequest {
            session_id: session_id.clone(),
            device_name: self.device_name.clone(),
            device_id: self.device_id.clone(),
            peer_id: self.local_peer_id.clone(),
        };

        let message = ProtocolMessage::Pairing(PairingMessage::Request(request));

        let message_bytes = message
            .to_bytes()
            .map_err(|e| AppError::p2p(format!("Failed to serialize pairing request: {}", e)))?;

        self.network_cmd_tx
            .send(NetworkCommand::SendPairingRequest {
                peer_id,
                message: message_bytes,
            })
            .await
            .map_err(|e| AppError::p2p(format!("Failed to send pairing request: {}", e)))?;

        Ok(session_id)
    }

    /// Accept pairing request (responder side)
    pub async fn accept_pairing(&self, session_id: &str) -> Result<()> {
        let (peer_id, peer_device_name, peer_device_id) = {
            let sessions = self.pairing_sessions.read().await;
            let session = sessions
                .get(session_id)
                .ok_or_else(|| AppError::validation("Session not found"))?;

            if session.is_expired() {
                return Err(AppError::validation("Pairing session expired"));
            }

            (
                session.peer_id.clone(),
                session.peer_device_name.clone(),
                session
                    .peer_device_id
                    .clone()
                    .ok_or_else(|| AppError::p2p("Peer device ID not found"))?,
            )
        };

        // Generate PIN
        let pin = self.generate_pin();

        info!(
            "Accepted pairing request {}, generated PIN: {}",
            session_id, pin
        );

        // Store PIN for verification
        {
            let mut sessions = self.pairing_sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                session.set_pin(pin.clone());
            }
        }

        // Emit p2p-pin-ready event to frontend
        let event_data = P2PPinReadyEventData {
            session_id: session_id.to_string(),
            pin: pin.clone(),
            peer_device_name: peer_device_name.clone(),
        };
        if let Err(e) = self.app_handle.emit("p2p-pin-ready", event_data) {
            error!("Failed to emit p2p-pin-ready event: {:?}", e);
        }

        // Send pairing challenge
        self.network_cmd_tx
            .send(NetworkCommand::SendPairingChallenge {
                peer_id,
                session_id: session_id.to_string(),
                pin,
                device_name: self.device_name.clone(),
                device_id: self.device_id.clone(),
                local_peer_id: self.local_peer_id.clone(),
            })
            .await
            .map_err(|e| AppError::p2p(format!("Failed to send pairing challenge: {}", e)))?;

        Ok(())
    }

    /// Verify PIN (initiator side)
    pub async fn verify_pin(&self, session_id: &str, pin_matches: bool) -> Result<()> {
        let session = {
            let sessions = self.pairing_sessions.read().await;
            sessions
                .get(session_id)
                .ok_or_else(|| AppError::validation("Session not found"))?
                .clone()
        };

        if session.is_expired() {
            return Err(AppError::validation("Pairing session expired"));
        }

        let peer_id = session.peer_id;

        if !pin_matches {
            warn!("PIN verification failed for session {}", session_id);

            // Send rejection response
            self.network_cmd_tx
                .send(NetworkCommand::SendPairingResponse {
                    peer_id,
                    session_id: session_id.to_string(),
                    pin_hash: vec![],
                    accepted: false,
                })
                .await
                .map_err(|e| AppError::p2p(format!("Failed to send pairing response: {}", e)))?;

            self.pairing_sessions.write().await.remove(session_id);
            return Ok(());
        }

        // Get the stored PIN and compute hash
        let pin = session
            .get_pin()
            .ok_or_else(|| AppError::p2p("PIN not found in session"))?;

        info!(
            "Computing Argon2id hash for PIN verification, session: {}, peer: {}",
            session_id, peer_id
        );

        let pin_hash = crate::infrastructure::p2p::pin_hash::hash_pin(pin)
            .map_err(|e| AppError::p2p(format!("Failed to compute PIN hash: {}", e)))?;

        // Clear PIN from memory
        {
            let mut sessions = self.pairing_sessions.write().await;
            if let Some(s) = sessions.get_mut(session_id) {
                s.clear_pin();
            }
        }

        info!(
            "Sending PairingResponse with Argon2id hash for session {}, peer: {}",
            session_id, peer_id
        );

        // Send PairingResponse with PIN hash
        self.network_cmd_tx
            .send(NetworkCommand::SendPairingResponse {
                peer_id,
                session_id: session_id.to_string(),
                pin_hash,
                accepted: true,
            })
            .await
            .map_err(|e| AppError::p2p(format!("Failed to send pairing response: {}", e)))?;

        Ok(())
    }

    /// Reject pairing request
    pub async fn reject_pairing(&self, session_id: &str, peer_id: String) -> Result<()> {
        info!("Rejected pairing request {} from peer {}", session_id, peer_id);

        self.network_cmd_tx
            .send(NetworkCommand::RejectPairing {
                peer_id,
                session_id: session_id.to_string(),
            })
            .await
            .map_err(|e| AppError::p2p(format!("Failed to send pairing rejection: {}", e)))?;

        self.pairing_sessions.write().await.remove(session_id);
        Ok(())
    }

    /// Unpair a device
    pub async fn unpair_device(&self, peer_id: &str) -> Result<()> {
        self.peer_storage
            .remove_peer(peer_id)
            .map_err(AppError::from)
    }

    // ========== Paired Device Management ==========

    /// Get paired peers
    pub async fn paired_peers(&self) -> Result<Vec<PairedPeer>> {
        self.peer_storage.get_all_peers().map_err(AppError::from)
    }

    /// Get paired peers with connection status
    pub async fn paired_peers_with_status(&self) -> Result<Vec<PairedPeerWithStatus>> {
        let paired = self.paired_peers().await?;
        let connected = self.connected_peers().await;

        Ok(paired
            .into_iter()
            .map(|p| {
                let is_connected = connected.contains_key(&p.peer_id);
                PairedPeerWithStatus {
                    peer_id: p.peer_id,
                    device_name: p.device_name,
                    paired_at: p.paired_at.to_rfc3339(),
                    last_seen: p.last_seen.map(|dt| dt.to_rfc3339()),
                    last_known_addresses: p.last_known_addresses,
                    connected: is_connected,
                }
            })
            .collect())
    }

    // ========== Internal Event Handling ==========

    /// Handle network events
    async fn handle_network_events(&self, mut event_rx: mpsc::Receiver<NetworkEvent>) {
        while let Some(event) = event_rx.recv().await {
            match event {
                NetworkEvent::PeerDiscovered(peer) => {
                    self.discovered_peers
                        .write()
                        .await
                        .insert(peer.peer_id.clone(), peer);
                }
                NetworkEvent::PeerLost(peer_id) => {
                    self.discovered_peers.write().await.remove(&peer_id);
                }
                NetworkEvent::PairingRequestReceived {
                    session_id,
                    peer_id,
                    request,
                } => {
                    self.handle_incoming_pairing_request(session_id, peer_id, request)
                        .await;
                }
                NetworkEvent::PairingPinReady {
                    session_id,
                    pin,
                    peer_device_name,
                    peer_device_id,
                } => {
                    self.handle_pin_ready(&session_id, pin, peer_device_name, peer_device_id)
                        .await;
                }
                NetworkEvent::PairingResponseReceived {
                    session_id,
                    peer_id,
                    response,
                } => {
                    self.handle_pairing_response(&session_id, peer_id, response).await;
                }
                NetworkEvent::PairingComplete {
                    session_id,
                    peer_id,
                    peer_device_id,
                    peer_device_name,
                } => {
                    self.handle_pairing_complete(
                        &session_id,
                        &peer_id,
                        &peer_device_id,
                        &peer_device_name,
                    )
                    .await;
                }
                NetworkEvent::PairingFailed { session_id, error } => {
                    self.handle_pairing_failed(&session_id, &error).await;
                }
                NetworkEvent::ClipboardReceived(msg) => {
                    info!(
                        "Handling incoming clipboard message from device '{}' (device-id: {})",
                        msg.origin_device_name, msg.origin_device_id
                    );
                    // Note: P2P clipboard sync is handled by Libp2pSync
                    // which is initialized in AppRuntime
                }
                NetworkEvent::PeerConnected(connected) => {
                    self.connected_peers
                        .write()
                        .await
                        .insert(connected.peer_id.clone(), connected.clone());
                    info!(
                        "Peer connected: {} ({})",
                        connected.peer_id, connected.device_name
                    );

                    // Emit event to frontend
                    let event_data = P2PPeerConnectionEvent {
                        peer_id: connected.peer_id.clone(),
                        device_name: Some(connected.device_name.clone()),
                        connected: true,
                    };
                    if let Err(e) = self
                        .app_handle
                        .emit("p2p-peer-connection-changed", event_data)
                    {
                        error!("Failed to emit p2p-peer-connection-changed event: {:?}", e);
                    }
                }
                NetworkEvent::PeerDisconnected(peer_id) => {
                    let device_name = {
                        let mut peers = self.connected_peers.write().await;
                        peers.remove(&peer_id).map(|p| p.device_name)
                    };
                    info!("Peer disconnected: {}", peer_id);

                    // Emit event to frontend
                    let event_data = P2PPeerConnectionEvent {
                        peer_id: peer_id.clone(),
                        device_name,
                        connected: false,
                    };
                    if let Err(e) = self
                        .app_handle
                        .emit("p2p-peer-connection-changed", event_data)
                    {
                        error!("Failed to emit p2p-peer-connection-changed event: {:?}", e);
                    }
                }
                _ => {}
            }
        }
    }

    /// Handle incoming pairing request
    async fn handle_incoming_pairing_request(
        &self,
        session_id: String,
        peer_id: String,
        request: crate::infrastructure::p2p::protocol::PairingRequest,
    ) {
        info!(
            "Received pairing request from peer {} (device: {}, device_id: {})",
            peer_id, request.device_name, request.device_id
        );

        // Create session
        let session = PairingSession::new(
            session_id.clone(),
            peer_id.clone(),
            self.device_name.clone(),
            request.device_name.clone(),
            false, // is_initiator (responder)
        );

        // Store peer's 6-digit device ID
        let mut session_with_id = session;
        session_with_id.peer_device_id = Some(request.device_id.clone());

        self.pairing_sessions
            .write()
            .await
            .insert(session_id.clone(), session_with_id);

        // Emit event to frontend
        let event_data = P2PPairingRequestEventData {
            session_id,
            peer_id,
            device_name: Some(request.device_name),
        };
        if let Err(e) = self
            .app_handle
            .emit("p2p-pairing-request", event_data)
        {
            error!("Failed to emit p2p-pairing-request event: {:?}", e);
        }
    }

    /// Handle PIN ready (initiator side)
    async fn handle_pin_ready(
        &self,
        session_id: &str,
        pin: String,
        peer_device_name: String,
        peer_device_id: String,
    ) {
        info!(
            "Received PIN challenge for session {}, peer device: {}, peer_device_id: {}",
            session_id, peer_device_name, peer_device_id
        );

        let mut sessions = self.pairing_sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            // Update peer device name when we receive the challenge
            session.peer_device_name = peer_device_name;
            // Store peer's 6-digit device ID
            session.peer_device_id = Some(peer_device_id);
            // Store PIN for later hash computation when user confirms
            session.set_pin(pin);
        }
    }

    /// Handle pairing response (responder side)
    async fn handle_pairing_response(
        &self,
        session_id: &str,
        _peer_id: String,
        response: crate::infrastructure::p2p::protocol::PairingResponse,
    ) {
        let (peer_id, local_device_name) = {
            let sessions = self.pairing_sessions.read().await;
            let session = match sessions.get(session_id) {
                Some(s) => s,
                None => {
                    warn!("Session not found for pairing response: {}", session_id);
                    return;
                }
            };

            if session.is_expired() {
                warn!("Pairing session expired: {}", session_id);
                return;
            }

            (
                session.peer_id.clone(),
                session.local_device_name.clone(),
            )
        };

        if !response.accepted {
            warn!(
                "Pairing rejected by initiator for session {}, peer: {}",
                session_id, peer_id
            );
            self.pairing_sessions.write().await.remove(session_id);

            // Send rejection confirm
            let _ = self
                .network_cmd_tx
                .send(NetworkCommand::SendPairingConfirm {
                    peer_id,
                    session_id: session_id.to_string(),
                    success: false,
                    device_name: local_device_name,
                    device_id: String::new(), // Empty for rejection
                })
                .await;

            return;
        }

        // Get the stored PIN for verification
        let pin = {
            let sessions = self.pairing_sessions.read().await;
            let session = match sessions.get(session_id) {
                Some(s) => s,
                None => {
                    warn!("Session not found for PIN verification: {}", session_id);
                    return;
                }
            };
            session.get_pin().map(|p| p.to_string())
        };

        let pin = match pin {
            Some(p) => p,
            None => {
                warn!("PIN not found in session: {}", session_id);
                return;
            }
        };

        info!(
            "Verifying Argon2id PIN hash for session {}, peer: {}",
            session_id, peer_id
        );

        // Verify the PIN hash
        let verified = match crate::infrastructure::p2p::pin_hash::verify_pin(&pin, &response.pin_hash)
        {
            Ok(v) => v,
            Err(e) => {
                warn!("Failed to verify PIN hash: {}", e);
                false
            }
        };

        if !verified {
            warn!(
                "PIN hash verification failed for session {}, peer: {}",
                session_id, peer_id
            );
            self.pairing_sessions.write().await.remove(session_id);

            // Send failure confirm
            let _ = self
                .network_cmd_tx
                .send(NetworkCommand::SendPairingConfirm {
                    peer_id,
                    session_id: session_id.to_string(),
                    success: false,
                    device_name: local_device_name,
                    device_id: String::new(), // Empty for failure
                })
                .await;

            return;
        }

        info!(
            "PIN hash verified successfully for session {}, peer: {}",
            session_id, peer_id
        );

        info!("Sending PairingConfirm with success=true for session {}", session_id);

        // Send success confirm
        let _ = self
            .network_cmd_tx
            .send(NetworkCommand::SendPairingConfirm {
                peer_id,
                session_id: session_id.to_string(),
                success: true,
                device_name: local_device_name,
                device_id: self.device_id.clone(),
            })
            .await;

        self.pairing_sessions.write().await.remove(session_id);
    }

    /// Handle pairing complete
    async fn handle_pairing_complete(
        &self,
        session_id: &str,
        peer_id: &str,
        peer_device_id: &str,
        peer_device_name: &str,
    ) {
        info!(
            "Pairing completed successfully for session {}, peer: {}",
            session_id, peer_id
        );

        // Save paired peer to PeerStorage
        let paired_peer = PairedPeer {
            peer_id: peer_id.to_string(),
            device_name: peer_device_name.to_string(),
            shared_secret: vec![],
            paired_at: Utc::now(),
            last_seen: Some(Utc::now()),
            last_known_addresses: vec![],
        };
        if let Err(e) = self.peer_storage.save_peer(paired_peer) {
            error!("Failed to save paired peer {}: {}", peer_id, e);
        } else {
            info!(
                "Saved paired peer: {} (device_id: {}, device: {})",
                peer_id, peer_device_id, peer_device_name
            );
        }

        // Also save to DeviceManager (database) for clipboard sync
        let device_manager = device_service::get_device_manager();
        use crate::domain::device::Device;
        match device_manager.get(peer_device_id) {
            Ok(Some(mut existing_device)) => {
                existing_device.peer_id = Some(peer_id.to_string());
                existing_device.device_name = Some(peer_device_name.to_string());
                existing_device.is_paired = true;
                existing_device.last_seen = Some(Utc::now().timestamp() as i32);
                if let Err(e) = device_manager.add(existing_device) {
                    error!("Failed to update paired device {}: {}", peer_device_id, e);
                }
            }
            Ok(None) => {
                let mut new_device = Device::new(
                    peer_device_id.to_string(),
                    None, // IP unknown
                    None,
                    None,
                );
                new_device.peer_id = Some(peer_id.to_string());
                new_device.device_name = Some(peer_device_name.to_string());
                new_device.is_paired = true;
                new_device.last_seen = Some(Utc::now().timestamp() as i32);
                if let Err(e) = device_manager.add(new_device) {
                    error!("Failed to add paired device {}: {}", peer_device_id, e);
                }
            }
            Err(e) => {
                error!("Failed to check device existence: {}", e);
            }
        }

        // Emit event to frontend
        let event_data = P2PPairingCompleteEventData {
            session_id: session_id.to_string(),
            peer_id: peer_id.to_string(),
            device_name: peer_device_name.to_string(),
        };
        if let Err(e) = self
            .app_handle
            .emit("p2p-pairing-complete", event_data)
        {
            error!("Failed to emit p2p-pairing-complete event: {:?}", e);
        }
    }

    /// Handle pairing failed
    async fn handle_pairing_failed(&self, session_id: &str, error: &str) {
        warn!("Pairing failed for session {}: {}", session_id, error);

        self.pairing_sessions.write().await.remove(session_id);

        // Emit event to frontend
        let event_data = P2PPairingFailedEventData {
            session_id: session_id.to_string(),
            error: error.to_string(),
        };
        if let Err(e) = self
            .app_handle
            .emit("p2p-pairing-failed", event_data)
        {
            error!("Failed to emit p2p-pairing-failed event: {:?}", e);
        }
    }
}
