# Setup State Machine Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add the onboarding setup state machine (domain + app orchestrator) and Tauri commands, with full support for creating a new encrypted space. Join-existing-space flow is TODO for now.

**Architecture:** Implement a pure state machine in `uc-core` and an orchestrator in `uc-app` that executes side-effects via ports. For now, only the create-space branch is executed; join-space actions are explicit TODOs. Expose commands in `uc-tauri` to drive the state machine from the frontend.

**Tech Stack:** Rust (uc-core/uc-app/uc-tauri), Tauri commands, tokio, tracing, serde.

---

### Task 1: Add core state machine types

**Files:**

- Create: `src-tauri/crates/uc-core/src/setup/state_machine.rs`
- Modify: `src-tauri/crates/uc-core/src/lib.rs`
- Modify: `src-tauri/crates/uc-core/src/setup/mod.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn welcome_choose_create_transitions_to_create_passphrase() {
    let state = SetupState::Welcome;
    let (next, actions) = SetupStateMachine::transition(state, SetupEvent::ChooseCreateSpace);
    assert_eq!(next, SetupState::CreateSpacePassphrase { error: None });
    assert!(actions.is_empty());
}
```

**Step 2: Run test to verify it fails**

Run (from `src-tauri/`): `cargo test -p uc-core setup_state_machine -- --nocapture`
Expected: FAIL with missing module/types.

**Step 3: Write minimal implementation**

```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SetupState { /* ... */ }

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SetupEvent { /* ... */ }

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SetupAction { /* ... */ }

pub struct SetupStateMachine;

impl SetupStateMachine {
    pub fn transition(state: SetupState, event: SetupEvent) -> (SetupState, Vec<SetupAction>) {
        // pure mapping
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p uc-core setup_state_machine -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-core/src/setup/state_machine.rs \
  src-tauri/crates/uc-core/src/setup/mod.rs \
  src-tauri/crates/uc-core/src/lib.rs

git commit -m "feat(core): add setup state machine"
```

---

### Task 2: Add setup errors + action types

**Files:**

- Modify: `src-tauri/crates/uc-core/src/setup/state_machine.rs`
- Modify: `src-tauri/crates/uc-core/src/setup/mod.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn create_passphrase_mismatch_sets_error() {
    let state = SetupState::CreateSpacePassphrase { error: None };
    let event = SetupEvent::SubmitCreatePassphrase { pass1: "a".into(), pass2: "b".into() };
    let (next, actions) = SetupStateMachine::transition(state, event);
    assert_eq!(next, SetupState::CreateSpacePassphrase { error: Some(SetupError::PassphraseMismatch) });
    assert!(actions.is_empty());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p uc-core setup_state_machine -- --nocapture`
Expected: FAIL with missing error enums.

**Step 3: Write minimal implementation**

```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SetupError {
    PassphraseMismatch,
    PassphraseTooShort { min_len: usize },
    PassphraseEmpty,
    PassphraseInvalidOrMismatch,
    NetworkTimeout,
    PeerUnavailable,
    PairingRejected,
    PairingFailed,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SetupAction {
    CreateEncryptedSpace { passphrase: String },
    ScanPeers,
    VerifyPassphraseWithPeer { peer_id: String, passphrase: String },
    StartPairing { peer_id: String },
    ConfirmPairing { session_id: String },
    CancelPairing { session_id: String },
    MarkSetupComplete,
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p uc-core setup_state_machine -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-core/src/setup/state_machine.rs \
  src-tauri/crates/uc-core/src/setup/mod.rs

git commit -m "feat(core): add setup errors and actions"
```

---

### Task 3: Implement setup orchestrator in uc-app

**Files:**

- Create: `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs`
- Create: `src-tauri/crates/uc-app/src/usecases/setup/mod.rs`
- Modify: `src-tauri/crates/uc-app/src/usecases/mod.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn orchestrator_drives_state_machine_and_executes_actions() {
    // Arrange a fake port set and initial state
    // Dispatch ChooseCreateSpace then SubmitCreatePassphrase
    // Assert CreateEncryptedSpace called and state == Done
}
```

**Step 2: Run test to verify it fails**

Run (from `src-tauri/`): `cargo test -p uc-app setup_orchestrator -- --nocapture`
Expected: FAIL with missing module.

**Step 3: Write minimal implementation**

- Orchestrator keeps `SetupState` in memory and exposes:
  - `get_state()`
  - `dispatch(event)` → returns `SetupState`
- Executes `SetupAction` by calling use cases/ports:
  - `CreateEncryptedSpace`: call `InitializeEncryption` use case
  - `ScanPeers` / `VerifyPassphraseWithPeer` / `StartPairing` / `ConfirmPairing` / `CancelPairing`: TODO (return explicit not-implemented error + log; do not change state)
  - `MarkSetupComplete`: call `CompleteOnboarding`
- No `unwrap/expect`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p uc-app setup_orchestrator -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs \
  src-tauri/crates/uc-app/src/usecases/setup/mod.rs \
  src-tauri/crates/uc-app/src/usecases/mod.rs

git commit -m "feat(app): add setup orchestrator"
```

---

### Task 4: Wire dependencies + expose Tauri commands

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`
- Create: `src-tauri/crates/uc-tauri/src/commands/setup.rs`
- Modify: `src-tauri/crates/uc-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/main.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn setup_commands_are_registered() {
    // Ensure commands module exposes setup commands
}
```

**Step 2: Run test to verify it fails**

Run (from `src-tauri/`): `cargo test -p uc-tauri setup_commands -- --nocapture`
Expected: FAIL with missing module/registration.

**Step 3: Write minimal implementation**

- Add `SetupOrchestrator` accessor to runtime usecases (in-memory state).
- Tauri commands:
  - `get_setup_state`
  - `dispatch_setup_event`
  - Ensure `_trace: Option<TraceMetadata>` + `info_span!` + `.instrument(span)`

**Step 4: Run test to verify it passes**

Run: `cargo test -p uc-tauri setup_commands -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs \
  src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs \
  src-tauri/crates/uc-tauri/src/commands/setup.rs \
  src-tauri/crates/uc-tauri/src/commands/mod.rs \
  src-tauri/src/main.rs

git commit -m "feat(tauri): wire setup state machine commands"
```

---

### Task 5: Update frontend API stubs for setup commands

**Files:**

- Modify: `src/api/onboarding.ts`

**Step 1: Write the failing test**

```ts
it('dispatchSetupEvent calls new tauri command', async () => {
  // mock invokeWithTrace and verify payload
})
```

**Step 2: Run test to verify it fails**

Run: `npm test -- onboarding`
Expected: FAIL with missing functions.

**Step 3: Write minimal implementation**

Add:

- `getSetupState()`
- `dispatchSetupEvent(event)`
- DTOs for `SetupState`/`SetupEvent`

**Step 4: Run test to verify it passes**

Run: `npm test -- onboarding`
Expected: PASS

**Step 5: Commit**

```bash
git add src/api/onboarding.ts

git commit -m "feat(frontend): add setup state machine API"
```

---

### Task 6: Add integration tests for setup flow

**Files:**

- Create: `src-tauri/crates/uc-app/tests/setup_flow_test.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn create_space_flow_marks_setup_complete() {
    // Dispatch events: ChooseCreateSpace -> SubmitCreatePassphrase
    // Assert Done + MarkSetupComplete called
}
```

**Step 2: Run test to verify it fails**

Run (from `src-tauri/`): `cargo test -p uc-app setup_flow_test -- --nocapture`
Expected: FAIL with missing test infra.

**Step 3: Write minimal implementation**

Use mock ports to verify side effects and errors are mapped correctly.

**Step 4: Run test to verify it passes**

Run: `cargo test -p uc-app setup_flow_test -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-app/tests/setup_flow_test.rs

git commit -m "test(app): add setup flow integration tests"
```

---

### Task 7: Update docs for setup flow

**Files:**

- Modify: `docs/plans/2025-01-15-onboarding-implementation-plan.md`

**Step 1: Write the failing test**

No automated test. This is a docs-only task.

**Step 2: Update docs**

Document the new state machine, ports, and commands.

**Step 3: Commit**

```bash
git add docs/plans/2025-01-15-onboarding-implementation-plan.md

git commit -m "docs: document setup state machine"
```
