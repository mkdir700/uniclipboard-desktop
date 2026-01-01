use chrono::Utc;
use futures::StreamExt;
use libp2p::{
    gossipsub, identify, mdns, noise,
    request_response::{self, ResponseChannel},
    swarm::SwarmEvent,
    tcp, yamux, Multiaddr, PeerId, Swarm,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use log::{debug, error, info, warn};

use super::behaviour::{
    PairingRequest as ReqPairingRequest, PairingResponse as ReqPairingResponse,
    UniClipboardBehaviour,
};
use super::events::{ConnectedPeer, DiscoveredPeer, NetworkEvent, NetworkStatus};
use super::protocol::{ClipboardMessage, DeviceAnnounceMessage, PairingMessage, ProtocolMessage};

/// Commands sent to NetworkManager
#[derive(Debug)]
pub enum NetworkCommand {
    StartListening,
    StopListening,
    SendPairingRequest {
        peer_id: String,
        message: Vec<u8>,
    },
    /// Send a pairing challenge (PIN) as a response to an incoming pairing request.
    SendPairingChallenge {
        peer_id: String,
        session_id: String,
        pin: String,
        device_name: String,
        public_key: Vec<u8>, // Our X25519 public key for ECDH
    },
    /// Reject a pairing request
    RejectPairing {
        peer_id: String,
        session_id: String,
    },
    /// Send pairing confirmation (after PIN verification on initiator side)
    SendPairingConfirm {
        peer_id: String,
        session_id: String,
        success: bool,
        shared_secret: Option<Vec<u8>>,
        device_name: String,
    },
    BroadcastClipboard {
        message: ClipboardMessage,
    },
    #[allow(dead_code)]
    GetPeers,
    /// Force reconnection to all discovered peers (used after app resume from background).
    /// Also attempts to reconnect to paired peers using their last-known addresses
    /// as fallback when mDNS hasn't rediscovered them yet.
    ReconnectPeers {
        /// Paired peers with their last-known addresses (from vault).
        /// Used as fallback when peer is not in discovered_peers.
        /// Format: Vec<(peer_id, Vec<address>)>
        paired_peer_addresses: Vec<(String, Vec<String>)>,
    },
    /// Re-emit PeerDiscovered event for a specific peer
    #[allow(dead_code)]
    RefreshPeer {
        peer_id: String,
    },
    /// Broadcast device name to all peers on the network.
    AnnounceDeviceName {
        device_name: String,
    },
}

/// Network manager that handles libp2p swarm event loop
pub struct NetworkManager {
    swarm: Swarm<UniClipboardBehaviour>,
    command_rx: mpsc::Receiver<NetworkCommand>,
    event_tx: mpsc::Sender<NetworkEvent>,
    discovered_peers: HashMap<PeerId, DiscoveredPeer>,
    connected_peers: HashMap<PeerId, ConnectedPeer>,
    pending_responses: HashMap<PeerId, ResponseChannel<ReqPairingResponse>>,
    /// Peers confirmed ready for broadcast (subscribed to clipboard topic).
    ready_peers: HashMap<PeerId, Instant>,
    /// Current device name (updated when settings change)
    device_name: String,
}

impl NetworkManager {
    pub async fn new(
        command_rx: mpsc::Receiver<NetworkCommand>,
        event_tx: mpsc::Sender<NetworkEvent>,
        local_key: libp2p::identity::Keypair,
        device_name: String,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let local_peer_id = PeerId::from(local_key.public());
        info!("Local peer ID: {}", local_peer_id);

        // Create swarm
        let swarm = libp2p::SwarmBuilder::with_existing_identity(local_key.clone())
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|_key| {
                UniClipboardBehaviour::new(local_peer_id, &local_key, &device_name)
                    .expect("Failed to create behaviour")
            })?
            .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        Ok(Self {
            swarm,
            command_rx,
            event_tx,
            discovered_peers: HashMap::new(),
            connected_peers: HashMap::new(),
            pending_responses: HashMap::new(),
            ready_peers: HashMap::new(),
            device_name,
        })
    }

    pub fn local_peer_id(&self) -> String {
        self.swarm.local_peer_id().to_string()
    }

    pub async fn run(&mut self) {
        // Subscribe to clipboard topic
        if let Err(e) = self.swarm.behaviour_mut().subscribe_clipboard() {
            error!("Failed to subscribe to clipboard topic: {}", e);
        }

        // Get preferred local address for listening
        // On macOS, binding to 0.0.0.0 can cause mDNS routing issues with multiple interfaces
        let local_ip = crate::utils::helpers::get_preferred_local_address();
        info!("P2P NetworkManager binding to {}:31773", local_ip);

        // Start listening on the preferred interface
        // Use a fixed port (31773) so cached addresses remain valid across app restarts.
        let listen_addr: Multiaddr = format!("/ip4/{}/tcp/31773", local_ip)
            .parse()
            .expect("Invalid listen address");
        if let Err(e) = self.swarm.listen_on(listen_addr) {
            error!("Failed to start listening: {}", e);
            let _ = self
                .event_tx
                .send(NetworkEvent::StatusChanged(NetworkStatus::Error(
                    e.to_string(),
                )))
                .await;
            return;
        }

        let _ = self
            .event_tx
            .send(NetworkEvent::StatusChanged(NetworkStatus::Connecting))
            .await;

        loop {
            tokio::select! {
                // Handle swarm events
                event = self.swarm.select_next_some() => {
                    self.handle_swarm_event(event).await;
                }

                // Handle commands
                Some(command) = self.command_rx.recv() => {
                    self.handle_command(command).await;
                }
            }
        }
    }

    async fn handle_swarm_event(
        &mut self,
        event: SwarmEvent<super::behaviour::UniClipboardBehaviourEvent>,
    ) {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("Listening on {}", address);
                let _ = self
                    .event_tx
                    .send(NetworkEvent::StatusChanged(NetworkStatus::Connected))
                    .await;
            }

            SwarmEvent::Behaviour(super::behaviour::UniClipboardBehaviourEvent::Mdns(event)) => {
                match event {
                    mdns::Event::Discovered(peers) => {
                        for (peer_id, addr) in peers {
                            debug!("mDNS discovered: {} at {}", peer_id, addr);

                            // Add to dial queue
                            if let Err(e) = self.swarm.dial(addr.clone()) {
                                warn!("Failed to dial {}: {}", peer_id, e);
                            }

                            // Track discovered peer
                            let discovered = DiscoveredPeer {
                                peer_id: peer_id.to_string(),
                                device_name: None,
                                addresses: vec![addr.to_string()],
                                discovered_at: Utc::now(),
                                is_paired: false,
                            };

                            self.discovered_peers.insert(peer_id, discovered.clone());
                            let _ = self
                                .event_tx
                                .send(NetworkEvent::PeerDiscovered(discovered))
                                .await;
                        }
                    }
                    mdns::Event::Expired(peers) => {
                        for (peer_id, _) in peers {
                            debug!("mDNS peer expired: {}", peer_id);
                            self.discovered_peers.remove(&peer_id);
                            let _ = self
                                .event_tx
                                .send(NetworkEvent::PeerLost(peer_id.to_string()))
                                .await;
                        }
                    }
                }
            }

            SwarmEvent::Behaviour(super::behaviour::UniClipboardBehaviourEvent::Gossipsub(
                event,
            )) => match event {
                gossipsub::Event::Message { message, .. } => {
                    match ProtocolMessage::from_bytes(&message.data) {
                        Ok(ProtocolMessage::Clipboard(clipboard_msg)) => {
                            debug!(
                                "Received clipboard message from {}",
                                clipboard_msg.origin_device_id
                            );
                            let _ = self
                                .event_tx
                                .send(NetworkEvent::ClipboardReceived(clipboard_msg))
                                .await;
                        }
                        Ok(ProtocolMessage::DeviceAnnounce(announce_msg)) => {
                            debug!(
                                "Received device announce from {}: {}",
                                announce_msg.peer_id, announce_msg.device_name
                            );

                            let _ = self
                                .event_tx
                                .send(NetworkEvent::PeerNameUpdated {
                                    peer_id: announce_msg.peer_id.clone(),
                                    device_name: announce_msg.device_name.clone(),
                                })
                                .await;

                            // Update local discovered_peers cache
                            if let Ok(pid) = announce_msg.peer_id.parse::<PeerId>() {
                                if let Some(discovered) = self.discovered_peers.get_mut(&pid) {
                                    let old_name = discovered.device_name.clone();
                                    discovered.device_name = Some(announce_msg.device_name.clone());

                                    if old_name != discovered.device_name {
                                        let _ = self
                                            .event_tx
                                            .send(NetworkEvent::PeerDiscovered(discovered.clone()))
                                            .await;
                                    }
                                }
                            }
                        }
                        Ok(msg) => {
                            debug!("Received non-clipboard message via gossipsub: {:?}", msg);
                        }
                        Err(e) => {
                            warn!("Failed to parse gossipsub message: {}", e);
                        }
                    }
                }
                gossipsub::Event::Subscribed { peer_id, topic } => {
                    info!("Peer {} subscribed to topic {}", peer_id, topic);

                    if topic.to_string().contains("clipboard") {
                        self.ready_peers.insert(peer_id, Instant::now());

                        let _ = self
                            .event_tx
                            .send(NetworkEvent::PeerReady {
                                peer_id: peer_id.to_string(),
                            })
                            .await;
                    }

                    // Announce our device name
                    let local_peer_id = self.swarm.local_peer_id().to_string();
                    let announce_msg = DeviceAnnounceMessage {
                        peer_id: local_peer_id,
                        device_name: self.device_name.clone(),
                        timestamp: Utc::now(),
                    };
                    let protocol_msg = ProtocolMessage::DeviceAnnounce(announce_msg);
                    if let Err(e) = self.swarm.behaviour_mut().publish_clipboard(&protocol_msg) {
                        debug!("Failed to announce device name: {}", e);
                    }
                }
                gossipsub::Event::Unsubscribed { peer_id, topic } => {
                    if topic.to_string().contains("clipboard") {
                        self.ready_peers.remove(&peer_id);

                        let _ = self
                            .event_tx
                            .send(NetworkEvent::PeerNotReady {
                                peer_id: peer_id.to_string(),
                            })
                            .await;
                    }
                }
                _ => {}
            },

            SwarmEvent::Behaviour(
                super::behaviour::UniClipboardBehaviourEvent::RequestResponse(event),
            ) => {
                match event {
                    request_response::Event::Message {
                        peer,
                        connection_id: _connection_id,
                        message,
                    } => {
                        match message {
                            request_response::Message::Request {
                                request, channel, ..
                            } => {
                                debug!("Received pairing message from {}", peer);

                                if let Ok(protocol_msg) =
                                    ProtocolMessage::from_bytes(&request.message)
                                {
                                    match protocol_msg {
                                        ProtocolMessage::Pairing(PairingMessage::Request(req)) => {
                                            self.pending_responses.remove(&peer);
                                            self.pending_responses.insert(peer, channel);

                                            let _ = self
                                                .event_tx
                                                .send(NetworkEvent::PairingRequestReceived {
                                                    session_id: req.session_id.clone(),
                                                    peer_id: peer.to_string(),
                                                    request: req,
                                                })
                                                .await;
                                        }
                                        ProtocolMessage::Pairing(PairingMessage::Confirm(
                                            confirm,
                                        )) => {
                                            // Get initiator's device name from the confirm message
                                            let initiator_device_name = confirm.sender_device_name.clone();

                                            if confirm.success {
                                                if let Some(shared_secret) =
                                                    confirm.shared_secret.clone()
                                                {
                                                    // Send ACK with responder's own device name
                                                    let ack = super::protocol::PairingConfirm {
                                                        session_id: confirm.session_id.clone(),
                                                        success: true,
                                                        shared_secret: Some(shared_secret.clone()),
                                                        error: None,
                                                        sender_device_name: self.device_name.clone(),
                                                    };
                                                    let ack_msg = ProtocolMessage::Pairing(
                                                        PairingMessage::Confirm(ack),
                                                    );
                                                    if let Ok(message) = ack_msg.to_bytes() {
                                                        let response =
                                                            ReqPairingResponse { message };
                                                        let _ = self
                                                            .swarm
                                                            .behaviour_mut()
                                                            .request_response
                                                            .send_response(channel, response);
                                                    }

                                                    let _ = self
                                                        .event_tx
                                                        .send(NetworkEvent::PairingComplete {
                                                            session_id: confirm.session_id,
                                                            peer_id: peer.to_string(),
                                                            peer_device_name: initiator_device_name,
                                                            shared_secret,
                                                        })
                                                        .await;
                                                }
                                            } else {
                                                let ack = super::protocol::PairingConfirm {
                                                    session_id: confirm.session_id.clone(),
                                                    success: false,
                                                    shared_secret: None,
                                                    error: confirm.error.clone(),
                                                    sender_device_name: self.device_name.clone(),
                                                };
                                                let ack_msg =
                                                    ProtocolMessage::Pairing(PairingMessage::Confirm(ack));
                                                if let Ok(message) = ack_msg.to_bytes() {
                                                    let response = ReqPairingResponse { message };
                                                    let _ = self
                                                        .swarm
                                                        .behaviour_mut()
                                                        .request_response
                                                        .send_response(channel, response);
                                                }

                                                let _ = self
                                                    .event_tx
                                                    .send(NetworkEvent::PairingFailed {
                                                        session_id: confirm.session_id,
                                                        error: confirm.error.unwrap_or_else(|| {
                                                            "Pairing cancelled".to_string()
                                                        }),
                                                    })
                                                    .await;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            request_response::Message::Response { response, .. } => {
                                if let Ok(ProtocolMessage::Pairing(pairing_msg)) =
                                    ProtocolMessage::from_bytes(&response.message)
                                {
                                    match pairing_msg {
                                        PairingMessage::Challenge(challenge) => {
                                            let _ = self
                                                .event_tx
                                                .send(NetworkEvent::PairingPinReady {
                                                    session_id: challenge.session_id,
                                                    pin: challenge.pin,
                                                    peer_device_name: challenge.device_name,
                                                    peer_public_key: challenge.public_key,
                                                })
                                                .await;
                                        }
                                        PairingMessage::Confirm(confirm) => {
                                            if confirm.success {
                                                if let Some(secret) = confirm.shared_secret {
                                                    // Get responder's device name from the ACK
                                                    let responder_device_name = confirm.sender_device_name.clone();

                                                    let _ = self
                                                        .event_tx
                                                        .send(NetworkEvent::PairingComplete {
                                                            session_id: confirm.session_id,
                                                            peer_id: peer.to_string(),
                                                            peer_device_name: responder_device_name,
                                                            shared_secret: secret,
                                                        })
                                                        .await;
                                                }
                                            } else {
                                                let _ = self
                                                    .event_tx
                                                    .send(NetworkEvent::PairingFailed {
                                                        session_id: confirm.session_id,
                                                        error: confirm.error.unwrap_or_else(|| {
                                                            "Unknown error".to_string()
                                                        }),
                                                    })
                                                    .await;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            SwarmEvent::Behaviour(super::behaviour::UniClipboardBehaviourEvent::Identify(
                identify::Event::Received { peer_id, info, .. },
            )) => {
                debug!("Identified peer {}: {}", peer_id, info.agent_version);

                // Parse device name from agent_version
                // Format: "uniclipboard/<version>/<device_name>"
                let device_name = if info.agent_version.starts_with("uniclipboard/") {
                    let parts: Vec<&str> = info.agent_version.splitn(3, '/').collect();
                    if parts.len() >= 3 {
                        Some(parts[2].to_string())
                    } else {
                        Some(info.agent_version.clone())
                    }
                } else {
                    Some(info.agent_version.clone())
                };

                if let Some(discovered) = self.discovered_peers.get_mut(&peer_id) {
                    let old_name = discovered.device_name.clone();
                    discovered.device_name = device_name;

                    if old_name != discovered.device_name {
                        let _ = self
                            .event_tx
                            .send(NetworkEvent::PeerDiscovered(discovered.clone()))
                            .await;
                    }
                }
            }

            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                debug!("Connection established with {}", peer_id);

                // Add peer to gossipsub mesh explicitly
                self.swarm
                    .behaviour_mut()
                    .gossipsub
                    .add_explicit_peer(&peer_id);

                let connected = ConnectedPeer {
                    peer_id: peer_id.to_string(),
                    device_name: self
                        .discovered_peers
                        .get(&peer_id)
                        .and_then(|p| p.device_name.clone())
                        .unwrap_or_else(|| "Unknown".to_string()),
                    connected_at: Utc::now(),
                };
                self.connected_peers.insert(peer_id, connected.clone());
                let _ = self
                    .event_tx
                    .send(NetworkEvent::PeerConnected(connected))
                    .await;
            }

            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                debug!("Connection closed with {}", peer_id);

                // Remove peer from gossipsub explicit peers
                self.swarm
                    .behaviour_mut()
                    .gossipsub
                    .remove_explicit_peer(&peer_id);

                if self.ready_peers.remove(&peer_id).is_some() {
                    let _ = self
                        .event_tx
                        .send(NetworkEvent::PeerNotReady {
                            peer_id: peer_id.to_string(),
                        })
                        .await;
                }

                self.connected_peers.remove(&peer_id);
                let _ = self
                    .event_tx
                    .send(NetworkEvent::PeerDisconnected(peer_id.to_string()))
                    .await;
            }

            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                warn!("Outgoing connection error to {:?}: {}", peer_id, error);
            }

            SwarmEvent::IncomingConnectionError { error, .. } => {
                warn!("Incoming connection error: {}", error);
            }

            SwarmEvent::Dialing { peer_id, .. } => {
                debug!("Dialing peer: {:?}", peer_id);
            }

            _ => {}
        }
    }

    async fn handle_command(&mut self, command: NetworkCommand) {
        match command {
            NetworkCommand::BroadcastClipboard { message } => {
                let protocol_msg = ProtocolMessage::Clipboard(message.clone());
                match self.swarm.behaviour_mut().publish_clipboard(&protocol_msg) {
                    Ok(_) => {
                        debug!("Broadcast clipboard message: {}", message.id);
                        let _ = self
                            .event_tx
                            .send(NetworkEvent::ClipboardSent {
                                id: message.id,
                                peer_count: self.connected_peers.len(),
                            })
                            .await;
                    }
                    Err(e) => {
                        warn!("Failed to broadcast clipboard: {}", e);
                    }
                }
            }

            NetworkCommand::SendPairingRequest { peer_id, message } => {
                if let Ok(peer) = peer_id.parse::<PeerId>() {
                    let request = ReqPairingRequest { message };
                    self.swarm
                        .behaviour_mut()
                        .request_response
                        .send_request(&peer, request);
                    debug!("Sent pairing request to {}", peer_id);
                }
            }

            NetworkCommand::SendPairingChallenge {
                peer_id,
                session_id,
                pin,
                device_name,
                public_key,
            } => {
                if let Ok(peer) = peer_id.parse::<PeerId>() {
                    if let Some(channel) = self.pending_responses.remove(&peer) {
                        let challenge = super::protocol::PairingChallenge {
                            session_id: session_id.clone(),
                            pin,
                            device_name,
                            public_key,
                        };
                        let protocol_msg =
                            ProtocolMessage::Pairing(PairingMessage::Challenge(challenge));
                        if let Ok(message) = protocol_msg.to_bytes() {
                            let response = ReqPairingResponse { message };
                            if self
                                .swarm
                                .behaviour_mut()
                                .request_response
                                .send_response(channel, response)
                                .is_ok()
                            {
                                debug!("Sent pairing challenge to {}", peer_id);
                            }
                        }
                    }
                }
            }

            NetworkCommand::RejectPairing {
                peer_id,
                session_id,
            } => {
                if let Ok(peer) = peer_id.parse::<PeerId>() {
                    if let Some(channel) = self.pending_responses.remove(&peer) {
                        let confirm = super::protocol::PairingConfirm {
                            session_id,
                            success: false,
                            shared_secret: None,
                            error: Some("Pairing rejected by user".to_string()),
                            device_name: None,
                        };
                        let protocol_msg =
                            ProtocolMessage::Pairing(PairingMessage::Confirm(confirm));
                        if let Ok(message) = protocol_msg.to_bytes() {
                            let response = ReqPairingResponse { message };
                            let _ = self
                                .swarm
                                .behaviour_mut()
                                .request_response
                                .send_response(channel, response);
                        }
                    }
                }
            }

            NetworkCommand::SendPairingConfirm {
                peer_id,
                session_id,
                success,
                shared_secret,
                device_name,
            } => {
                if let Ok(peer) = peer_id.parse::<PeerId>() {
                    let confirm = super::protocol::PairingConfirm {
                        session_id,
                        success,
                        shared_secret,
                        error: None,
                        device_name: Some(device_name),
                    };
                    let protocol_msg = ProtocolMessage::Pairing(PairingMessage::Confirm(confirm));
                    if let Ok(message) = protocol_msg.to_bytes() {
                        let request = ReqPairingRequest { message };
                        self.swarm
                            .behaviour_mut()
                            .request_response
                            .send_request(&peer, request);
                    }
                }
            }

            NetworkCommand::ReconnectPeers {
                paired_peer_addresses,
            } => {
                info!("Reconnecting to peers");

                let mesh_peers: std::collections::HashSet<PeerId> = self
                    .swarm
                    .behaviour()
                    .gossipsub
                    .all_mesh_peers()
                    .cloned()
                    .collect();

                for peer_id in &mesh_peers {
                    if !self.ready_peers.contains_key(peer_id) {
                        self.ready_peers.insert(*peer_id, Instant::now());
                        let _ = self
                            .event_tx
                            .send(NetworkEvent::PeerReady {
                                peer_id: peer_id.to_string(),
                            })
                            .await;
                    }
                }

                let mut dialed_peers: std::collections::HashSet<PeerId> =
                    std::collections::HashSet::new();

                for (peer_id, peer) in &self.discovered_peers {
                    if self.connected_peers.contains_key(peer_id) {
                        continue;
                    }
                    if let Some(addr_str) = peer.addresses.first() {
                        if let Ok(addr) = addr_str.parse::<Multiaddr>() {
                            if let Err(e) = self.swarm.dial(addr) {
                                warn!("Failed to reconnect to {}: {}", peer_id, e);
                            }
                            dialed_peers.insert(*peer_id);
                        }
                    }
                }

                for (peer_id_str, addresses) in &paired_peer_addresses {
                    if let Ok(peer_id) = peer_id_str.parse::<PeerId>() {
                        if self.connected_peers.contains_key(&peer_id)
                            || dialed_peers.contains(&peer_id)
                        {
                            continue;
                        }

                        for addr_str in addresses {
                            if let Ok(addr) = addr_str.parse::<Multiaddr>() {
                                if let Err(e) = self.swarm.dial(addr.clone()) {
                                    warn!("Failed to dial paired peer {} at {}: {}", peer_id, addr, e);
                                } else {
                                    dialed_peers.insert(peer_id);
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            NetworkCommand::RefreshPeer { peer_id } => {
                if let Ok(pid) = peer_id.parse::<PeerId>() {
                    if let Some(peer) = self.discovered_peers.get(&pid) {
                        let _ = self
                            .event_tx
                            .send(NetworkEvent::PeerDiscovered(peer.clone()))
                            .await;
                    }
                }
            }

            NetworkCommand::AnnounceDeviceName { device_name } => {
                self.device_name = device_name.clone();

                let local_peer_id = self.swarm.local_peer_id().to_string();
                let announce_msg = DeviceAnnounceMessage {
                    peer_id: local_peer_id,
                    device_name,
                    timestamp: Utc::now(),
                };
                let protocol_msg = ProtocolMessage::DeviceAnnounce(announce_msg);
                match self.swarm.behaviour_mut().publish_clipboard(&protocol_msg) {
                    Ok(_) => {
                        debug!("Broadcast device name announcement");
                    }
                    Err(e) => {
                        warn!("Failed to broadcast device name announcement: {}", e);
                    }
                }
            }

            NetworkCommand::StartListening
            | NetworkCommand::StopListening
            | NetworkCommand::GetPeers => {}
        }
    }
}
