# Dual-Role Pairing State Machine Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace current pairing state machine and libp2p adapter flow with the new dual-role state machine spec, including protocol updates and tests.

**Architecture:** Update uc-core pairing protocol/state machine first, then adjust uc-app orchestrator to consume new actions/events, finally rework uc-platform libp2p adapter and tests to align with the new message flow. Keep libp2p adapter thin and rely on uc-core for transitions.

**Tech Stack:** Rust, libp2p request_response, tokio, uc-core/uc-app/uc-platform crates

---

### Task 1: Update Pairing Protocol Messages (uc-core)

**Files:**

- Modify: `src-tauri/crates/uc-core/src/network/protocol.rs`
- Test: `src-tauri/crates/uc-core/src/network/protocol.rs` (existing tests)

**Step 1: Write failing tests**

```rust
#[test]
fn pairing_message_session_id_handles_cancel_and_reject() {
    let reject = PairingMessage::Reject(PairingReject {
        session_id: "s1".to_string(),
        reason: Some("user".to_string()),
    });
    let cancel = PairingMessage::Cancel(PairingCancel {
        session_id: "s2".to_string(),
        reason: Some("timeout".to_string()),
    });
    let busy = PairingMessage::Busy(PairingBusy {
        session_id: "s3".to_string(),
        reason: Some("occupied".to_string()),
    });

    assert_eq!(reject.session_id(), "s1");
    assert_eq!(cancel.session_id(), "s2");
    assert_eq!(busy.session_id(), "s3");
}
```

**Step 2: Run test to verify it fails**

Run (from `src-tauri/`):

```bash
cargo test -p uc-core pairing_message_session_id_handles_cancel_and_reject
```

Expected: FAIL (enum variants missing).

**Step 3: Write minimal implementation**

- Add `PairingMessage::Reject/Cancel/Busy` and structs.
- Update `session_id()` and Debug redactions.

**Step 4: Run test to verify it passes**

```bash
cargo test -p uc-core pairing_message_session_id_handles_cancel_and_reject
```

Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-core/src/network/protocol.rs
git commit -m "feat: add pairing cancel/reject/busy messages"
```

### Task 2: Rewrite Pairing State Machine (uc-core)

**Files:**

- Modify: `src-tauri/crates/uc-core/src/network/pairing_state_machine.rs`
- Test: `src-tauri/crates/uc-core/src/network/pairing_state_machine.rs`

**Step 1: Write failing tests for new states**

```rust
#[test]
fn initiator_flow_transitions_to_request_sent() {
    let mut sm = PairingStateMachine::new_with_local_identity(
        "Local".to_string(),
        "device-1".to_string(),
        vec![1; 32],
    );

    let (state, actions) = sm.handle_event(
        PairingEvent::StartPairing {
            role: PairingRole::Initiator,
            peer_id: "peer-2".to_string(),
        },
        Utc::now(),
    );

    assert!(matches!(state, PairingState::RequestSent { .. }));
    assert!(actions.iter().any(|action| matches!(action, PairingAction::Send { .. })));
}
```

**Step 2: Run test to verify it fails**

```bash
cargo test -p uc-core initiator_flow_transitions_to_request_sent
```

Expected: FAIL (state machine still uses Starting/WaitingForRequest).

**Step 3: Write minimal implementation**

- Replace old states with new: `RequestSent`, `AwaitingUserConfirm`, `ResponseSent`, `AwaitingUserApproval`, `ChallengeSent`, `Finalizing`, plus terminal states.
- Align events to spec: `RecvReject`, `RecvCancel`, `RecvBusy` should transition to Cancelled/Failed as per spec.
- Generate actions for sending messages and starting/canceling timers for each state.
- Add `TimeoutKind::UserApproval` and `TimeoutKind::Persist` (or equivalent).
- Ensure `PairingAction::EmitResult` remains for terminal states.

**Step 4: Run state machine test suite**

```bash
cargo test -p uc-core pairing_state_machine
```

Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-core/src/network/pairing_state_machine.rs
git commit -m "refactor: align pairing state machine with dual-role flow"
```

### Task 3: Update Pairing Orchestrator (uc-app)

**Files:**

- Modify: `src-tauri/crates/uc-app/src/usecases/pairing/orchestrator.rs`
- Test: `src-tauri/crates/uc-app/src/usecases/pairing/orchestrator.rs`

**Step 1: Write failing test for StartPairing send action**

```rust
#[tokio::test]
async fn initiate_pairing_emits_request_action() {
    let config = PairingConfig::default();
    let device_repo = Arc::new(MockDeviceRepository);
    let (orchestrator, mut action_rx) = PairingOrchestrator::new(
        config,
        device_repo,
        "Local".to_string(),
        "device-1".to_string(),
        "peer-local".to_string(),
        vec![1; 32],
    );

    let _session_id = orchestrator
        .initiate_pairing("peer-remote".to_string())
        .await
        .expect("initiate pairing");

    let action = tokio::time::timeout(Duration::from_secs(1), action_rx.recv())
        .await
        .expect("action timeout")
        .expect("action missing");

    assert!(matches!(action, PairingAction::Send { message: PairingMessage::Request(_), .. }));
}
```

**Step 2: Run test to verify it fails**

```bash
cargo test -p uc-app initiate_pairing_emits_request_action
```

Expected: FAIL (orchestrator sends request directly, not via actions).

**Step 3: Write minimal implementation**

- Use state machine actions for `StartPairing` and execute them instead of manual request send.
- Add handlers for `PairingMessage::Reject/Cancel/Busy` mapping to events.
- Ensure timers align with new `TimeoutKind` values.

**Step 4: Run orchestrator tests**

```bash
cargo test -p uc-app pairing::orchestrator
```

Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/pairing/orchestrator.rs
git commit -m "refactor: route pairing actions through new state machine"
```

### Task 4: Rewrite libp2p Adapter Pairing Flow (uc-platform)

**Files:**

- Modify: `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs`
- Test: `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs`

**Step 1: Write failing tests for new message handling**

```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pairing_reject_uses_stored_channel() {
    // Similar to pairing_response_uses_stored_channel but with Reject message
}
```

**Step 2: Run test to verify it fails**

```bash
cargo test -p uc-platform pairing_reject_uses_stored_channel
```

Expected: FAIL (message variants missing/adapter not handling).

**Step 3: Write minimal implementation**

- Update pairing request_response handling to route new message variants.
- Keep response-channel mapping by `session_id` and always respond when channel exists.
- Update adapter tests for new message types and ensure request/response still works.

**Step 4: Run adapter tests**

```bash
cargo test -p uc-platform libp2p_network
```

Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs
git commit -m "refactor: align libp2p pairing adapter with new protocol"
```

### Task 5: Full Verification

**Files:**

- Test: `src-tauri/crates/uc-core/src/network/pairing_state_machine.rs`
- Test: `src-tauri/crates/uc-app/src/usecases/pairing/orchestrator.rs`
- Test: `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs`

**Step 1: Run core/app/platform tests**

```bash
cargo test -p uc-core pairing_state_machine
cargo test -p uc-app pairing::orchestrator
cargo test -p uc-platform libp2p_network
```

**Step 2: Document results**

- Update `progress.md` with test outcomes.

**Step 3: Commit (if needed)**

```bash
git add progress.md
git commit -m "test: verify pairing flow updates"
```
