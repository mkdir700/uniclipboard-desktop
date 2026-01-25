# P2P Discovery + Pairing Repository Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement LAN mDNS peer discovery (libp2p) in the new architecture and add a persistent paired-device repository with trust state.

**Architecture:** mDNS discovery is implemented in `uc-platform` via a real `NetworkPort`, maintaining an in-memory discovered peer list with `last_seen`. Pairing persistence is defined in `uc-core` (domain + port), implemented in `uc-infra` with SQLite, wired in `uc-tauri`, and exposed via minimal use cases + commands. Legacy code remains untouched.

**Tech Stack:** Rust, Tauri, libp2p (mdns/identify), tokio, chrono, serde, Diesel + SQLite.

---

### Task 1: Add `last_seen` to core discovered peers

**Files:**

- Modify: `src-tauri/crates/uc-core/src/network/events.rs`
- Test: `src-tauri/crates/uc-core/src/network/events.rs`

**Step 1: Write the failing test**

Update the existing test to include `last_seen` and assert round-trip serialization:

```rust
#[test]
fn test_discovered_peer_serialization() {
    let peer = DiscoveredPeer {
        peer_id: "12D3KooW...".to_string(),
        device_name: Some("Test Device".to_string()),
        device_id: Some("ABC123".to_string()),
        addresses: vec!["/ip4/192.168.1.100/tcp/8000".to_string()],
        discovered_at: Utc::now(),
        last_seen: Utc::now(),
        is_paired: false,
    };

    let json = serde_json::to_string(&peer).unwrap();
    let deserialized: DiscoveredPeer = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.peer_id, peer.peer_id);
    assert_eq!(deserialized.device_name, peer.device_name);
    assert_eq!(deserialized.last_seen, peer.last_seen);
    assert!(!deserialized.is_paired);
}
```

**Step 2: Run test to verify it fails**

Run (from `src-tauri/`):

```bash
cargo test -p uc-core test_discovered_peer_serialization
```

Expected: compile error about missing field `last_seen`.

**Step 3: Write minimal implementation**

Add `last_seen: DateTime<Utc>` to `DiscoveredPeer` and update any instantiations in `uc-core` code to set it.

**Step 4: Run test to verify it passes**

Run (from `src-tauri/`):

```bash
cargo test -p uc-core test_discovered_peer_serialization
```

Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-core/src/network/events.rs
git commit -m "feat(core): add last_seen to discovered peers"
```

---

### Task 2: Implement libp2p mDNS discovery in `uc-platform`

**Files:**

- Modify: `src-tauri/crates/uc-platform/Cargo.toml`
- Modify: `src-tauri/crates/uc-platform/src/adapters/network.rs`
- Test: `src-tauri/crates/uc-platform/src/adapters/network.rs`

**Step 1: Write the failing test**

Add unit tests for pure helper functions that update in-memory peer maps (no real network). Example:

```rust
#[test]
fn test_upsert_discovered_peer_updates_last_seen() {
    let mut peers = HashMap::new();
    let now = Utc::now();
    upsert_discovered_peer(&mut peers, "peer1", vec!["/ip4/1.1.1.1/tcp/1"], now);
    let first_seen = peers.get("peer1").unwrap().last_seen;

    let later = now + chrono::Duration::seconds(1);
    upsert_discovered_peer(&mut peers, "peer1", vec!["/ip4/1.1.1.1/tcp/2"], later);
    let updated = peers.get("peer1").unwrap();

    assert_eq!(updated.addresses.len(), 1);
    assert!(updated.last_seen > first_seen);
}
```

These tests will fail until the helper functions exist.

**Step 2: Run test to verify it fails**

Run (from `src-tauri/`):

```bash
cargo test -p uc-platform network
```

Expected: compile error for missing helper functions.

**Step 3: Write minimal implementation**

1. Add libp2p dependency to `uc-platform/Cargo.toml` (minimal features needed for mdns + identify + transport):

```toml
libp2p = { version = "0.56", features = ["tokio", "tcp", "noise", "yamux", "mdns", "identify", "macros"] }
```

2. Replace `PlaceholderNetworkPort` implementation with a real `Libp2pNetworkPort`:
   - Construct a libp2p Swarm with `mdns` + `identify`.
   - Maintain `discovered_peers: Arc<RwLock<HashMap<String, DiscoveredPeer>>>`.
   - On `mdns::Event::Discovered`: upsert peer, update `last_seen`, emit `NetworkEvent::PeerDiscovered`, log.
   - On `mdns::Event::Expired`: remove or mark stale and emit `NetworkEvent::PeerLost`, log.
   - Implement `get_discovered_peers()` and `local_peer_id()`; keep other methods returning `anyhow!("not implemented")` with logs.
   - Ensure all async errors are logged (no silent failures).

**Step 4: Run test to verify it passes**

Run (from `src-tauri/`):

```bash
cargo test -p uc-platform network
```

Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-platform/Cargo.toml src-tauri/crates/uc-platform/src/adapters/network.rs
git commit -m "feat(platform): add libp2p mdns discovery"
```

---

### Task 3: Add paired device domain + repository port in `uc-core`

**Files:**

- Create: `src-tauri/crates/uc-core/src/network/paired_device.rs`
- Modify: `src-tauri/crates/uc-core/src/network/mod.rs`
- Create: `src-tauri/crates/uc-core/src/ports/paired_device_repository.rs`
- Modify: `src-tauri/crates/uc-core/src/ports/mod.rs`
- Modify: `src-tauri/crates/uc-core/src/ports/errors.rs`
- Modify (optional re-export): `src-tauri/crates/uc-core/src/lib.rs`

**Step 1: Write the failing test**

In `paired_device.rs`, add tests for serialization and state handling:

```rust
#[test]
fn test_paired_device_serialization() {
    let device = PairedDevice {
        peer_id: PeerId::from("12D3KooW..."),
        pairing_state: PairingState::Trusted,
        identity_fingerprint: "fp".to_string(),
        paired_at: Utc::now(),
        last_seen_at: None,
    };

    let json = serde_json::to_string(&device).unwrap();
    let restored: PairedDevice = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.pairing_state, PairingState::Trusted);
    assert_eq!(restored.identity_fingerprint, device.identity_fingerprint);
}
```

**Step 2: Run test to verify it fails**

Run (from `src-tauri/`):

```bash
cargo test -p uc-core paired_device
```

Expected: compile error (module/struct missing).

**Step 3: Write minimal implementation**

1. Define:
   - `PairingState` enum (`Pending`, `Trusted`, `Revoked`)
   - `PairedDevice` struct with fields: `peer_id`, `pairing_state`, `identity_fingerprint`, `paired_at`, `last_seen_at`
2. Add `PairedDeviceRepositoryPort` trait with methods:
   - `get_by_peer_id`, `list_all`, `upsert`, `set_state`, `update_last_seen`, `delete`
3. Add `PairedDeviceRepositoryError` in `ports/errors.rs`.
4. Export in `ports/mod.rs` and `network/mod.rs`.

**Step 4: Run test to verify it passes**

Run (from `src-tauri/`):

```bash
cargo test -p uc-core paired_device
```

Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-core/src/network/paired_device.rs \
  src-tauri/crates/uc-core/src/network/mod.rs \
  src-tauri/crates/uc-core/src/ports/paired_device_repository.rs \
  src-tauri/crates/uc-core/src/ports/mod.rs \
  src-tauri/crates/uc-core/src/ports/errors.rs
git commit -m "feat(core): add paired device domain and port"
```

---

### Task 4: Implement SQLite repository in `uc-infra`

**Files:**

- Create: `src-tauri/migrations/2026-01-24-000000_create_paired_device/up.sql`
- Modify: `src-tauri/crates/uc-infra/src/db/schema.rs`
- Create: `src-tauri/crates/uc-infra/src/db/models/paired_device_row.rs`
- Modify: `src-tauri/crates/uc-infra/src/db/models/mod.rs`
- Create: `src-tauri/crates/uc-infra/src/db/mappers/paired_device_mapper.rs`
- Modify: `src-tauri/crates/uc-infra/src/db/mappers/mod.rs`
- Create: `src-tauri/crates/uc-infra/src/db/repositories/paired_device_repo.rs`
- Modify: `src-tauri/crates/uc-infra/src/db/repositories/mod.rs`

**Step 1: Write the failing test**

In `paired_device_repo.rs`, add tests for CRUD and persistence:

```rust
#[tokio::test]
async fn test_paired_device_persistence() {
    let dir = tempfile::TempDir::new().unwrap();
    let db_path = dir.path().join("paired.db");
    let pool = init_db_pool(db_path.to_str().unwrap()).unwrap();
    let repo = DieselPairedDeviceRepository::new(DieselSqliteExecutor::new(pool.clone()), PairedDeviceRowMapper);

    let device = PairedDevice { /* ... Trusted ... */ };
    repo.upsert(device.clone()).await.unwrap();

    let fresh_pool = init_db_pool(db_path.to_str().unwrap()).unwrap();
    let fresh_repo = DieselPairedDeviceRepository::new(DieselSqliteExecutor::new(fresh_pool), PairedDeviceRowMapper);
    let loaded = fresh_repo.get_by_peer_id(&device.peer_id).await.unwrap();
    assert!(loaded.is_some());
}
```

**Step 2: Run test to verify it fails**

Run (from `src-tauri/`):

```bash
cargo test -p uc-infra paired_device_repo
```

Expected: compile error (repo/module missing).

**Step 3: Write minimal implementation**

1. Migration (single table):
   - `paired_device` table with columns: `peer_id` (PK), `pairing_state`, `identity_fingerprint`, `paired_at`, `last_seen_at`.
2. Diesel schema and model structs.
3. Mapper converts timestamps between `DateTime<Utc>` and `i64` seconds.
4. Repository methods use `DbExecutor` and `on_conflict(peer_id).do_update()`.

**Step 4: Run test to verify it passes**

Run (from `src-tauri/`):

```bash
cargo test -p uc-infra paired_device_repo
```

Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/migrations/2026-01-24-000000_create_paired_device/up.sql \
  src-tauri/crates/uc-infra/src/db/schema.rs \
  src-tauri/crates/uc-infra/src/db/models/paired_device_row.rs \
  src-tauri/crates/uc-infra/src/db/models/mod.rs \
  src-tauri/crates/uc-infra/src/db/mappers/paired_device_mapper.rs \
  src-tauri/crates/uc-infra/src/db/mappers/mod.rs \
  src-tauri/crates/uc-infra/src/db/repositories/paired_device_repo.rs \
  src-tauri/crates/uc-infra/src/db/repositories/mod.rs
git commit -m "feat(infra): add sqlite paired device repository"
```

---

### Task 5: Wire repo + add use cases + commands

**Files:**

- Modify: `src-tauri/crates/uc-app/src/deps.rs`
- Create: `src-tauri/crates/uc-app/src/usecases/pairing/mod.rs`
- Create: `src-tauri/crates/uc-app/src/usecases/pairing/set_pairing_state.rs`
- Create: `src-tauri/crates/uc-app/src/usecases/pairing/list_paired_devices.rs`
- Modify: `src-tauri/crates/uc-app/src/usecases/mod.rs`
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`
- Create: `src-tauri/crates/uc-tauri/src/commands/pairing.rs`
- Modify: `src-tauri/crates/uc-tauri/src/commands/mod.rs`

**Step 1: Write the failing test**

Add uc-app unit tests with a mock repository:

```rust
#[tokio::test]
async fn test_set_pairing_state_updates_repo() {
    let repo = Arc::new(MockPairedDeviceRepo::default());
    let uc = SetPairingState::new(repo.clone());
    uc.execute(PeerId::from("peer"), PairingState::Trusted).await.unwrap();
    assert_eq!(repo.last_state("peer"), PairingState::Trusted);
}
```

**Step 2: Run test to verify it fails**

Run (from `src-tauri/`):

```bash
cargo test -p uc-app pairing
```

Expected: compile error (use case missing).

**Step 3: Write minimal implementation**

1. Add `paired_device_repo: Arc<dyn PairedDeviceRepositoryPort>` to `AppDeps`.
2. Add use cases:
   - `SetPairingState` (Trusted/Revoked)
   - `ListPairedDevices`
3. Update `UseCases` accessor in `uc-tauri` to expose these.
4. Wire repository in `bootstrap/wiring.rs` using existing `DieselSqliteExecutor` and new mapper.
5. Add Tauri commands (minimal):
   - `set_pairing_state(peer_id, state)`
   - `list_paired_devices()`

**Step 4: Run test to verify it passes**

Run (from `src-tauri/`):

```bash
cargo test -p uc-app pairing
cargo test -p uc-tauri
```

Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-app/src/deps.rs \
  src-tauri/crates/uc-app/src/usecases/pairing \
  src-tauri/crates/uc-app/src/usecases/mod.rs \
  src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs \
  src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs \
  src-tauri/crates/uc-tauri/src/commands/pairing.rs \
  src-tauri/crates/uc-tauri/src/commands/mod.rs
git commit -m "feat(app): add paired device use cases and commands"
```

---

### Task 6: Manual verification (acceptance)

**Step 1: Run app on two devices on same LAN**

Expected:

- Each device logs discovered peers with `peer_id`.
- `last_seen` updates over time or entry disappears on peer exit.

**Step 2: Verify pairing repository persistence**

1. Call command to set peer to `Trusted`.
2. Restart app.
3. `list_paired_devices` returns the peer with `Trusted` state.
4. Call command to set `Revoked` and verify state changes.

---

## Notes

- No legacy modules are touched.
- No `unwrap/expect` in production code; tests only.
- All cargo commands must be executed from `src-tauri/`.
- Ensure all async errors are logged (no silent failure paths).
