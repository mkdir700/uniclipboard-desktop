# P2P Pairing over libp2p-stream Migration Implementation Plan

REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace pairing transport from `libp2p::request_response` to `libp2p_stream` with length-delimited framing and session-based streams (one `session_id` = one stream), with bounded concurrency/backpressure.

**Architecture:**

- Hexagonal boundary stays intact: `uc-core` defines ports + domain types only, no `libp2p`/`tokio` dependencies.
- Transport details live in `uc-platform` (adapter implementations). `uc-tauri` remains the wiring/host integration.
- Pairing is “session-driven”: initiator explicitly opens a stream per session; both sides send multiple messages on that stream, then close.

**Tech Stack:** Rust, tokio, tracing, anyhow, serde_json, libp2p (incl. `libp2p_stream`), futures `AsyncRead/AsyncWrite`.

---

## Pre-flight Notes (Read Before Coding)

- **Inbound streams are dropped if you fall behind.** `libp2p_stream::IncomingStreams` is lazy; if your accept loop is not polled fast enough, libp2p will drop inbound streams.
  - Source: `rust-libp2p/protocols/stream/README.md` (see docs.rs / Context7 snippet).
- **Don’t spawn unbounded per-stream tasks.** Unbounded spawning is equivalent to unbounded buffering and can lead to memory pressure.
- **Dial/open under heavy load can backpressure or drop.** There are known issues where stream dial requests can be dropped under high load if internal bounded channels fill (see `libp2p/rust-libp2p#6157`).

## Current State (Concrete Anchors)

- Pairing currently uses request/response, with a session map to store `ResponseChannel`:
  - Modify: `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs`
    - `pairing: request_response::json::Behaviour<PairingMessage, PairingMessage>`
    - `pairing_response_channels: HashMap<session_id, ResponseChannel<...>>`
- There is already a `libp2p_stream` behaviour used for “business” payload echo:
  - `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs`
    - `stream: libp2p_stream::Behaviour`
    - `control.accept(...)` and `control.open_stream(...)`
- Protocol ID definitions live in:
  - `src-tauri/crates/uc-core/src/network/protocol_ids.rs`
- Pairing action loop currently sends via `NetworkPort::send_pairing_message`:
  - `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`

---

## Task 1: Add New Protocol IDs (Stream Namespaces)

**Files:**

- Modify: `src-tauri/crates/uc-core/src/network/protocol_ids.rs`
- Test: `src-tauri/crates/uc-core/src/network/protocol_ids.rs`
- Modify: `src-tauri/crates/uc-core/src/network/events.rs`

**Step 1: Write failing tests for new IDs**

Update the existing unit test to assert the new IDs:

```rust
#[test]
fn protocol_id_strings_match_expected_values() {
    assert_eq!(ProtocolId::PairingStream.as_str(), "/uniclipboard/pairing-stream/1.0.0");
    assert_eq!(ProtocolId::Business.as_str(), "/uniclipboard/business/1.0.0");
}
```

Expected: compile FAIL because `PairingStream` doesn’t exist and string values differ.

**Step 2: Run tests to verify failure**

Run from `src-tauri/`:

`cargo test -p uc-core`

Expected: FAIL with missing enum variant / assertion mismatch.

**Step 3: Minimal implementation**

In `ProtocolId`:

```rust
pub enum ProtocolId {
    PairingStream,
    Business,
}

impl ProtocolId {
    pub const fn as_str(&self) -> &'static str {
        match self {
            ProtocolId::PairingStream => "/uniclipboard/pairing-stream/1.0.0",
            ProtocolId::Business => "/uniclipboard/business/1.0.0",
        }
    }
}
```

**Step 4: Update `NetworkEvent::ProtocolDenied` test fixture string**

Update the literal in `src-tauri/crates/uc-core/src/network/events.rs`:

```rust
protocol_id: "/uniclipboard/business/1.0.0".to_string(),
```

**Step 5: Re-run tests**

Run from `src-tauri/`:

`cargo test -p uc-core`

Expected: PASS.

**Step 6: Commit**

```bash
git add src-tauri/crates/uc-core/src/network/protocol_ids.rs src-tauri/crates/uc-core/src/network/events.rs
git commit -m "feat: add uniclipboard stream protocol ids"
```

---

## Task 2: Add Length-Delimited Framing Helpers (uc-platform)

**Files:**

- Create: `src-tauri/crates/uc-platform/src/adapters/pairing_stream/framing.rs`
- Create: `src-tauri/crates/uc-platform/src/adapters/pairing_stream/framing_test.rs`

**Step 1: Write failing tests (happy path)**

Create a tokio test that round-trips a JSON payload through a duplex stream.

```rust
#[tokio::test]
async fn framing_round_trips_single_frame() {
    let (client, server) = tokio::io::duplex(64 * 1024);
    let mut client = client.compat();
    let mut server = server.compat();

    let payload = br#"{\"k\":\"v\"}"#.to_vec();
    let write_task = tokio::spawn(async move {
        write_length_prefixed(&mut client, &payload).await
    });

    let read = read_length_prefixed(&mut server, MAX_PAIRING_FRAME_BYTES).await;
    let wrote = write_task.await.expect("write task");

    wrote.expect("write ok");
    assert_eq!(read.expect("read ok"), payload);
}
```

Expected: compile FAIL because framing helpers don’t exist.

**Step 2: Run tests to verify failure**

Run from `src-tauri/`:

`cargo test -p uc-platform`

Expected: FAIL with missing imports/symbols.

**Step 3: Implement minimal framing helpers**

Implement u32 big-endian length prefix:

```rust
pub const MAX_PAIRING_FRAME_BYTES: usize = 16 * 1024;

pub async fn write_length_prefixed<W>(w: &mut W, payload: &[u8]) -> anyhow::Result<()>
where
    W: AsyncWrite + Unpin,
{
    let len: u32 = payload
        .len()
        .try_into()
        .map_err(|_| anyhow::anyhow!("frame too large for u32"))?;
    w.write_all(&len.to_be_bytes()).await?;
    w.write_all(payload).await?;
    w.flush().await?;
    Ok(())
}

pub async fn read_length_prefixed<R>(r: &mut R, max: usize) -> anyhow::Result<Vec<u8>>
where
    R: AsyncRead + Unpin,
{
    let mut len_buf = [0u8; 4];
    r.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    if len > max {
        return Err(anyhow::anyhow!("frame exceeds max: {len} > {max}"));
    }
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf).await?;
    Ok(buf)
}
```

**Step 4: Add failing tests for oversize frames**

```rust
#[tokio::test]
async fn framing_rejects_oversize_frame() {
    let (client, server) = tokio::io::duplex(64 * 1024);
    let mut client = client.compat();
    let mut server = server.compat();

    let oversize = vec![0u8; MAX_PAIRING_FRAME_BYTES + 1];
    tokio::spawn(async move {
        // Write raw prefix + payload to simulate malicious peer.
        let len = (oversize.len() as u32).to_be_bytes();
        let _ = client.write_all(&len).await;
        let _ = client.write_all(&oversize).await;
    });

    let err = read_length_prefixed(&mut server, MAX_PAIRING_FRAME_BYTES)
        .await
        .expect_err("should error");
    assert!(err.to_string().contains("exceeds max"));
}
```

**Step 5: Run tests**

Run from `src-tauri/`:

`cargo test -p uc-platform`

Expected: PASS.

**Step 6: Commit**

```bash
git add src-tauri/crates/uc-platform/src/adapters/pairing_stream/framing.rs src-tauri/crates/uc-platform/src/adapters/pairing_stream/framing_test.rs
git commit -m "test: add length-delimited framing helpers"
```

---

## Task 3: Redesign NetworkPort Pairing API (Session-Driven)

**Files:**

- Modify: `src-tauri/crates/uc-core/src/ports/network.rs`
- Modify (implementations/mocks):
  - `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs`
  - `src-tauri/crates/uc-platform/src/adapters/network.rs`
  - `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`
  - `src-tauri/crates/uc-app/src/usecases/pairing/get_local_peer_id.rs`
  - `src-tauri/crates/uc-app/src/usecases/pairing/list_discovered_peers.rs`
  - `src-tauri/crates/uc-app/src/usecases/pairing/list_connected_peers.rs`
  - `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`
  - plus any other test-only NetworkPort mocks found by grep.

**Step 1: Add failing compilation by changing the trait**

In `src-tauri/crates/uc-core/src/ports/network.rs`, replace the pairing section with:

```rust
// === Pairing operations ===
async fn open_pairing_session(&self, peer_id: String, session_id: String) -> Result<()>;
async fn send_pairing_on_session(&self, session_id: String, message: PairingMessage) -> Result<()>;
async fn close_pairing_session(&self, session_id: String, reason: Option<String>) -> Result<()>;
```

Expected: compile FAIL across crates due to missing methods.

**Step 2: Run compile to see all breakages**

Run from `src-tauri/`:

`cargo test -p uc-core`

Expected: PASS (trait compiles).

Then run:

`cargo test -p uc-app`

Expected: FAIL in test mocks implementing `NetworkPort`.

**Step 3: Update all test mocks in uc-app to implement new trait methods**

Minimal no-op implementations:

```rust
async fn open_pairing_session(&self, _peer_id: String, _session_id: String) -> anyhow::Result<()> { Ok(()) }
async fn send_pairing_on_session(&self, _session_id: String, _message: PairingMessage) -> anyhow::Result<()> { Ok(()) }
async fn close_pairing_session(&self, _session_id: String, _reason: Option<String>) -> anyhow::Result<()> { Ok(()) }
```

**Step 4: Update uc-tauri action loop to use session API**

In `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`:

- On `PairingAction::Send { peer_id, message }`:
  - Derive `session_id = message.session_id().to_string()`.
  - Call `open_pairing_session(peer_id.clone(), session_id.clone())` **best-effort**.
  - Then call `send_pairing_on_session(session_id, message)`.
- On `PairingAction::EmitResult { session_id, .. }`:
  - Call `close_pairing_session(session_id.clone(), None)` best-effort.

This keeps `uc-app` orchestrator decoupled from transport while still being session-driven at the port boundary.

**Step 5: Run crate tests**

Run from `src-tauri/`:

`cargo test -p uc-app`

Expected: PASS.

`cargo test -p uc-tauri`

Expected: PASS.

**Step 6: Commit**

```bash
git add src-tauri/crates/uc-core/src/ports/network.rs src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs src-tauri/crates/uc-app/src/usecases/pairing/get_local_peer_id.rs src-tauri/crates/uc-app/src/usecases/pairing/list_discovered_peers.rs src-tauri/crates/uc-app/src/usecases/pairing/list_connected_peers.rs src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs src-tauri/crates/uc-platform/src/adapters/network.rs
git commit -m "refactor: make pairing network API session-driven"
```

---

## Task 4: Implement PairingStream Service in uc-platform

**Files:**

- Modify: `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs`
- Create: `src-tauri/crates/uc-platform/src/adapters/pairing_stream/mod.rs`
- Create: `src-tauri/crates/uc-platform/src/adapters/pairing_stream/service.rs`
- Use existing: `src-tauri/crates/uc-platform/src/adapters/pairing_stream/framing.rs`

### Design (Implementation Guidance)

- Keep one accept loop per protocol, and ensure it is continuously polled.
- Use bounded concurrency:
  - Global semaphore: `MAX_PAIRING_CONCURRENCY = 16`
  - Per-peer semaphore: 2 permits per peer
- Each session has a task with:
  - Reader loop: `read_length_prefixed -> serde_json::from_slice::<PairingMessage>`
  - Writer loop: `mpsc::Receiver<PairingMessage>` to serialize and `write_length_prefixed`
  - `tokio::select!` with read/write timeouts and session idle timeout.
- Observability:
  - session span: `info_span!("pairing.session", peer_id = %peer_id, session_id = %session_id)`
  - warn logs for protocol violation, oversize frames, timeouts.

**Step 1: Add failing tests for service wiring (unit-level)**

Add a unit test module in `pairing_stream/service.rs` that:

- Creates a tokio duplex pair.
- Runs a minimal `SessionTask` reading one frame and emitting a `NetworkEvent::PairingMessageReceived`.

Expected: compile FAIL (service not implemented yet).

**Step 2: Remove request_response pairing from behaviour composition**

In `Libp2pBehaviour`:

- Remove `pairing: request_response::json::Behaviour<...>`
- Remove related out_event plumbing
- Remove `pairing_response_channels`

Expected: compile FAIL until all call sites are updated.

**Step 3: Add PairingStream accept loop**

In `spawn_swarm` (or a new router module called from it):

- `let mut incoming = control.accept(StreamProtocol::new(ProtocolId::PairingStream.as_str()))?;`
- `tokio::spawn` an accept loop that:
  - does minimal work (acquire permits + hand-off)
  - hands off `(peer, stream)` to `PairingStreamService::handle_incoming_stream(peer, stream)`

**Step 4: Implement outbound open/send/close in Libp2pNetworkAdapter**

Implement the new trait methods:

- `open_pairing_session(peer_id, session_id)`:
  - open outbound stream using `Control::open_stream`
  - create `SessionTask` (spawn)
  - store session handle in a session map: `HashMap<SessionId, SessionHandle>`
- `send_pairing_on_session(session_id, message)`:
  - look up session handle; send into writer mailbox
  - if missing: return error (upper layers will surface via logs/UI)
- `close_pairing_session(session_id, reason)`:
  - remove from map; signal task to shutdown; best-effort close underlying stream

**Step 5: Update integration tests**

Replace existing request_response-based pairing tests in `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs` with stream-based tests.

Minimum set:

- `pairing_stream_e2e_request_challenge`
- `pairing_stream_protocol_violation_oversize_frame`
- `pairing_stream_idle_timeout_closes_session`

**Step 6: Run tests**

Run from `src-tauri/`:

`cargo test -p uc-platform`

Expected: PASS.

**Step 7: Commit**

```bash
git add src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs src-tauri/crates/uc-platform/src/adapters/pairing_stream
git commit -m "feat: implement pairing over libp2p-stream sessions"
```

---

## Task 5: Cleanup + Verification

**Files:**

- Modify: `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs`
- Modify: any Cargo deps / feature flags if request_response is no longer needed.
- Modify: docs: `docs/plans/2026-01-30-p2p-pairing-libp2p-stream-plan.md` (optional cross-link)

**Step 1: Remove dead code & dependencies**

- Delete all pairing request_response code paths.
- Ensure no `request_response` remains for pairing.

**Step 2: Full workspace tests**

Run from `src-tauri/`:

`cargo test --workspace`

Expected: PASS.

**Step 3: Commit**

```bash
git add -A
git commit -m "chore: remove request_response pairing remnants"
```

---

## Execution Handoff

Plan complete and saved to `docs/plans/2026-01-30-p2p-pairing-libp2p-stream-implementation-plan.md`.

Two execution options:

1. Subagent-Driven (this session) - I dispatch a fresh subagent per task, review between tasks, fast iteration
2. Parallel Session (separate) - Open new session with executing-plans, batch execution with checkpoints

## Status Update (2026-01-31)

- Tasks 1-6 shipped: protocol IDs, framing helpers, session-based NetworkPort, uc-tauri wiring, PairingStream service, and request_response removal (legacy directory untouched).
- Legacy peers that only speak `/uc-pairing/1.0.0` now trigger `NetworkEvent::ProtocolDenied` with `reason = NotSupported`, while stream clients stay on `/uniclipboard/pairing-stream/1.0.0`.
- Verification snapshot: `cd src-tauri && cargo test --workspace` (warnings limited to unused macOS window-style helper; all tests/doc-tests passed as of 2026-01-31).
