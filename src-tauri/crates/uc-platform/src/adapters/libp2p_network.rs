use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use libp2p::{
    futures::StreamExt,
    mdns, noise,
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, SwarmBuilder,
};
use log::{debug, error, info, warn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock as StdRwLock};
use tokio::sync::{mpsc, RwLock};
use uc_core::network::{ClipboardMessage, ConnectedPeer, DiscoveredPeer, NetworkEvent};
use uc_core::ports::{IdentityStorePort, NetworkControlPort, NetworkPort};

use crate::identity_store::load_or_create_identity;

pub struct PeerCaches {
    discovered_peers: HashMap<String, DiscoveredPeer>,
    reachable_peers: HashSet<String>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum NetworkStartState {
    NotStarted,
    Starting,
    Started,
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
#[behaviour(out_event = "MdnsBehaviourEvent")]
struct MdnsBehaviour {
    mdns: mdns::tokio::Behaviour,
}

#[derive(Debug)]
enum MdnsBehaviourEvent {
    Mdns(mdns::Event),
}

impl From<mdns::Event> for MdnsBehaviourEvent {
    fn from(event: mdns::Event) -> Self {
        Self::Mdns(event)
    }
}

impl MdnsBehaviour {
    fn new(local_peer_id: PeerId) -> Result<Self> {
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)
            .map_err(|e| anyhow!("failed to create mdns behaviour: {e}"))?;
        Ok(Self { mdns })
    }
}

pub struct Libp2pNetworkAdapter {
    local_peer_id: StdRwLock<Option<String>>,
    caches: Arc<RwLock<PeerCaches>>,
    event_tx: mpsc::Sender<NetworkEvent>,
    event_rx: Mutex<Option<mpsc::Receiver<NetworkEvent>>>,
    clipboard_tx: mpsc::Sender<ClipboardMessage>,
    clipboard_rx: Mutex<Option<mpsc::Receiver<ClipboardMessage>>>,
    identity_store: Arc<dyn IdentityStorePort>,
    start_state: Mutex<NetworkStartState>,
}

impl Libp2pNetworkAdapter {
    pub fn new(identity_store: Arc<dyn IdentityStorePort>) -> Result<Self> {
        let (event_tx, event_rx) = mpsc::channel(64);
        let (clipboard_tx, clipboard_rx) = mpsc::channel(64);

        Ok(Self {
            local_peer_id: StdRwLock::new(None),
            caches: Arc::new(RwLock::new(PeerCaches::new())),
            event_tx,
            event_rx: Mutex::new(Some(event_rx)),
            clipboard_tx,
            clipboard_rx: Mutex::new(Some(clipboard_rx)),
            identity_store,
            start_state: Mutex::new(NetworkStartState::NotStarted),
        })
    }

    pub fn spawn_swarm(&self) -> Result<()> {
        self.mark_starting()?;

        let result = (|| {
            let keypair = load_or_create_identity(self.identity_store.as_ref())
                .map_err(|e| anyhow!("failed to load libp2p identity: {e}"))?;
            let local_peer_id = PeerId::from(keypair.public());
            let local_peer_id_str = local_peer_id.to_string();
            self.set_local_peer_id(Some(local_peer_id_str.clone()))?;
            info!("Loaded libp2p identity: {}", local_peer_id_str);
            let mdns_behaviour = MdnsBehaviour::new(local_peer_id)
                .map_err(|e| anyhow!("failed to create mdns behaviour: {e}"))?;

            let mut swarm = SwarmBuilder::with_existing_identity(keypair)
                .with_tokio()
                .with_tcp(
                    tcp::Config::default().nodelay(true),
                    noise::Config::new,
                    yamux::Config::default,
                )
                .map_err(|e| anyhow!("failed to configure tcp transport: {e}"))?
                .with_behaviour(move |_| mdns_behaviour)
                .map_err(|e| anyhow!("failed to attach mdns behaviour: {e}"))?
                .build();

            listen_on_swarm(
                &mut swarm,
                "/ip4/0.0.0.0/tcp/0"
                    .parse()
                    .map_err(|e| anyhow!("failed to parse listen address: {e}"))?,
                &self.event_tx,
            )?;

            let caches = self.caches.clone();
            let event_tx = self.event_tx.clone();
            tokio::spawn(async move {
                run_swarm(swarm, caches, event_tx).await;
            });
            Ok(())
        })();

        match result {
            Ok(()) => {
                self.mark_started()?;
                Ok(())
            }
            Err(err) => {
                self.reset_start_state()?;
                Err(err)
            }
        }
    }

    fn mark_starting(&self) -> Result<()> {
        let mut guard = self
            .start_state
            .lock()
            .map_err(|_| anyhow!("start state mutex poisoned"))?;
        match *guard {
            NetworkStartState::NotStarted => {
                *guard = NetworkStartState::Starting;
                Ok(())
            }
            NetworkStartState::Starting | NetworkStartState::Started => {
                Err(anyhow!("swarm already started"))
            }
        }
    }

    fn mark_started(&self) -> Result<()> {
        let mut guard = self
            .start_state
            .lock()
            .map_err(|_| anyhow!("start state mutex poisoned"))?;
        *guard = NetworkStartState::Started;
        Ok(())
    }

    fn reset_start_state(&self) -> Result<()> {
        let mut guard = self
            .start_state
            .lock()
            .map_err(|_| anyhow!("start state mutex poisoned"))?;
        *guard = NetworkStartState::NotStarted;
        self.set_local_peer_id(None)?;
        Ok(())
    }

    fn set_local_peer_id(&self, peer_id: Option<String>) -> Result<()> {
        let mut guard = self
            .local_peer_id
            .write()
            .map_err(|_| anyhow!("local peer id lock poisoned"))?;
        *guard = peer_id;
        Ok(())
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
        Err(anyhow!("NetworkPort::send_clipboard not implemented yet"))
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
        match self.local_peer_id.read() {
            Ok(guard) => guard.clone().unwrap_or_default(),
            Err(_) => {
                warn!("local peer id lock poisoned");
                String::new()
            }
        }
    }

    async fn initiate_pairing(&self, _peer_id: String, _device_name: String) -> Result<String> {
        Err(anyhow!("NetworkPort::initiate_pairing not implemented yet"))
    }

    async fn send_pin_response(&self, _session_id: String, _pin_match: bool) -> Result<()> {
        Err(anyhow!(
            "NetworkPort::send_pin_response not implemented yet"
        ))
    }

    async fn send_pairing_rejection(&self, _session_id: String, _peer_id: String) -> Result<()> {
        Err(anyhow!(
            "NetworkPort::send_pairing_rejection not implemented yet"
        ))
    }

    async fn accept_pairing(&self, _session_id: String) -> Result<()> {
        Err(anyhow!("NetworkPort::accept_pairing not implemented yet"))
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

async fn run_swarm(
    mut swarm: Swarm<MdnsBehaviour>,
    caches: Arc<RwLock<PeerCaches>>,
    event_tx: mpsc::Sender<NetworkEvent>,
) {
    info!("libp2p mDNS swarm started");

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::Behaviour(MdnsBehaviourEvent::Mdns(event)) => match event {
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
}

fn listen_on_swarm(
    swarm: &mut Swarm<MdnsBehaviour>,
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
    use libp2p::identity;
    use libp2p::Multiaddr;
    use std::sync::{Arc, Mutex};
    use tokio::time::{sleep, timeout, Duration};

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

    #[derive(Default)]
    struct CountingIdentityStore {
        data: Mutex<Option<Vec<u8>>>,
        load_calls: std::sync::atomic::AtomicUsize,
    }

    impl CountingIdentityStore {
        fn load_count(&self) -> usize {
            self.load_calls.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    impl IdentityStorePort for CountingIdentityStore {
        fn load_identity(&self) -> Result<Option<Vec<u8>>, uc_core::ports::IdentityStoreError> {
            self.load_calls
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let guard = self.data.lock().expect("lock counting identity store");
            Ok(guard.clone())
        }

        fn store_identity(
            &self,
            identity: &[u8],
        ) -> Result<(), uc_core::ports::IdentityStoreError> {
            let mut guard = self.data.lock().expect("lock counting identity store");
            *guard = Some(identity.to_vec());
            Ok(())
        }
    }

    #[tokio::test]
    async fn subscribe_clipboard_receiver_is_open() {
        let adapter = Libp2pNetworkAdapter::new(Arc::new(TestIdentityStore::default()))
            .expect("create adapter");

        let receiver = adapter
            .subscribe_clipboard()
            .await
            .expect("subscribe clipboard");

        assert!(!receiver.is_closed());
    }

    #[test]
    fn new_does_not_load_identity() {
        let store = Arc::new(CountingIdentityStore::default());

        let _adapter = Libp2pNetworkAdapter::new(store.clone()).expect("create adapter");

        assert_eq!(store.load_count(), 0, "identity should not load on new");
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
        let adapter_a = Libp2pNetworkAdapter::new(Arc::new(TestIdentityStore::default()))
            .expect("create adapter a");
        let adapter_b = Libp2pNetworkAdapter::new(Arc::new(TestIdentityStore::default()))
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
        let mdns_behaviour = MdnsBehaviour::new(local_peer_id).expect("mdns behaviour");
        let mut swarm = SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_tcp(
                tcp::Config::default().nodelay(true),
                noise::Config::new,
                yamux::Config::default,
            )
            .expect("tcp config")
            .with_behaviour(move |_| mdns_behaviour)
            .expect("attach mdns behaviour")
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
