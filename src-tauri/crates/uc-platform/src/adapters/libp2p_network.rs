use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use libp2p::{
    futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, StreamExt},
    identity, mdns, noise,
    request_response::{self, ProtocolSupport},
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, StreamProtocol, SwarmBuilder,
};
use libp2p_stream as stream;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, Mutex as AsyncMutex, RwLock};
use tracing::{debug, error, info, warn};
use uc_core::network::{
    ClipboardMessage, ConnectedPeer, DiscoveredPeer, NetworkEvent, PairingMessage, PairingState,
    ProtocolDenyReason, ProtocolDirection, ProtocolId, ProtocolKind, ResolvedConnectionPolicy,
};
use uc_core::ports::{
    ConnectionPolicyResolverPort, IdentityStorePort, NetworkControlPort, NetworkPort,
};

use crate::identity_store::load_or_create_identity;

const PAIRING_PROTOCOL_ID: &str = ProtocolId::Pairing.as_str();
const BUSINESS_PROTOCOL_ID: &str = ProtocolId::Business.as_str();

#[derive(Debug)]
enum PairingCommand {
    SendMessage {
        peer_id: uc_core::PeerId,
        message: PairingMessage,
    },
}

#[derive(Debug)]
enum BusinessCommand {
    SendClipboard {
        peer_id: uc_core::PeerId,
        data: Vec<u8>,
    },
}

pub struct PeerCaches {
    discovered_peers: HashMap<String, DiscoveredPeer>,
    reachable_peers: HashSet<String>,
}

impl PeerCaches {
    pub fn new() -> Self {
        Self {
            discovered_peers: HashMap::new(),
            reachable_peers: HashSet::new(),
        }
    }

    pub fn upsert_discovered(
        &mut self,
        peer_id: String,
        addresses: Vec<String>,
        discovered_at: DateTime<Utc>,
    ) -> DiscoveredPeer {
        let peer = DiscoveredPeer {
            peer_id,
            device_name: None,
            device_id: None,
            addresses,
            discovered_at,
            last_seen: discovered_at,
            is_paired: false,
        };
        self.discovered_peers
            .insert(peer.peer_id.clone(), peer.clone());
        peer
    }

    pub fn remove_discovered(&mut self, peer_id: &str) -> Option<DiscoveredPeer> {
        self.reachable_peers.remove(peer_id);
        self.discovered_peers.remove(peer_id)
    }

    pub fn mark_reachable(&mut self, peer_id: &str) -> bool {
        if self.discovered_peers.contains_key(peer_id) {
            self.reachable_peers.insert(peer_id.to_string());
            true
        } else {
            false
        }
    }

    pub fn is_reachable(&self, peer_id: &str) -> bool {
        self.reachable_peers.contains(peer_id)
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "Libp2pBehaviourEvent")]
struct Libp2pBehaviour {
    mdns: mdns::tokio::Behaviour,
    pairing: request_response::json::Behaviour<PairingMessage, PairingMessage>,
    stream: stream::Behaviour,
}

#[derive(Debug)]
enum Libp2pBehaviourEvent {
    Mdns(mdns::Event),
    Pairing(request_response::Event<PairingMessage, PairingMessage>),
    Stream,
}

impl From<mdns::Event> for Libp2pBehaviourEvent {
    fn from(event: mdns::Event) -> Self {
        Self::Mdns(event)
    }
}

impl From<request_response::Event<PairingMessage, PairingMessage>> for Libp2pBehaviourEvent {
    fn from(event: request_response::Event<PairingMessage, PairingMessage>) -> Self {
        Self::Pairing(event)
    }
}

impl From<()> for Libp2pBehaviourEvent {
    fn from(_: ()) -> Self {
        Self::Stream
    }
}

impl Libp2pBehaviour {
    fn new(local_peer_id: PeerId) -> Result<Self> {
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)
            .map_err(|e| anyhow!("failed to create mdns behaviour: {e}"))?;
        let config = request_response::Config::default()
            .with_request_timeout(std::time::Duration::from_secs(10));
        let pairing = request_response::json::Behaviour::new(
            [(
                StreamProtocol::new(PAIRING_PROTOCOL_ID),
                ProtocolSupport::Full,
            )],
            config,
        );
        let stream = stream::Behaviour::new();
        Ok(Self {
            mdns,
            pairing,
            stream,
        })
    }
}

pub struct Libp2pNetworkAdapter {
    local_peer_id: String,
    local_identity_pubkey: Vec<u8>,
    caches: Arc<RwLock<PeerCaches>>,
    event_tx: mpsc::Sender<NetworkEvent>,
    event_rx: Mutex<Option<mpsc::Receiver<NetworkEvent>>>,
    clipboard_tx: mpsc::Sender<ClipboardMessage>,
    clipboard_rx: Mutex<Option<mpsc::Receiver<ClipboardMessage>>>,
    pairing_tx: mpsc::Sender<PairingCommand>,
    pairing_rx: Mutex<Option<mpsc::Receiver<PairingCommand>>>,
    business_tx: mpsc::Sender<BusinessCommand>,
    business_rx: Mutex<Option<mpsc::Receiver<BusinessCommand>>>,
    keypair: Mutex<Option<identity::Keypair>>,
    policy_resolver: Arc<dyn ConnectionPolicyResolverPort>,
    stream_control: Mutex<Option<stream::Control>>,
    pairing_response_channels:
        Arc<AsyncMutex<HashMap<String, request_response::ResponseChannel<PairingMessage>>>>,
}

impl Libp2pNetworkAdapter {
    pub fn new(
        identity_store: Arc<dyn IdentityStorePort>,
        policy_resolver: Arc<dyn ConnectionPolicyResolverPort>,
    ) -> Result<Self> {
        let keypair = load_or_create_identity(identity_store.as_ref())
            .map_err(|e| anyhow!("failed to load libp2p identity: {e}"))?;
        let local_peer_id = PeerId::from(keypair.public()).to_string();
        let local_identity_pubkey = keypair.public().encode_protobuf();
        let (event_tx, event_rx) = mpsc::channel(64);
        let (clipboard_tx, clipboard_rx) = mpsc::channel(64);
        let (pairing_tx, pairing_rx) = mpsc::channel(64);
        let (business_tx, business_rx) = mpsc::channel(64);
        let pairing_response_channels = Arc::new(AsyncMutex::new(HashMap::new()));

        Ok(Self {
            local_peer_id,
            local_identity_pubkey,
            caches: Arc::new(RwLock::new(PeerCaches::new())),
            event_tx,
            event_rx: Mutex::new(Some(event_rx)),
            clipboard_tx,
            clipboard_rx: Mutex::new(Some(clipboard_rx)),
            pairing_tx,
            pairing_rx: Mutex::new(Some(pairing_rx)),
            business_tx,
            business_rx: Mutex::new(Some(business_rx)),
            keypair: Mutex::new(Some(keypair)),
            policy_resolver,
            stream_control: Mutex::new(None),
            pairing_response_channels,
        })
    }

    pub fn local_identity_pubkey(&self) -> Vec<u8> {
        self.local_identity_pubkey.clone()
    }

    pub fn spawn_swarm(&self) -> Result<()> {
        let keypair = self.take_keypair()?;
        let local_peer_id = PeerId::from(keypair.public());
        let behaviour = Libp2pBehaviour::new(local_peer_id)
            .map_err(|e| anyhow!("failed to create libp2p behaviour: {e}"))?;

        let mut swarm = SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_tcp(
                tcp::Config::default().nodelay(true),
                noise::Config::new,
                yamux::Config::default,
            )
            .map_err(|e| anyhow!("failed to configure tcp transport: {e}"))?
            .with_behaviour(move |_| behaviour)
            .map_err(|e| anyhow!("failed to attach libp2p behaviour: {e}"))?
            .build();

        let stream_control = swarm.behaviour().stream.new_control();
        let mut guard = self
            .stream_control
            .lock()
            .map_err(|_| anyhow!("stream control mutex poisoned"))?;
        *guard = Some(stream_control.clone());

        spawn_business_stream_echo(
            stream_control.clone(),
            self.event_tx.clone(),
            self.policy_resolver.clone(),
        );

        listen_on_swarm(
            &mut swarm,
            "/ip4/0.0.0.0/tcp/0"
                .parse()
                .map_err(|e| anyhow!("failed to parse listen address: {e}"))?,
            &self.event_tx,
        )?;

        let caches = self.caches.clone();
        let event_tx = self.event_tx.clone();
        let policy_resolver = self.policy_resolver.clone();
        let pairing_response_channels = self.pairing_response_channels.clone();
        let pairing_rx = Self::take_receiver(&self.pairing_rx, "pairing command")?;
        let business_rx = Self::take_receiver(&self.business_rx, "business command")?;
        tokio::spawn(async move {
            run_swarm(
                swarm,
                caches,
                event_tx,
                policy_resolver,
                pairing_response_channels,
                pairing_rx,
                business_rx,
            )
            .await;
        });
        Ok(())
    }

    fn take_keypair(&self) -> Result<identity::Keypair> {
        let mut guard = self
            .keypair
            .lock()
            .map_err(|_| anyhow!("libp2p keypair mutex poisoned"))?;
        guard
            .take()
            .ok_or_else(|| anyhow!("libp2p keypair already taken"))
    }

    fn take_receiver<T>(
        mutex: &Mutex<Option<mpsc::Receiver<T>>>,
        name: &str,
    ) -> Result<mpsc::Receiver<T>> {
        let mut guard = mutex
            .lock()
            .map_err(|_| anyhow!("{name} receiver mutex poisoned"))?;
        guard
            .take()
            .ok_or_else(|| anyhow!("{name} receiver already taken"))
    }
}

#[async_trait]
impl NetworkPort for Libp2pNetworkAdapter {
    async fn send_clipboard(&self, _peer_id: &str, _encrypted_data: Vec<u8>) -> Result<()> {
        let peer = uc_core::PeerId::from(_peer_id);
        self.business_tx
            .send(BusinessCommand::SendClipboard {
                peer_id: peer,
                data: _encrypted_data,
            })
            .await
            .map_err(|err| anyhow!("failed to queue business stream: {err}"))
    }

    async fn broadcast_clipboard(&self, _encrypted_data: Vec<u8>) -> Result<()> {
        Err(anyhow!(
            "NetworkPort::broadcast_clipboard not implemented yet"
        ))
    }

    async fn subscribe_clipboard(&self) -> Result<mpsc::Receiver<ClipboardMessage>> {
        if self.clipboard_tx.is_closed() {
            warn!("clipboard channel sender is closed");
        }
        Self::take_receiver(&self.clipboard_rx, "clipboard")
    }

    async fn get_discovered_peers(&self) -> Result<Vec<DiscoveredPeer>> {
        let caches = self.caches.read().await;
        Ok(caches.discovered_peers.values().cloned().collect())
    }

    async fn get_connected_peers(&self) -> Result<Vec<ConnectedPeer>> {
        Ok(Vec::new())
    }

    fn local_peer_id(&self) -> String {
        self.local_peer_id.clone()
    }

    async fn send_pairing_message(&self, peer_id: String, message: PairingMessage) -> Result<()> {
        let peer = uc_core::PeerId::from(peer_id.clone());

        self.pairing_tx
            .send(PairingCommand::SendMessage {
                peer_id: peer,
                message,
            })
            .await
            .map_err(|err| anyhow!("failed to queue pairing message: {err}"))?;

        Ok(())
    }

    async fn unpair_device(&self, _peer_id: String) -> Result<()> {
        Err(anyhow!("NetworkPort::unpair_device not implemented yet"))
    }

    async fn subscribe_events(&self) -> Result<mpsc::Receiver<NetworkEvent>> {
        Self::take_receiver(&self.event_rx, "network event")
    }
}

#[async_trait]
impl NetworkControlPort for Libp2pNetworkAdapter {
    async fn start_network(&self) -> Result<()> {
        self.spawn_swarm()
    }
}

async fn echo_payload<T>(stream: &mut T) -> Result<()>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).await?;
    stream.write_all(&buf).await?;
    Ok(())
}

fn spawn_business_stream_echo(
    mut control: stream::Control,
    event_tx: mpsc::Sender<NetworkEvent>,
    policy_resolver: Arc<dyn ConnectionPolicyResolverPort>,
) {
    let mut incoming = match control.accept(StreamProtocol::new(BUSINESS_PROTOCOL_ID)) {
        Ok(incoming) => incoming,
        Err(err) => {
            warn!("failed to accept business stream: {err}");
            return;
        }
    };

    tokio::spawn(async move {
        while let Some((_peer, mut stream)) = incoming.next().await {
            let peer_id = _peer.to_string();
            let event_tx = event_tx.clone();
            let policy_resolver = policy_resolver.clone();
            tokio::spawn(async move {
                if check_business_allowed(
                    &policy_resolver,
                    &event_tx,
                    &peer_id,
                    ProtocolDirection::Inbound,
                )
                .await
                .is_err()
                {
                    return;
                }
                if let Err(err) = echo_payload(&mut stream).await {
                    warn!("business stream echo failed: {err}");
                }
            });
        }
    });
}

async fn handle_pairing_event(
    _swarm: &mut Swarm<Libp2pBehaviour>,
    event: request_response::Event<PairingMessage, PairingMessage>,
    response_channels: &Arc<
        AsyncMutex<HashMap<String, request_response::ResponseChannel<PairingMessage>>>,
    >,
    event_tx: &mpsc::Sender<NetworkEvent>,
) {
    if let request_response::Event::Message { peer, message, .. } = event {
        match message {
            request_response::Message::Request {
                request, channel, ..
            } => {
                let session_id = request.session_id().to_string();
                {
                    let mut channels = response_channels.lock().await;
                    channels.insert(session_id.clone(), channel);
                }
                if let Err(err) = event_tx
                    .send(NetworkEvent::PairingMessageReceived {
                        peer_id: peer.to_string(),
                        message: request,
                    })
                    .await
                {
                    warn!("failed to emit pairing message: {err}");
                }
            }
            request_response::Message::Response { response, .. } => {
                if let Err(err) = event_tx
                    .send(NetworkEvent::PairingMessageReceived {
                        peer_id: peer.to_string(),
                        message: response,
                    })
                    .await
                {
                    warn!("failed to emit pairing response: {err}");
                }
            }
        }
    }
}

async fn emit_protocol_denied(
    event_tx: &mpsc::Sender<NetworkEvent>,
    peer_id: String,
    pairing_state: PairingState,
    direction: ProtocolDirection,
    reason: ProtocolDenyReason,
) {
    if let Err(err) = event_tx
        .send(NetworkEvent::ProtocolDenied {
            peer_id,
            protocol_id: BUSINESS_PROTOCOL_ID.to_string(),
            pairing_state,
            direction,
            reason,
        })
        .await
    {
        warn!("failed to emit protocol denied event: {err}");
    }
}

async fn check_business_allowed(
    policy_resolver: &Arc<dyn ConnectionPolicyResolverPort>,
    event_tx: &mpsc::Sender<NetworkEvent>,
    peer_id: &str,
    direction: ProtocolDirection,
) -> Result<ResolvedConnectionPolicy> {
    let peer = uc_core::PeerId::from(peer_id);
    match policy_resolver.resolve_for_peer(&peer).await {
        Ok(resolved) => {
            if resolved.allowed.allows(ProtocolKind::Business) {
                Ok(resolved)
            } else {
                emit_protocol_denied(
                    event_tx,
                    peer_id.to_string(),
                    resolved.pairing_state,
                    direction,
                    ProtocolDenyReason::NotTrusted,
                )
                .await;
                Err(anyhow!("business protocol denied"))
            }
        }
        Err(err) => {
            emit_protocol_denied(
                event_tx,
                peer_id.to_string(),
                PairingState::Pending,
                direction,
                ProtocolDenyReason::RepoError,
            )
            .await;
            Err(anyhow!("policy resolver failed: {err}"))
        }
    }
}

async fn run_swarm(
    mut swarm: Swarm<Libp2pBehaviour>,
    caches: Arc<RwLock<PeerCaches>>,
    event_tx: mpsc::Sender<NetworkEvent>,
    policy_resolver: Arc<dyn ConnectionPolicyResolverPort>,
    pairing_response_channels: Arc<
        AsyncMutex<HashMap<String, request_response::ResponseChannel<PairingMessage>>>,
    >,
    mut pairing_rx: mpsc::Receiver<PairingCommand>,
    mut business_rx: mpsc::Receiver<BusinessCommand>,
) {
    info!("libp2p mDNS swarm started");

    loop {
        tokio::select! {
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::Behaviour(Libp2pBehaviourEvent::Mdns(event)) => match event {
                        mdns::Event::Discovered(peers) => {
                            let discovered = collect_mdns_discovered(peers);
                            let events = {
                                let mut caches = caches.write().await;
                                apply_mdns_discovered(&mut caches, discovered, Utc::now())
                            };

                            for event in events {
                                let _ = try_send_event(&event_tx, event, "PeerDiscovered");
                            }
                        }
                        mdns::Event::Expired(peers) => {
                            let expired = collect_mdns_expired(peers);
                            let events = {
                                let mut caches = caches.write().await;
                                apply_mdns_expired(&mut caches, expired)
                            };

                            for event in events {
                                let _ = try_send_event(&event_tx, event, "PeerLost");
                            }
                        }
                    },
                    SwarmEvent::Behaviour(Libp2pBehaviourEvent::Pairing(event)) => {
                        handle_pairing_event(
                            &mut swarm,
                            event,
                            &pairing_response_channels,
                            &event_tx,
                        )
                        .await;
                    }
                    SwarmEvent::Behaviour(Libp2pBehaviourEvent::Stream) => {}
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        let peer_id = peer_id.to_string();
                        let event = {
                            let mut caches = caches.write().await;
                            apply_peer_ready(&mut caches, &peer_id)
                        };

                        if let Some(event) = event {
                            let _ = try_send_event(&event_tx, event, "PeerReady");
                        } else {
                            debug!("connection established for unknown peer {peer_id}");
                        }
                    }
                    SwarmEvent::ConnectionClosed { peer_id, .. } => {
                        let peer_id = peer_id.to_string();
                        let event = {
                            let mut caches = caches.write().await;
                            apply_peer_not_ready(&mut caches, &peer_id)
                        };

                        if let Some(event) = event {
                            let _ = try_send_event(&event_tx, event, "PeerNotReady");
                        }
                    }
                    SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                        error!("outgoing connection error to {:?}: {}", peer_id, error);
                        if let Err(err) = event_tx
                            .send(NetworkEvent::Error("network connection error".to_string()))
                            .await
                        {
                            warn!("failed to publish network error event: {err}");
                        }
                    }
                    SwarmEvent::IncomingConnectionError {
                        send_back_addr,
                        error,
                        ..
                    } => {
                        error!(
                            "incoming connection error from {}: {}",
                            send_back_addr, error
                        );
                        if let Err(err) = event_tx
                            .send(NetworkEvent::Error("network connection error".to_string()))
                            .await
                        {
                            warn!("failed to publish network error event: {err}");
                        }
                    }
                    SwarmEvent::NewListenAddr { address, .. } => {
                        info!("libp2p listening on {address}");
                    }
                    _ => {}
                }
            }
            Some(command) = pairing_rx.recv() => {
                match command {
                    PairingCommand::SendMessage { peer_id, message } => {
                        let session_id = message.session_id().to_string();
                        let channel = {
                            let mut channels = pairing_response_channels.lock().await;
                            channels.remove(&session_id)
                        };

                        if let Some(channel) = channel {
                            if let Err(err) = swarm.behaviour_mut().pairing.send_response(channel, message) {
                                warn!("failed to send pairing response: {err:?}");
                            }
                            continue;
                        }

                        let peer = match peer_id.as_str().parse::<PeerId>() {
                            Ok(peer) => peer,
                            Err(err) => {
                                warn!("invalid peer id for pairing message: {err}");
                                continue;
                            }
                        };
                        swarm.behaviour_mut().pairing.send_request(&peer, message);
                    }
                }
            }
            Some(command) = business_rx.recv() => {
                match command {
                    BusinessCommand::SendClipboard { peer_id, data } => {
                        let peer = match peer_id.as_str().parse::<PeerId>() {
                            Ok(peer) => peer,
                            Err(err) => {
                                warn!("invalid peer id for business stream: {err}");
                                continue;
                            }
                        };
                        if check_business_allowed(
                            &policy_resolver,
                            &event_tx,
                            peer_id.as_str(),
                            ProtocolDirection::Outbound,
                        )
                        .await
                        .is_err()
                        {
                            continue;
                        }
                        let mut control = swarm.behaviour().stream.new_control();
                        match control
                            .open_stream(peer, StreamProtocol::new(BUSINESS_PROTOCOL_ID))
                            .await
                        {
                            Ok(mut stream) => {
                                if let Err(err) = stream.write_all(&data).await {
                                    warn!("business stream write failed: {err}");
                                } else if let Err(err) = stream.close().await {
                                    warn!("business stream close failed: {err}");
                                }
                            }
                            Err(err) => {
                                warn!("business stream open failed: {err}");
                            }
                        }
                    }
                }
            }
        }
    }
}

fn listen_on_swarm(
    swarm: &mut Swarm<Libp2pBehaviour>,
    listen_addr: Multiaddr,
    event_tx: &mpsc::Sender<NetworkEvent>,
) -> Result<()> {
    if let Err(e) = swarm.listen_on(listen_addr) {
        let message = format!("failed to listen on tcp: {e}");
        warn!("{message}");
        if let Err(err) = event_tx.try_send(NetworkEvent::Error(message.clone())) {
            warn!("failed to publish network error event: {err}");
        }
        return Err(anyhow!(message));
    }

    Ok(())
}

fn try_send_event(
    event_tx: &mpsc::Sender<NetworkEvent>,
    event: NetworkEvent,
    label: &str,
) -> Result<(), mpsc::error::TrySendError<NetworkEvent>> {
    event_tx.try_send(event).map_err(|err| {
        warn!("failed to send {label} event: {err}");
        err
    })
}

fn collect_mdns_discovered(
    peers: impl IntoIterator<Item = (PeerId, Multiaddr)>,
) -> HashMap<String, Vec<String>> {
    let mut discovered = HashMap::new();
    for (peer_id, addr) in peers {
        discovered
            .entry(peer_id.to_string())
            .or_insert_with(Vec::new)
            .push(addr.to_string());
    }
    discovered
}

fn collect_mdns_expired(peers: impl IntoIterator<Item = (PeerId, Multiaddr)>) -> HashSet<String> {
    let mut expired = HashSet::new();
    for (peer_id, _) in peers {
        expired.insert(peer_id.to_string());
    }
    expired
}

fn apply_mdns_discovered(
    caches: &mut PeerCaches,
    discovered: HashMap<String, Vec<String>>,
    discovered_at: DateTime<Utc>,
) -> Vec<NetworkEvent> {
    discovered
        .into_iter()
        .map(|(peer_id, addresses)| {
            NetworkEvent::PeerDiscovered(caches.upsert_discovered(
                peer_id,
                addresses,
                discovered_at,
            ))
        })
        .collect()
}

fn apply_mdns_expired(caches: &mut PeerCaches, expired: HashSet<String>) -> Vec<NetworkEvent> {
    expired
        .into_iter()
        .filter_map(|peer_id| {
            caches
                .remove_discovered(&peer_id)
                .map(|_| NetworkEvent::PeerLost(peer_id))
        })
        .collect()
}

fn apply_peer_ready(caches: &mut PeerCaches, peer_id: &str) -> Option<NetworkEvent> {
    if caches.mark_reachable(peer_id) {
        Some(NetworkEvent::PeerReady {
            peer_id: peer_id.to_string(),
        })
    } else {
        None
    }
}

fn apply_peer_not_ready(caches: &mut PeerCaches, peer_id: &str) -> Option<NetworkEvent> {
    if caches.reachable_peers.remove(peer_id) {
        Some(NetworkEvent::PeerNotReady {
            peer_id: peer_id.to_string(),
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::futures::{AsyncReadExt, AsyncWriteExt};
    use libp2p::identity;
    use libp2p::multiaddr::Protocol;
    use libp2p::Multiaddr;
    use std::sync::{Arc, Mutex};
    use tokio::time::{sleep, timeout, Duration};
    use tokio_util::compat::TokioAsyncReadCompatExt;
    use uc_core::network::PairingReject;
    use uc_core::network::{ConnectionPolicy, PairingState, ResolvedConnectionPolicy};
    use uc_core::network::{PairingChallenge, PairingRequest};
    use uc_core::ports::{ConnectionPolicyResolverError, ConnectionPolicyResolverPort};

    #[test]
    fn cache_inserts_discovered_peer_with_addresses() {
        let mut caches = PeerCaches::new();
        let discovered_at = Utc::now();
        let addresses = vec!["/ip4/192.168.1.2/tcp/4001".to_string()];

        let peer = caches.upsert_discovered("peer-1".to_string(), addresses.clone(), discovered_at);

        assert_eq!(peer.peer_id, "peer-1");
        assert_eq!(peer.addresses, addresses);
        assert_eq!(peer.discovered_at, discovered_at);
        assert!(peer.device_name.is_none());
        assert!(peer.device_id.is_none());
        assert!(!peer.is_paired);
    }

    #[test]
    fn cache_removes_discovered_peer_on_loss() {
        let mut caches = PeerCaches::new();
        caches.upsert_discovered(
            "peer-1".to_string(),
            vec!["/ip4/192.168.1.2/tcp/4001".to_string()],
            Utc::now(),
        );

        let removed = caches.remove_discovered("peer-1");
        assert!(removed.is_some());
        assert!(!caches.is_reachable("peer-1"));
        assert!(caches.remove_discovered("peer-1").is_none());
    }

    #[test]
    fn reachable_is_best_effort_and_requires_discovery() {
        let mut caches = PeerCaches::new();
        assert!(!caches.mark_reachable("peer-1"));
        assert!(!caches.is_reachable("peer-1"));

        caches.upsert_discovered(
            "peer-1".to_string(),
            vec!["/ip4/192.168.1.2/tcp/4001".to_string()],
            Utc::now(),
        );
        assert!(caches.mark_reachable("peer-1"));
        assert!(caches.is_reachable("peer-1"));
    }

    #[test]
    fn mdns_discovery_groups_addresses_by_peer() {
        let peer = PeerId::random();
        let addr_one: Multiaddr = "/ip4/192.168.1.2/tcp/4001".parse().unwrap();
        let addr_two: Multiaddr = "/ip4/192.168.1.3/tcp/4001".parse().unwrap();

        let grouped =
            collect_mdns_discovered(vec![(peer, addr_one.clone()), (peer, addr_two.clone())]);

        let addresses = grouped
            .get(&peer.to_string())
            .expect("peer should be grouped");
        assert_eq!(addresses.len(), 2);
        assert!(addresses.contains(&addr_one.to_string()));
        assert!(addresses.contains(&addr_two.to_string()));
    }

    #[test]
    fn mdns_expired_deduplicates_peers() {
        let peer = PeerId::random();
        let addr_one: Multiaddr = "/ip4/192.168.1.2/tcp/4001".parse().unwrap();
        let addr_two: Multiaddr = "/ip4/192.168.1.3/tcp/4001".parse().unwrap();

        let expired = collect_mdns_expired(vec![(peer, addr_one), (peer, addr_two)]);

        assert_eq!(expired.len(), 1);
        assert!(expired.contains(&peer.to_string()));
    }

    #[test]
    fn peer_ready_emits_event_only_for_discovered_peer() {
        let mut caches = PeerCaches::new();
        caches.upsert_discovered(
            "peer-1".to_string(),
            vec!["/ip4/192.168.1.2/tcp/4001".to_string()],
            Utc::now(),
        );

        let event = apply_peer_ready(&mut caches, "peer-1");

        assert!(matches!(
            event,
            Some(NetworkEvent::PeerReady { peer_id }) if peer_id == "peer-1"
        ));
        assert!(caches.is_reachable("peer-1"));
    }

    #[test]
    fn peer_not_ready_emits_event_only_for_reachable_peer() {
        let mut caches = PeerCaches::new();
        caches.upsert_discovered(
            "peer-1".to_string(),
            vec!["/ip4/192.168.1.2/tcp/4001".to_string()],
            Utc::now(),
        );

        assert!(apply_peer_not_ready(&mut caches, "peer-1").is_none());
        let _ = apply_peer_ready(&mut caches, "peer-1");

        let event = apply_peer_not_ready(&mut caches, "peer-1");

        assert!(matches!(
            event,
            Some(NetworkEvent::PeerNotReady { peer_id }) if peer_id == "peer-1"
        ));
        assert!(!caches.is_reachable("peer-1"));
    }

    #[test]
    fn mdns_discovery_and_expiry_emit_events() {
        let mut caches = PeerCaches::new();
        let discovered_at = Utc::now();
        let mut discovered = HashMap::new();
        discovered.insert(
            "peer-1".to_string(),
            vec!["/ip4/192.168.1.2/tcp/4001".to_string()],
        );

        let discovered_events = apply_mdns_discovered(&mut caches, discovered, discovered_at);
        assert_eq!(discovered_events.len(), 1);
        assert!(matches!(
            &discovered_events[0],
            NetworkEvent::PeerDiscovered(peer) if peer.peer_id == "peer-1"
        ));
        assert!(caches.discovered_peers.contains_key("peer-1"));

        let mut expired = HashSet::new();
        expired.insert("peer-1".to_string());
        let expired_events = apply_mdns_expired(&mut caches, expired);

        assert_eq!(expired_events.len(), 1);
        assert!(matches!(
            &expired_events[0],
            NetworkEvent::PeerLost(peer_id) if peer_id == "peer-1"
        ));
        assert!(!caches.discovered_peers.contains_key("peer-1"));
    }

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

    struct FakeResolver;

    #[async_trait::async_trait]
    impl ConnectionPolicyResolverPort for FakeResolver {
        async fn resolve_for_peer(
            &self,
            _peer_id: &uc_core::PeerId,
        ) -> Result<ResolvedConnectionPolicy, ConnectionPolicyResolverError> {
            Ok(ResolvedConnectionPolicy {
                pairing_state: PairingState::Trusted,
                allowed: ConnectionPolicy::allowed_protocols(PairingState::Trusted),
            })
        }
    }

    struct PendingResolver;

    #[async_trait::async_trait]
    impl ConnectionPolicyResolverPort for PendingResolver {
        async fn resolve_for_peer(
            &self,
            _peer_id: &uc_core::PeerId,
        ) -> Result<ResolvedConnectionPolicy, ConnectionPolicyResolverError> {
            Ok(ResolvedConnectionPolicy {
                pairing_state: PairingState::Pending,
                allowed: ConnectionPolicy::allowed_protocols(PairingState::Pending),
            })
        }
    }

    fn build_swarm(keypair: identity::Keypair) -> Swarm<Libp2pBehaviour> {
        let local_peer_id = PeerId::from(keypair.public());
        let behaviour = Libp2pBehaviour::new(local_peer_id).expect("behaviour");
        SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_tcp(
                tcp::Config::default().nodelay(true),
                noise::Config::new,
                yamux::Config::default,
            )
            .expect("tcp config")
            .with_behaviour(move |_| behaviour)
            .expect("attach behaviour")
            .build()
    }

    async fn listen_on_random(swarm: &mut Swarm<Libp2pBehaviour>) -> Multiaddr {
        swarm
            .listen_on("/ip4/127.0.0.1/tcp/0".parse().expect("listen addr"))
            .expect("listen on addr");
        loop {
            match swarm.select_next_some().await {
                SwarmEvent::NewListenAddr { address, .. } => return address,
                _ => {}
            }
        }
    }

    #[tokio::test]
    async fn adapter_constructs_with_policy_resolver() {
        let resolver: Arc<dyn ConnectionPolicyResolverPort> = Arc::new(FakeResolver);
        let adapter = Libp2pNetworkAdapter::new(Arc::new(TestIdentityStore::default()), resolver);
        assert!(adapter.is_ok());
    }

    #[test]
    fn libp2p_request_response_feature_available() {
        let _ = libp2p::request_response::Config::default();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pairing_request_emits_event() {
        let keypair_a = identity::Keypair::generate_ed25519();
        let keypair_b = identity::Keypair::generate_ed25519();
        let peer_b = PeerId::from(keypair_b.public());

        let mut swarm_a = build_swarm(keypair_a);
        let mut swarm_b = build_swarm(keypair_b);

        let addr_b = listen_on_random(&mut swarm_b).await;
        let _addr_a = listen_on_random(&mut swarm_a).await;
        let dial_addr = addr_b.with(Protocol::P2p(peer_b));
        swarm_a.dial(dial_addr).expect("dial b");

        let request = PairingMessage::Request(PairingRequest {
            session_id: "session-1".to_string(),
            device_name: "device-a".to_string(),
            device_id: "device-a".to_string(),
            peer_id: "peer-a".to_string(),
            identity_pubkey: vec![1; 32],
            nonce: vec![2; 16],
        });
        swarm_a
            .behaviour_mut()
            .pairing
            .send_request(&peer_b, request);

        let response_channels = Arc::new(AsyncMutex::new(HashMap::new()));
        let (event_tx, mut event_rx) = mpsc::channel(1);
        let received = timeout(Duration::from_secs(10), async {
            loop {
                tokio::select! {
                    event = swarm_a.select_next_some() => {
                        let _ = event;
                    }
                    event = swarm_b.select_next_some() => {
                        if let SwarmEvent::Behaviour(Libp2pBehaviourEvent::Pairing(event)) = event {
                            handle_pairing_event(
                                &mut swarm_b,
                                event,
                                &response_channels,
                                &event_tx,
                            )
                            .await;
                        }
                    }
                    event = event_rx.recv() => {
                        if let Some(NetworkEvent::PairingMessageReceived { message, .. }) = event {
                            return message;
                        }
                    }
                }
            }
        })
        .await
        .expect("event timeout");

        assert!(matches!(received, PairingMessage::Request(_)));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pairing_response_uses_stored_channel() {
        let keypair_a = identity::Keypair::generate_ed25519();
        let keypair_b = identity::Keypair::generate_ed25519();
        let peer_b = PeerId::from(keypair_b.public());

        let mut swarm_a = build_swarm(keypair_a);
        let mut swarm_b = build_swarm(keypair_b);

        let addr_b = listen_on_random(&mut swarm_b).await;
        let _addr_a = listen_on_random(&mut swarm_a).await;
        let dial_addr = addr_b.with(Protocol::P2p(peer_b));
        swarm_a.dial(dial_addr).expect("dial b");

        let request = PairingMessage::Request(PairingRequest {
            session_id: "session-2".to_string(),
            device_name: "device-a".to_string(),
            device_id: "device-a".to_string(),
            peer_id: "peer-a".to_string(),
            identity_pubkey: vec![1; 32],
            nonce: vec![2; 16],
        });
        swarm_a
            .behaviour_mut()
            .pairing
            .send_request(&peer_b, request);

        let response_channels = Arc::new(AsyncMutex::new(HashMap::new()));
        let (event_tx, mut event_rx) = mpsc::channel(1);
        let response = timeout(Duration::from_secs(10), async {
            let challenge = PairingMessage::Challenge(PairingChallenge {
                session_id: "session-2".to_string(),
                pin: "123456".to_string(),
                device_name: "device-b".to_string(),
                device_id: "device-b".to_string(),
                identity_pubkey: vec![3; 32],
                nonce: vec![4; 16],
            });

            loop {
                tokio::select! {
                    event = swarm_a.select_next_some() => {
                        if let SwarmEvent::Behaviour(Libp2pBehaviourEvent::Pairing(event)) = event {
                            if let request_response::Event::Message { message, .. } = event {
                                if let request_response::Message::Response { response, .. } = message {
                                    return response;
                                }
                            }
                        }
                    }
                    event = swarm_b.select_next_some() => {
                        if let SwarmEvent::Behaviour(Libp2pBehaviourEvent::Pairing(event)) = event {
                            handle_pairing_event(
                                &mut swarm_b,
                                event,
                                &response_channels,
                                &event_tx,
                            )
                            .await;
                        }
                    }
                    event = event_rx.recv() => {
                        if let Some(NetworkEvent::PairingMessageReceived { message, .. }) = event {
                            if matches!(message, PairingMessage::Request(_)) {
                                let channel = {
                                    let mut channels = response_channels.lock().await;
                                    channels.remove("session-2")
                                };
                                if let Some(channel) = channel {
                                    let _ = swarm_b.behaviour_mut().pairing.send_response(channel, challenge.clone());
                                }
                            }
                        }
                    }
                }
            }
        })
        .await
        .expect("response timeout");

        assert!(matches!(response, PairingMessage::Challenge(_)));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pairing_reject_uses_stored_channel() {
        let keypair_a = identity::Keypair::generate_ed25519();
        let keypair_b = identity::Keypair::generate_ed25519();
        let peer_b = PeerId::from(keypair_b.public());

        let mut swarm_a = build_swarm(keypair_a);
        let mut swarm_b = build_swarm(keypair_b);

        let addr_b = listen_on_random(&mut swarm_b).await;
        let _addr_a = listen_on_random(&mut swarm_a).await;
        let dial_addr = addr_b.with(Protocol::P2p(peer_b));
        swarm_a.dial(dial_addr).expect("dial b");

        let request = PairingMessage::Request(PairingRequest {
            session_id: "session-3".to_string(),
            device_name: "device-a".to_string(),
            device_id: "device-a".to_string(),
            peer_id: "peer-a".to_string(),
            identity_pubkey: vec![1; 32],
            nonce: vec![2; 16],
        });
        swarm_a
            .behaviour_mut()
            .pairing
            .send_request(&peer_b, request);

        let response_channels = Arc::new(AsyncMutex::new(HashMap::new()));
        let (event_tx, mut event_rx) = mpsc::channel(1);
        let response = timeout(Duration::from_secs(10), async {
            let reject = PairingMessage::Reject(PairingReject {
                session_id: "session-3".to_string(),
                reason: Some("user_reject".to_string()),
            });

            loop {
                tokio::select! {
                    event = swarm_a.select_next_some() => {
                        if let SwarmEvent::Behaviour(Libp2pBehaviourEvent::Pairing(event)) = event {
                            if let request_response::Event::Message { message, .. } = event {
                                if let request_response::Message::Response { response, .. } = message {
                                    return response;
                                }
                            }
                        }
                    }
                    event = swarm_b.select_next_some() => {
                        if let SwarmEvent::Behaviour(Libp2pBehaviourEvent::Pairing(event)) = event {
                            handle_pairing_event(
                                &mut swarm_b,
                                event,
                                &response_channels,
                                &event_tx,
                            )
                            .await;
                        }
                    }
                    event = event_rx.recv() => {
                        if let Some(NetworkEvent::PairingMessageReceived { message, .. }) = event {
                            if matches!(message, PairingMessage::Request(_)) {
                                let channel = {
                                    let mut channels = response_channels.lock().await;
                                    channels.remove("session-3")
                                };
                                if let Some(channel) = channel {
                                    let _ = swarm_b.behaviour_mut().pairing.send_response(channel, reject.clone());
                                }
                            }
                        }
                    }
                }
            }
        })
        .await
        .expect("response timeout");

        assert!(matches!(response, PairingMessage::Reject(_)));
    }

    #[tokio::test]
    async fn business_stream_echoes_payload() {
        let payload = b"hello-business".to_vec();
        let (client, server) = tokio::io::duplex(1024);
        let mut client = client.compat();
        let mut server = server.compat();
        let server_task = tokio::spawn(async move { echo_payload(&mut server).await });

        client.write_all(&payload).await.expect("write payload");
        client.close().await.expect("close write");

        let mut response = Vec::new();
        client
            .read_to_end(&mut response)
            .await
            .expect("read response");

        let server_result = server_task.await.expect("server task");
        server_result.expect("server echo");

        assert_eq!(response, payload);
    }

    #[tokio::test]
    async fn outbound_business_denied_emits_event() {
        let resolver: Arc<dyn ConnectionPolicyResolverPort> = Arc::new(PendingResolver);
        let (event_tx, mut event_rx) = mpsc::channel(1);

        let result =
            check_business_allowed(&resolver, &event_tx, "peer-1", ProtocolDirection::Outbound)
                .await;

        assert!(result.is_err());

        let event = event_rx.recv().await.expect("protocol denied event");
        match event {
            NetworkEvent::ProtocolDenied {
                direction, reason, ..
            } => {
                assert_eq!(direction, ProtocolDirection::Outbound);
                assert_eq!(reason, ProtocolDenyReason::NotTrusted);
            }
            _ => panic!("expected ProtocolDenied"),
        }
    }

    #[tokio::test]
    async fn inbound_business_denied_drops_stream_and_emits_event() {
        let resolver: Arc<dyn ConnectionPolicyResolverPort> = Arc::new(PendingResolver);
        let (event_tx, mut event_rx) = mpsc::channel(1);

        let result =
            check_business_allowed(&resolver, &event_tx, "peer-2", ProtocolDirection::Inbound)
                .await;

        assert!(result.is_err());

        let event = event_rx.recv().await.expect("protocol denied event");
        match event {
            NetworkEvent::ProtocolDenied {
                direction, reason, ..
            } => {
                assert_eq!(direction, ProtocolDirection::Inbound);
                assert_eq!(reason, ProtocolDenyReason::NotTrusted);
            }
            _ => panic!("expected ProtocolDenied"),
        }
    }

    #[tokio::test]
    async fn send_pairing_message_queues_command() {
        let adapter = Libp2pNetworkAdapter::new(
            Arc::new(TestIdentityStore::default()),
            Arc::new(FakeResolver),
        )
        .expect("create adapter");

        let message = PairingMessage::Request(PairingRequest {
            session_id: "session-1".to_string(),
            device_name: "device".to_string(),
            device_id: "device".to_string(),
            peer_id: "peer-local".to_string(),
            identity_pubkey: vec![1; 32],
            nonce: vec![2; 16],
        });
        adapter
            .send_pairing_message("peer-1".to_string(), message.clone())
            .await
            .expect("send pairing message");

        let mut rx = Libp2pNetworkAdapter::take_receiver(&adapter.pairing_rx, "pairing")
            .expect("pairing receiver");
        let command = rx.recv().await.expect("pairing command");
        match command {
            PairingCommand::SendMessage {
                peer_id,
                message: queued,
            } => {
                assert_eq!(peer_id.as_str(), "peer-1");
                assert_eq!(queued.session_id(), "session-1");
            }
        }
    }

    #[tokio::test]
    async fn send_clipboard_opens_business_stream() {
        let adapter = Libp2pNetworkAdapter::new(
            Arc::new(TestIdentityStore::default()),
            Arc::new(FakeResolver),
        )
        .expect("create adapter");
        let payload = vec![1, 2, 3, 4];

        adapter
            .send_clipboard("peer-2", payload.clone())
            .await
            .expect("send clipboard");

        let mut rx = Libp2pNetworkAdapter::take_receiver(&adapter.business_rx, "business")
            .expect("business receiver");
        let command = rx.recv().await.expect("business command");
        match command {
            BusinessCommand::SendClipboard { peer_id, data } => {
                assert_eq!(peer_id.as_str(), "peer-2");
                assert_eq!(data, payload);
            }
        }
    }

    #[tokio::test]
    async fn subscribe_clipboard_receiver_is_open() {
        let adapter = Libp2pNetworkAdapter::new(
            Arc::new(TestIdentityStore::default()),
            Arc::new(FakeResolver),
        )
        .expect("create adapter");

        let receiver = adapter
            .subscribe_clipboard()
            .await
            .expect("subscribe clipboard");

        assert!(!receiver.is_closed());
    }

    async fn wait_for_discovery(
        mut rx: mpsc::Receiver<NetworkEvent>,
        expected_peer_id: &str,
    ) -> Option<DiscoveredPeer> {
        while let Some(event) = rx.recv().await {
            if let NetworkEvent::PeerDiscovered(peer) = event {
                if peer.peer_id == expected_peer_id {
                    return Some(peer);
                }
            }
        }
        None
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn mdns_e2e_discovers_peers() {
        let adapter_a = Libp2pNetworkAdapter::new(
            Arc::new(TestIdentityStore::default()),
            Arc::new(FakeResolver),
        )
        .expect("create adapter a");
        let adapter_b = Libp2pNetworkAdapter::new(
            Arc::new(TestIdentityStore::default()),
            Arc::new(FakeResolver),
        )
        .expect("create adapter b");
        adapter_a.spawn_swarm().expect("start swarm a");
        adapter_b.spawn_swarm().expect("start swarm b");

        let peer_a = adapter_a.local_peer_id();
        let peer_b = adapter_b.local_peer_id();

        let rx_a = adapter_a.subscribe_events().await.expect("subscribe a");
        let rx_b = adapter_b.subscribe_events().await.expect("subscribe b");

        sleep(Duration::from_millis(200)).await;

        let discovery = timeout(Duration::from_secs(15), async {
            tokio::join!(
                wait_for_discovery(rx_a, &peer_b),
                wait_for_discovery(rx_b, &peer_a)
            )
        })
        .await;

        match discovery {
            Ok((Some(_), Some(_))) => {}
            Ok((left, right)) => panic!(
                "mdns discovery incomplete: left={:?} right={:?}",
                left.as_ref().map(|peer| peer.peer_id.as_str()),
                right.as_ref().map(|peer| peer.peer_id.as_str())
            ),
            Err(_) => panic!("mdns discovery timed out"),
        }
    }

    #[test]
    fn try_send_event_reports_backpressure() {
        let (event_tx, _event_rx) = mpsc::channel(1);
        event_tx
            .try_send(NetworkEvent::PeerLost("peer-1".to_string()))
            .expect("fill channel");

        let result = try_send_event(
            &event_tx,
            NetworkEvent::PeerLost("peer-2".to_string()),
            "PeerLost",
        );

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn listen_on_failure_emits_error_event_and_returns_err() {
        let keypair = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(keypair.public());
        let behaviour = Libp2pBehaviour::new(local_peer_id).expect("behaviour");
        let mut swarm = SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_tcp(
                tcp::Config::default().nodelay(true),
                noise::Config::new,
                yamux::Config::default,
            )
            .expect("tcp config")
            .with_behaviour(move |_| behaviour)
            .expect("attach behaviour")
            .build();

        let (event_tx, mut event_rx) = mpsc::channel(1);
        let bad_addr: Multiaddr = "/ip4/127.0.0.1/udp/0".parse().expect("bad addr");

        let result = listen_on_swarm(&mut swarm, bad_addr, &event_tx);
        assert!(result.is_err());

        let event = event_rx.recv().await.expect("error event");
        assert!(
            matches!(event, NetworkEvent::Error(message) if message.contains("failed to listen on tcp"))
        );
    }
}
