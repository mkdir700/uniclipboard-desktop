use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use libp2p::{
    futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, StreamExt},
    identity, mdns, noise,
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, StreamProtocol, SwarmBuilder,
};
use libp2p_stream as stream;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use uc_core::network::{
    ClipboardMessage, ConnectedPeer, DiscoveredPeer, NetworkEvent, PairingMessage, PairingState,
    ProtocolDenyReason, ProtocolDirection, ProtocolId, ProtocolKind, ResolvedConnectionPolicy,
};
use uc_core::ports::{
    ConnectionPolicyResolverPort, IdentityStorePort, NetworkControlPort, NetworkPort,
};

use super::pairing_stream::service::{
    PairingStreamConfig, PairingStreamError, PairingStreamService,
};
use crate::identity_store::load_or_create_identity;
const BUSINESS_PROTOCOL_ID: &str = ProtocolId::Business.as_str();

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
    stream: stream::Behaviour,
}

#[derive(Debug)]
enum Libp2pBehaviourEvent {
    Mdns(mdns::Event),
    Stream,
}

impl From<mdns::Event> for Libp2pBehaviourEvent {
    fn from(event: mdns::Event) -> Self {
        Self::Mdns(event)
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
        let stream = stream::Behaviour::new();
        Ok(Self { mdns, stream })
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
    business_tx: mpsc::Sender<BusinessCommand>,
    business_rx: Mutex<Option<mpsc::Receiver<BusinessCommand>>>,
    keypair: Mutex<Option<identity::Keypair>>,
    policy_resolver: Arc<dyn ConnectionPolicyResolverPort>,
    stream_control: Mutex<Option<stream::Control>>,
    pairing_service: Mutex<Option<PairingStreamService>>,
}

impl Libp2pNetworkAdapter {
    pub fn new(
        identity_store: Arc<dyn IdentityStorePort>,
        policy_resolver: Arc<dyn ConnectionPolicyResolverPort>,
    ) -> Result<Self> {
        let keypair = load_or_create_identity(identity_store.as_ref())
            .map_err(|e| anyhow!("failed to load libp2p identity: {e}"))?;
        let local_peer_id = PeerId::from(keypair.public()).to_string();
        let local_identity_pubkey = keypair
            .public()
            .try_into_ed25519()
            .map_err(|err| anyhow!("failed to extract ed25519 public key: {err}"))?
            .to_bytes()
            .to_vec();
        let (event_tx, event_rx) = mpsc::channel(64);
        let (clipboard_tx, clipboard_rx) = mpsc::channel(64);
        let (business_tx, business_rx) = mpsc::channel(64);
        let pairing_service = Mutex::new(None);

        Ok(Self {
            local_peer_id,
            local_identity_pubkey,
            caches: Arc::new(RwLock::new(PeerCaches::new())),
            event_tx,
            event_rx: Mutex::new(Some(event_rx)),
            clipboard_tx,
            clipboard_rx: Mutex::new(Some(clipboard_rx)),
            business_tx,
            business_rx: Mutex::new(Some(business_rx)),
            keypair: Mutex::new(Some(keypair)),
            policy_resolver,
            stream_control: Mutex::new(None),
            pairing_service,
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
        let pairing_service = PairingStreamService::new(
            stream_control.clone(),
            self.event_tx.clone(),
            PairingStreamConfig::default(),
        );
        pairing_service.spawn_accept_loop();
        {
            let mut guard = self
                .stream_control
                .lock()
                .map_err(|_| anyhow!("stream control mutex poisoned"))?;
            *guard = Some(stream_control.clone());
        }
        {
            let mut guard = self
                .pairing_service
                .lock()
                .map_err(|_| anyhow!("pairing service mutex poisoned"))?;
            *guard = Some(pairing_service);
        }

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
        let business_rx = Self::take_receiver(&self.business_rx, "business command")?;
        tokio::spawn(async move {
            run_swarm(swarm, caches, event_tx, policy_resolver, business_rx).await;
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

    async fn open_pairing_session(&self, peer_id: String, session_id: String) -> Result<()> {
        let service = {
            let guard = self
                .pairing_service
                .lock()
                .map_err(|_| anyhow!("pairing service mutex poisoned"))?;
            guard
                .as_ref()
                .cloned()
                .ok_or_else(|| anyhow!("pairing service not initialized"))?
        };
        match service
            .open_pairing_session(peer_id.clone(), session_id)
            .await
        {
            Ok(()) => Ok(()),
            Err(err) => {
                handle_pairing_open_error(&self.policy_resolver, &self.event_tx, &peer_id, &err)
                    .await;
                Err(err)
            }
        }
    }

    async fn send_pairing_on_session(
        &self,
        session_id: String,
        message: PairingMessage,
    ) -> Result<()> {
        let service = {
            let guard = self
                .pairing_service
                .lock()
                .map_err(|_| anyhow!("pairing service mutex poisoned"))?;
            guard
                .as_ref()
                .cloned()
                .ok_or_else(|| anyhow!("pairing service not initialized"))?
        };
        service.send_pairing_on_session(session_id, message).await
    }

    async fn close_pairing_session(
        &self,
        session_id: String,
        reason: Option<String>,
    ) -> Result<()> {
        let service = {
            let guard = self
                .pairing_service
                .lock()
                .map_err(|_| anyhow!("pairing service mutex poisoned"))?;
            guard
                .as_ref()
                .cloned()
                .ok_or_else(|| anyhow!("pairing service not initialized"))?
        };
        service.close_pairing_session(session_id, reason).await
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

async fn emit_protocol_denied(
    event_tx: &mpsc::Sender<NetworkEvent>,
    peer_id: String,
    protocol_id: &str,
    pairing_state: PairingState,
    direction: ProtocolDirection,
    reason: ProtocolDenyReason,
) {
    if let Err(err) = event_tx
        .send(NetworkEvent::ProtocolDenied {
            peer_id,
            protocol_id: protocol_id.to_string(),
            pairing_state,
            direction,
            reason,
        })
        .await
    {
        warn!("failed to emit protocol denied event: {err}");
    }
}

async fn handle_pairing_open_error(
    policy_resolver: &Arc<dyn ConnectionPolicyResolverPort>,
    event_tx: &mpsc::Sender<NetworkEvent>,
    peer_id: &str,
    error: &anyhow::Error,
) {
    if let Some(pairing_error) = error.downcast_ref::<PairingStreamError>() {
        if matches!(pairing_error, PairingStreamError::UnsupportedProtocol) {
            let peer = uc_core::PeerId::from(peer_id);
            let pairing_state = match policy_resolver.resolve_for_peer(&peer).await {
                Ok(resolved) => resolved.pairing_state,
                Err(err) => {
                    warn!("policy resolver failed for pairing protocol peer={peer_id}: {err}");
                    PairingState::Pending
                }
            };
            emit_protocol_denied(
                event_tx,
                peer_id.to_string(),
                ProtocolId::Pairing.as_str(),
                pairing_state,
                ProtocolDirection::Outbound,
                ProtocolDenyReason::NotSupported,
            )
            .await;
        }
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
                    BUSINESS_PROTOCOL_ID,
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
                BUSINESS_PROTOCOL_ID,
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
    use libp2p::Multiaddr;
    use std::sync::{Arc, Mutex};
    use tokio::time::{sleep, timeout, Duration};
    use tokio_util::compat::TokioAsyncReadCompatExt;
    use uc_core::network::{ConnectionPolicy, PairingState, ResolvedConnectionPolicy};
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

    #[tokio::test]
    async fn adapter_constructs_with_policy_resolver() {
        let resolver: Arc<dyn ConnectionPolicyResolverPort> = Arc::new(FakeResolver);
        let adapter = Libp2pNetworkAdapter::new(Arc::new(TestIdentityStore::default()), resolver);
        assert!(adapter.is_ok());
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
                protocol_id,
                direction,
                reason,
                ..
            } => {
                assert_eq!(protocol_id, BUSINESS_PROTOCOL_ID);
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
                protocol_id,
                direction,
                reason,
                ..
            } => {
                assert_eq!(protocol_id, BUSINESS_PROTOCOL_ID);
                assert_eq!(direction, ProtocolDirection::Inbound);
                assert_eq!(reason, ProtocolDenyReason::NotTrusted);
            }
            _ => panic!("expected ProtocolDenied"),
        }
    }

    #[tokio::test]
    async fn legacy_pairing_denied_emits_protocol_id() {
        let resolver: Arc<dyn ConnectionPolicyResolverPort> = Arc::new(FakeResolver);
        let (event_tx, mut event_rx) = mpsc::channel(1);
        let error = anyhow::Error::new(PairingStreamError::UnsupportedProtocol);

        handle_pairing_open_error(&resolver, &event_tx, "peer-legacy", &error).await;

        let event = event_rx.recv().await.expect("protocol denied event");
        match event {
            NetworkEvent::ProtocolDenied {
                peer_id,
                protocol_id,
                pairing_state,
                direction,
                reason,
            } => {
                assert_eq!(peer_id, "peer-legacy");
                assert_eq!(protocol_id, ProtocolId::Pairing.as_str());
                assert_eq!(pairing_state, PairingState::Trusted);
                assert_eq!(direction, ProtocolDirection::Outbound);
                assert_eq!(reason, ProtocolDenyReason::NotSupported);
            }
            _ => panic!("expected ProtocolDenied"),
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

    #[test]
    fn adapter_exposes_raw_identity_pubkey() {
        let adapter = Libp2pNetworkAdapter::new(
            Arc::new(TestIdentityStore::default()),
            Arc::new(FakeResolver),
        )
        .expect("create adapter");

        let pubkey = adapter.local_identity_pubkey();
        assert_eq!(pubkey.len(), 32);
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
