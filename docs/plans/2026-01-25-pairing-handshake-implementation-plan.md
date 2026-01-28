# Pairing Handshake Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Complete the new-architecture pairing handshake so users verify short code/fingerprint, both sides persist Trusted, and the existing UI continues to work via legacy event names marked as deprecated.

**Architecture:** Extend the uc-core state machine to cover initiator/responder flows and persistence; forward UI-oriented actions from the orchestrator to the tauri wiring loop, which emits legacy pairing events with deprecation markers. Commands keep legacy names but route to new orchestrator.

**Tech Stack:** Rust (Tokio, Tauri), libp2p adapter, uc-core state machine, uc-app orchestrator, uc-infra repositories.

---

### Task 1: Add state machine tests for full handshake

**Files:**

- Modify: `src-tauri/crates/uc-core/src/network/pairing_state_machine.rs`
- Test: `src-tauri/crates/uc-core/src/network/pairing_state_machine.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_initiator_flow_persists_after_confirm() {
    let mut sm = PairingStateMachine::new_with_local_identity(
        "LocalDevice".to_string(),
        "device-1".to_string(),
        vec![1; 32],
    );

    let challenge = PairingChallenge {
        session_id: "session-1".to_string(),
        pin: "123456".to_string(),
        device_name: "PeerDevice".to_string(),
        device_id: "device-2".to_string(),
        identity_pubkey: vec![2; 32],
        nonce: vec![9; 16],
    };

    let (_state, actions) = sm.handle_event(
        PairingEvent::RecvChallenge {
            session_id: "session-1".to_string(),
            challenge,
        },
        Utc::now(),
    );

    assert!(actions.iter().any(|action| matches!(action, PairingAction::ShowVerification { .. })));
}

#[test]
fn test_responder_flow_persists_after_response() {
    let mut sm = PairingStateMachine::new_with_local_identity(
        "LocalDevice".to_string(),
        "device-1".to_string(),
        vec![1; 32],
    );
    let request = PairingRequest {
        session_id: "session-1".to_string(),
        device_name: "PeerDevice".to_string(),
        device_id: "device-2".to_string(),
        peer_id: "peer-remote".to_string(),
        identity_pubkey: vec![2; 32],
        nonce: vec![9; 16],
    };
    sm.handle_event(
        PairingEvent::RecvRequest {
            session_id: "session-1".to_string(),
            request,
        },
        Utc::now(),
    );
    let (_state, actions) = sm.handle_event(
        PairingEvent::UserAccept {
            session_id: "session-1".to_string(),
        },
        Utc::now(),
    );
    let challenge = actions.iter().find_map(|action| match action {
        PairingAction::Send {
            message: PairingMessage::Challenge(challenge),
            ..
        } => Some(challenge.clone()),
        _ => None,
    }).expect("challenge");

    let response = PairingResponse {
        session_id: "session-1".to_string(),
        pin_hash: crate::crypto::pin_hash::hash_pin(&challenge.pin).unwrap(),
        accepted: true,
    };
    let (_state, actions) = sm.handle_event(
        PairingEvent::RecvResponse {
            session_id: "session-1".to_string(),
            response,
        },
        Utc::now(),
    );

    assert!(actions.iter().any(|action| matches!(action, PairingAction::PersistPairedDevice { .. })));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p uc-core test_initiator_flow_persists_after_confirm`
Expected: FAIL (missing transitions / actions)

**Step 3: Write minimal implementation**

Implement missing transitions in `pairing_state_machine.rs`:

- `RecvChallenge -> WaitingUserVerification` produces `ShowVerification`
- `WaitingUserVerification + UserAccept -> ResponseSent` produces `Send(Response)`
- `ResponseSent + RecvConfirm -> PersistingTrust` produces `PersistPairedDevice`
- `WaitingForResponse + RecvResponse -> PersistingTrust` produces `PersistPairedDevice`
- `PersistingTrust + PersistOk -> Paired` produces `EmitResult(success=true)`
- `PersistingTrust + PersistErr -> Failed` produces `EmitResult(success=false)`

Build `PairedDevice` using `IdentityFingerprint::from_public_key(peer_identity_pubkey)` and
`PairingState::Trusted`. On errors, return `FailureReason::CryptoError`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p uc-core test_initiator_flow_persists_after_confirm`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-core/src/network/pairing_state_machine.rs
git commit -m "feat: close pairing state machine transitions"
```

### Task 2: Forward UI actions from orchestrator

**Files:**

- Modify: `src-tauri/crates/uc-app/src/usecases/pairing/orchestrator.rs`
- Test: `src-tauri/crates/uc-app/src/usecases/pairing/orchestrator.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn test_show_verification_is_forwarded_to_action_channel() {
    let (orchestrator, mut action_rx) = PairingOrchestrator::new(
        PairingConfig::default(),
        Arc::new(MockDeviceRepository),
        "Local".to_string(),
        "device-1".to_string(),
        "peer-local".to_string(),
        vec![1; 32],
    );

    let challenge = PairingChallenge {
        session_id: "session-1".to_string(),
        pin: "123456".to_string(),
        device_name: "Peer".to_string(),
        device_id: "device-2".to_string(),
        identity_pubkey: vec![2; 32],
        nonce: vec![9; 16],
    };

    orchestrator.handle_challenge("session-1", "peer-remote", challenge).await.unwrap();

    let action = tokio::time::timeout(std::time::Duration::from_secs(1), action_rx.recv())
        .await
        .unwrap()
        .unwrap();

    assert!(matches!(action, PairingAction::ShowVerification { .. }));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p uc-app test_show_verification_is_forwarded_to_action_channel`
Expected: FAIL (no action forwarded)

**Step 3: Write minimal implementation**

Update `execute_action` to forward UI-oriented actions to `action_tx`:

```rust
PairingAction::ShowVerification { .. } | PairingAction::EmitResult { .. } => {
    self.action_tx.send(action).await.context("Failed to queue ui action")?;
}
```

Keep `PersistPairedDevice` handled locally and `Send` handled via `send_message`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p uc-app test_show_verification_is_forwarded_to_action_channel`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/pairing/orchestrator.rs
git commit -m "feat: forward pairing ui actions from orchestrator"
```

### Task 3: Emit legacy pairing events with deprecation markers

**Files:**

- Create: `src-tauri/crates/uc-tauri/src/events/p2p_pairing.rs`
- Modify: `src-tauri/crates/uc-tauri/src/events/mod.rs`
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`
- Test: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`

**Step 1: Write the failing test**

Add a unit test verifying the payload builder includes `deprecated` fields:

```rust
#[test]
fn pairing_request_payload_includes_deprecation_marker() {
    let payload = P2PPairingRequestEvent::deprecated("session-1", "peer-1", "Device");
    assert!(payload.deprecated);
    assert!(!payload.deprecated_reason.is_empty());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p uc-tauri pairing_request_payload_includes_deprecation_marker`
Expected: FAIL (struct missing)

**Step 3: Write minimal implementation**

Create event structs with deprecated markers:

```rust
pub struct P2PPairingRequestEvent {
    pub session_id: String,
    pub peer_id: String,
    pub device_name: Option<String>,
    pub deprecated: bool,
    pub deprecated_reason: String,
}

impl P2PPairingRequestEvent {
    pub fn deprecated(session_id: &str, peer_id: &str, device_name: &str) -> Self { ... }
}
```

In `run_pairing_event_loop`, emit `p2p-pairing-request` when receiving `PairingMessage::Request`.
In `run_pairing_action_loop`, handle `ShowVerification` and `EmitResult`:

- `ShowVerification` -> `p2p-pin-ready` (include pin, short_code, local/peer fingerprint, peer device)
- `EmitResult(success=true)` -> `p2p-pairing-complete`
- `EmitResult(success=false)` -> `p2p-pairing-failed`

Each payload includes deprecated markers and logs `warn!`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p uc-tauri pairing_request_payload_includes_deprecation_marker`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/events src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs
git commit -m "feat: emit deprecated pairing events from new flow"
```

### Task 4: Add pairing commands routed to new orchestrator

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/pairing.rs`
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`
- Test: `src-tauri/crates/uc-tauri/tests/pairing_commands_test.rs` (new)

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn initiate_pairing_routes_to_orchestrator() {
    let runtime = build_test_runtime_with_pairing_orchestrator().await;
    let response = initiate_p2p_pairing(runtime.clone(), PairingRequest { peer_id: "peer".into() })
        .await
        .unwrap();
    assert!(response.success);
    assert!(!response.session_id.is_empty());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p uc-tauri initiate_pairing_routes_to_orchestrator`
Expected: FAIL (command missing)

**Step 3: Write minimal implementation**

- Add `pairing_orchestrator: Arc<PairingOrchestrator>` to `AppDeps`.
- Create orchestrator in `wire_dependencies` and pass into `start_background_tasks`.
- In `commands/pairing.rs`, add:
  - `initiate_p2p_pairing` -> `orchestrator.initiate_pairing(peer_id)`
  - `accept_p2p_pairing` -> `orchestrator.user_accept_pairing(session_id)`
  - `reject_p2p_pairing` -> `orchestrator.user_reject_pairing(session_id)`
  - `verify_p2p_pairing_pin` -> map to `user_accept_pairing` with matches bool (reject if false)

**Step 4: Run test to verify it passes**

Run: `cargo test -p uc-tauri initiate_pairing_routes_to_orchestrator`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/pairing.rs src-tauri/crates/uc-tauri/src/bootstrap
git commit -m "feat: route pairing commands to new orchestrator"
```

### Task 5: Persistence and gating verification

**Files:**

- Modify: `src-tauri/crates/uc-core/src/network/pairing_state_machine.rs`
- Test: `src-tauri/crates/uc-tauri/tests/pairing_integration_test.rs` (new)

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn pairing_persists_trusted_and_allows_business() {
    let ctx = TestPairingContext::new().await;
    ctx.complete_pairing().await;
    let device = ctx.repo.get_by_peer_id(&PeerId::from("peer-remote")).await.unwrap().unwrap();
    assert_eq!(device.pairing_state, PairingState::Trusted);
    assert!(ctx.policy.allows_business(&device));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p uc-tauri pairing_persists_trusted_and_allows_business`
Expected: FAIL (incomplete persistence or policy path)

**Step 3: Write minimal implementation**

Ensure `PersistPairedDevice` builds `PairedDevice` with `PairingState::Trusted` and
`identity_fingerprint` derived from `IdentityFingerprint::from_public_key`. Ensure
`PersistOk` transitions to `Paired` and `EmitResult(success=true)`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p uc-tauri pairing_persists_trusted_and_allows_business`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-core/src/network/pairing_state_machine.rs src-tauri/crates/uc-tauri/tests
git commit -m "feat: persist trusted pairing and verify gating"
```

---

Plan complete and saved to `docs/plans/2026-01-25-pairing-handshake-implementation-plan.md`. Two execution options:

1. Subagent-Driven (this session) - I dispatch fresh subagent per task, review between tasks, fast iteration
2. Parallel Session (separate) - Open new session with executing-plans, batch execution with checkpoints

Which approach?
