# Connection Gating (New Architecture) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement new-architecture connection gating so unpaired peers can only use pairing protocol while paired peers can use business streams, with full logging and event reporting.

**Architecture:** Add pure domain ConnectionPolicy in uc-core, resolve policy in uc-app using PairedDeviceRepositoryPort, and enforce only at uc-platform protocol entrypoints. uc-platform must not infer pairing state; it only calls the resolver and applies the returned allowlist. ConnectionPolicy only inspects allow/deny state (Pending/Trusted/Revoked) and ignores process metadata.

**Tech Stack:** Rust, libp2p (request-response + stream), tokio, serde, thiserror, tracing/log.

---

### Task 1: Add ConnectionPolicy domain model (uc-core)

**Files:**

- Create: `src-tauri/crates/uc-core/src/network/connection_policy.rs`
- Modify: `src-tauri/crates/uc-core/src/network/mod.rs`

**Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::PairingState;

    #[test]
    fn pending_allows_only_pairing() {
        let allowed = ConnectionPolicy::allowed_protocols(PairingState::Pending);
        assert!(allowed.allows(ProtocolKind::Pairing));
        assert!(!allowed.allows(ProtocolKind::Business));
    }

    #[test]
    fn trusted_allows_pairing_and_business() {
        let allowed = ConnectionPolicy::allowed_protocols(PairingState::Trusted);
        assert!(allowed.allows(ProtocolKind::Pairing));
        assert!(allowed.allows(ProtocolKind::Business));
    }

    #[test]
    fn revoked_allows_pairing_only() {
        let allowed = ConnectionPolicy::allowed_protocols(PairingState::Revoked);
        assert!(allowed.allows(ProtocolKind::Pairing));
        assert!(!allowed.allows(ProtocolKind::Business));
    }
}
```

**Step 2: Run tests to verify failure**

Run from `src-tauri/`: `cargo test -p uc-core connection_policy`  
Expected: compile error (ConnectionPolicy not found).

**Step 3: Write minimal implementation**

```rust
use crate::network::PairingState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtocolKind {
    Pairing,
    Business,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllowedProtocols {
    pairing: bool,
    business: bool,
}

impl AllowedProtocols {
    pub fn allows(&self, kind: ProtocolKind) -> bool {
        match kind {
            ProtocolKind::Pairing => self.pairing,
            ProtocolKind::Business => self.business,
        }
    }
}

pub struct ConnectionPolicy;

impl ConnectionPolicy {
    pub fn allowed_protocols(state: PairingState) -> AllowedProtocols {
        match state {
            PairingState::Trusted => AllowedProtocols { pairing: true, business: true },
            PairingState::Pending => AllowedProtocols { pairing: true, business: false },
            PairingState::Revoked => AllowedProtocols { pairing: true, business: false },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedConnectionPolicy {
    pub pairing_state: PairingState,
    pub allowed: AllowedProtocols,
}
```

**Step 4: Run tests to verify pass**

Run from `src-tauri/`: `cargo test -p uc-core connection_policy`  
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-core/src/network/connection_policy.rs src-tauri/crates/uc-core/src/network/mod.rs
git commit -m "feat: add connection policy domain model"
```

---

### Task 2: Add protocol denial event (uc-core)

**Files:**

- Modify: `src-tauri/crates/uc-core/src/network/events.rs`
- Modify: `src-tauri/crates/uc-core/src/network/mod.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn protocol_denied_event_serializes() {
    use crate::network::PairingState;

    let event = NetworkEvent::ProtocolDenied {
        peer_id: "peer-1".to_string(),
        protocol_id: "/uc-business/1.0.0".to_string(),
        pairing_state: PairingState::Pending,
        direction: ProtocolDirection::Inbound,
        reason: ProtocolDenyReason::NotTrusted,
    };

    let json = serde_json::to_string(&event).unwrap();
    let restored: NetworkEvent = serde_json::from_str(&json).unwrap();
    match restored {
        NetworkEvent::ProtocolDenied { .. } => {}
        _ => panic!("expected ProtocolDenied"),
    }
}
```

**Step 2: Run tests to verify failure**

Run from `src-tauri/`: `cargo test -p uc-core protocol_denied_event_serializes`  
Expected: compile error (ProtocolDenied not found).

**Step 3: Write minimal implementation**

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProtocolDirection {
    Inbound,
    Outbound,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProtocolDenyReason {
    NotTrusted,
    Blocked,
    RepoError,
}

pub enum NetworkEvent {
    // ...
    ProtocolDenied {
        peer_id: String,
        protocol_id: String,
        pairing_state: PairingState,
        direction: ProtocolDirection,
        reason: ProtocolDenyReason,
    },
}
```

**Step 4: Run tests to verify pass**

Run from `src-tauri/`: `cargo test -p uc-core protocol_denied_event_serializes`  
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-core/src/network/events.rs src-tauri/crates/uc-core/src/network/mod.rs
git commit -m "feat: add protocol denial network event"
```

---

### Task 3: Add policy resolver port (uc-core)

**Files:**

- Create: `src-tauri/crates/uc-core/src/ports/connection_policy.rs`
- Modify: `src-tauri/crates/uc-core/src/ports/mod.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn connection_policy_resolver_trait_is_object_safe() {
    struct Dummy;
    #[async_trait::async_trait]
    impl ConnectionPolicyResolverPort for Dummy {
        async fn resolve_for_peer(
            &self,
            _peer_id: &PeerId,
        ) -> Result<ResolvedConnectionPolicy, ConnectionPolicyResolverError> {
            Ok(ResolvedConnectionPolicy {
                pairing_state: PairingState::Pending,
                allowed: ConnectionPolicy::allowed_protocols(PairingState::Pending),
            })
        }
    }

    let _resolver: &dyn ConnectionPolicyResolverPort = &Dummy;
}
```

**Step 2: Run tests to verify failure**

Run from `src-tauri/`: `cargo test -p uc-core connection_policy_resolver_trait_is_object_safe`  
Expected: compile error (ConnectionPolicyResolverPort not found).

**Step 3: Write minimal implementation**

```rust
use async_trait::async_trait;
use crate::ids::PeerId;
use crate::network::connection_policy::ResolvedConnectionPolicy;

#[derive(Debug, thiserror::Error)]
pub enum ConnectionPolicyResolverError {
    #[error("repository error: {0}")]
    Repository(String),
}

#[async_trait]
pub trait ConnectionPolicyResolverPort: Send + Sync {
    async fn resolve_for_peer(
        &self,
        peer_id: &PeerId,
    ) -> Result<ResolvedConnectionPolicy, ConnectionPolicyResolverError>;
}
```

**Step 4: Run tests to verify pass**

Run from `src-tauri/`: `cargo test -p uc-core connection_policy_resolver_trait_is_object_safe`  
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-core/src/ports/connection_policy.rs src-tauri/crates/uc-core/src/ports/mod.rs
git commit -m "feat: add connection policy resolver port"
```

---

### Task 4: Add ResolveConnectionPolicy use case (uc-app)

**Files:**

- Create: `src-tauri/crates/uc-app/src/usecases/pairing/resolve_connection_policy.rs`
- Modify: `src-tauri/crates/uc-app/src/usecases/pairing/mod.rs`
- Modify: `src-tauri/crates/uc-app/src/usecases/mod.rs`

**Step 1: Write the failing tests**

```rust
#[tokio::test]
async fn unpaired_peer_allows_pairing_only() {
    let repo = Arc::new(MockRepo::new(None));
    let uc = ResolveConnectionPolicy::new(repo);
    let resolved = uc.execute(PeerId::from("peer-1")).await.unwrap();
    assert_eq!(resolved.pairing_state, PairingState::Pending);
    assert!(resolved.allowed.allows(ProtocolKind::Pairing));
    assert!(!resolved.allowed.allows(ProtocolKind::Business));
}

#[tokio::test]
async fn trusted_peer_allows_business() {
    let repo = Arc::new(MockRepo::new(Some(PairingState::Trusted)));
    let uc = ResolveConnectionPolicy::new(repo);
    let resolved = uc.execute(PeerId::from("peer-1")).await.unwrap();
    assert_eq!(resolved.pairing_state, PairingState::Trusted);
    assert!(resolved.allowed.allows(ProtocolKind::Business));
}

#[tokio::test]
async fn repo_failure_returns_error() {
    let repo = Arc::new(MockRepo::failing());
    let uc = ResolveConnectionPolicy::new(repo);
    let result = uc.execute(PeerId::from("peer-1")).await;
    assert!(result.is_err());
}
```

**Step 2: Run tests to verify failure**

Run from `src-tauri/`: `cargo test -p uc-app resolve_connection_policy`  
Expected: compile error (ResolveConnectionPolicy not found).

**Step 3: Write minimal implementation**

```rust
pub struct ResolveConnectionPolicy {
    repo: Arc<dyn PairedDeviceRepositoryPort>,
}

impl ResolveConnectionPolicy {
    pub fn new(repo: Arc<dyn PairedDeviceRepositoryPort>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        peer_id: PeerId,
    ) -> Result<ResolvedConnectionPolicy, ResolveConnectionPolicyError> {
        let state = match self.repo.get_by_peer_id(&peer_id).await {
            Ok(Some(device)) => device.pairing_state,
            Ok(None) => PairingState::Pending,
            Err(err) => return Err(ResolveConnectionPolicyError::Repository(err.to_string())),
        };
        Ok(ResolvedConnectionPolicy {
            pairing_state: state,
            allowed: ConnectionPolicy::allowed_protocols(state),
        })
    }
}

#[async_trait::async_trait]
impl ConnectionPolicyResolverPort for ResolveConnectionPolicy {
    async fn resolve_for_peer(
        &self,
        peer_id: &PeerId,
    ) -> Result<ResolvedConnectionPolicy, ConnectionPolicyResolverError> {
        self.execute(peer_id.clone())
            .await
            .map_err(|err| ConnectionPolicyResolverError::Repository(err.to_string()))
    }
}
```

**Step 4: Run tests to verify pass**

Run from `src-tauri/`: `cargo test -p uc-app resolve_connection_policy`  
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/pairing/resolve_connection_policy.rs src-tauri/crates/uc-app/src/usecases/pairing/mod.rs src-tauri/crates/uc-app/src/usecases/mod.rs
git commit -m "feat: add resolve connection policy use case"
```

---

### Task 5: Wire resolver into libp2p adapter (uc-tauri + uc-platform)

**Files:**

- Modify: `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs`
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn adapter_constructs_with_policy_resolver() {
    let resolver = Arc::new(FakeResolver::allow_all());
    let adapter = Libp2pNetworkAdapter::new(identity_store, resolver);
    assert!(adapter.is_ok());
}
```

**Step 2: Run tests to verify failure**

Run from `src-tauri/`: `cargo test -p uc-platform adapter_constructs_with_policy_resolver`  
Expected: compile error (new signature missing).

**Step 3: Update constructor signature and wiring**

```rust
pub struct Libp2pNetworkAdapter {
    // ...
    policy_resolver: Arc<dyn ConnectionPolicyResolverPort>,
}

pub fn new(
    identity_store: Arc<dyn IdentityStorePort>,
    policy_resolver: Arc<dyn ConnectionPolicyResolverPort>,
) -> Result<Self> {
    // ...
}
```

**Step 4: Update bootstrap wiring**

```rust
let policy_resolver = Arc::new(uc_app::usecases::pairing::ResolveConnectionPolicy::new(
    paired_device_repo.clone(),
));
let libp2p_network = Arc::new(Libp2pNetworkAdapter::new(identity_store, policy_resolver)?);
```

**Step 5: Run tests to verify pass**

Run from `src-tauri/`: `cargo test -p uc-platform adapter_constructs_with_policy_resolver`  
Expected: PASS.

**Step 6: Commit**

```bash
git add src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs
git commit -m "feat: wire connection policy resolver into libp2p adapter"
```

---

### Task 6: Enable libp2p protocol features (uc-platform)

**Files:**

- Modify: `src-tauri/crates/uc-platform/Cargo.toml`

**Step 1: Write the failing test**

```rust
#[test]
fn libp2p_request_response_feature_available() {
    let _ = libp2p::request_response::Event::ResponseSent;
}
```

**Step 2: Run tests to verify failure**

Run from `src-tauri/`: `cargo test -p uc-platform libp2p_request_response_feature_available`  
Expected: compile error (feature not enabled).

**Step 3: Update libp2p features**

```toml
libp2p = { version = "0.56", features = [
  "tokio", "tcp", "noise", "yamux", "mdns", "identify", "macros",
  "request-response", "stream",
] }
```

**Step 4: Run tests to verify pass**

Run from `src-tauri/`: `cargo test -p uc-platform libp2p_request_response_feature_available`  
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-platform/Cargo.toml
git commit -m "chore: enable libp2p request-response and stream"
```

---

### Task 7: Implement minimal pairing and business protocols (uc-platform)

**Files:**

- Modify: `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs`

**Step 1: Write the failing tests**

```rust
#[tokio::test]
async fn pairing_request_emits_response_event() {
    // build behaviour with pairing protocol, inject a request and verify a response is sent
}

#[tokio::test]
async fn pairing_rejected_does_not_enable_business() {
    // resolver returns pairing-only, pairing request gets Reject,
    // and no business stream is accepted for that peer
}

#[tokio::test]
async fn business_stream_echoes_payload() {
    // open business stream and verify echo payload
}
```

**Step 2: Run tests to verify failure**

Run from `src-tauri/`: `cargo test -p uc-platform pairing_request_emits_response_event`  
Expected: FAIL (protocol not implemented).

**Step 3: Add protocol constants + message types**

```rust
const PAIRING_PROTOCOL_ID: &str = "/uc-pairing/1.0.0";
const BUSINESS_PROTOCOL_ID: &str = "/uc-business/1.0.0";

#[derive(Debug, Serialize, Deserialize)]
struct PairingHello { session_id: String }

#[derive(Debug, Serialize, Deserialize)]
enum PairingReply {
    Accept,
    Reject { reason: String },
}
```

**Step 4: Add request-response behaviour (pairing)**

```rust
let mut cfg = request_response::Config::default();
cfg.set_request_timeout(Duration::from_secs(10));
let pairing = request_response::json::Behaviour::new(
    [(StreamProtocol::new(PAIRING_PROTOCOL_ID), ProtocolSupport::Full)],
    cfg,
);
```

**Step 5: Add business stream behaviour + echo handler**

```rust
let mut incoming = control.accept(StreamProtocol::new(BUSINESS_PROTOCOL_ID))?;
while let Some((peer, mut stream)) = incoming.next().await {
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).await?;
    stream.write_all(&buf).await?;
}
```

**Step 6: Run tests to verify pass**

Run from `src-tauri/`: `cargo test -p uc-platform pairing_request_emits_response_event`  
Expected: PASS.

**Step 7: Commit**

```bash
git add src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs
git commit -m "feat: add minimal pairing and business protocols"
```

---

### Task 8: Enforce connection gating at protocol entrypoints (uc-platform)

**Files:**

- Modify: `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs`

**Step 1: Write the failing tests**

```rust
#[tokio::test]
async fn outbound_business_denied_emits_event() {
    // resolver returns pairing-only, attempt open_stream -> expect ProtocolDenied event
}

#[tokio::test]
async fn inbound_business_denied_drops_stream_and_emits_event() {
    // resolver returns pairing-only, inbound business stream -> expect ProtocolDenied event
}
```

**Step 2: Run tests to verify failure**

Run from `src-tauri/`: `cargo test -p uc-platform outbound_business_denied_emits_event`  
Expected: FAIL (gating not implemented).

**Step 3: Implement outbound gating before open_stream**

```rust
let resolved = self.policy_resolver.resolve_for_peer(&peer_id).await;
if !resolved.allowed.allows(ProtocolKind::Business) {
    warn!("protocol denied", /* peer_id, protocol_id, pairing_state, direction, reason */);
    let _ = self.event_tx.send(NetworkEvent::ProtocolDenied {
        // pairing_state: resolved.pairing_state
    }).await;
    return Err(anyhow!("business protocol denied"));
}
```

**Step 4: Implement inbound gating after accept**

```rust
// NOTE: Gating happens after accept in MVP.
// Future hardening may move this check earlier or add rate limits.
if !resolved.allowed.allows(ProtocolKind::Business) {
    warn!("protocol denied", /* peer_id, protocol_id, pairing_state, direction, reason */);
    let _ = event_tx.send(NetworkEvent::ProtocolDenied {
        // pairing_state: resolved.pairing_state
    }).await;
    continue; // drop stream
}
```

**Step 5: Run tests to verify pass**

Run from `src-tauri/`: `cargo test -p uc-platform outbound_business_denied_emits_event`  
Expected: PASS.

**Step 6: Commit**

```bash
git add src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs
git commit -m "feat: enforce connection gating at protocol entrypoints"
```

---

### Task 9: Update NetworkPort methods to use new protocols (uc-platform)

**Files:**

- Modify: `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs`

**Step 1: Write the failing tests**

```rust
#[tokio::test]
async fn initiate_pairing_sends_request_response() {
    // call initiate_pairing and verify request is sent for /uc-pairing/1.0.0
}

#[tokio::test]
async fn send_clipboard_opens_business_stream() {
    // call send_clipboard and verify open_stream is attempted for /uc-business/1.0.0
}
```

**Step 2: Run tests to verify failure**

Run from `src-tauri/`: `cargo test -p uc-platform initiate_pairing_sends_request_response`  
Expected: FAIL (methods not implemented).

**Step 3: Implement minimal protocol usage**

```rust
async fn initiate_pairing(&self, peer_id: String, _device_name: String) -> Result<String> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let request = PairingHello { session_id: session_id.clone() };
    self.pairing_sender.send_request(&peer, request);
    Ok(session_id)
}

async fn send_clipboard(&self, peer_id: &str, encrypted_data: Vec<u8>) -> Result<()> {
    // open business stream and write bytes (echo handled by receiver)
}
```

**Step 4: Run tests to verify pass**

Run from `src-tauri/`: `cargo test -p uc-platform initiate_pairing_sends_request_response`  
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs
git commit -m "feat: connect network port to pairing and business protocols"
```

---

### Task 10: Full verification sweep

**Step 1: Run uc-core tests**

Run from `src-tauri/`: `cargo test -p uc-core`  
Expected: PASS.

**Step 2: Run uc-app tests**

Run from `src-tauri/`: `cargo test -p uc-app`  
Expected: PASS.

**Step 3: Run uc-platform tests**

Run from `src-tauri/`: `cargo test -p uc-platform`  
Expected: PASS.

**Step 4: Commit**

```bash
git add -A
git commit -m "test: verify connection gating stack"
```

---

### Optional Task (Non-blocking): Centralize protocol IDs

**Files:**

- Create: `src-tauri/crates/uc-core/src/network/protocol_ids.rs`
- Modify: `src-tauri/crates/uc-core/src/network/mod.rs`

**Goal:** Avoid hardcoded strings by centralizing `/uc-pairing/1.0.0` and `/uc-business/1.0.0`.

```rust
pub enum ProtocolId {
    Pairing,
    Business,
}

impl ProtocolId {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProtocolId::Pairing => "/uc-pairing/1.0.0",
            ProtocolId::Business => "/uc-business/1.0.0",
        }
    }
}
```
