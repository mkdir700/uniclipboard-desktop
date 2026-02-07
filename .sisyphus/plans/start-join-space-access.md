# Wire StartJoinSpaceAccess End-to-End

## TL;DR

> **Quick Summary**: Replace the `StartJoinSpaceAccess` placeholder in `SetupOrchestrator` with a real call to `SpaceAccessOrchestrator`, implementing all missing production adapters (TransportPort, ProofPort, PersistencePort) and fixing the incomplete `derive_master_key_from_keyslot()` in CryptoAdapter.
>
> **Deliverables**:
>
> - Production `SpaceAccessTransportPort` adapter (fix and integrate `network_adapter.rs`)
> - Production `ProofPort` adapter (HMAC-based proof build/verify)
> - Production `PersistencePort` adapter (space access persistence)
> - Complete `SpaceAccessCryptoAdapter.derive_master_key_from_keyslot()` for joiner side
> - `SetupOrchestrator` wired to construct `SpaceAccessExecutor` at runtime
> - Updated `SetupRuntimePorts` in bootstrap for new port dependencies
> - Integration tests for the full join flow
>
> **Estimated Effort**: Medium
> **Parallel Execution**: YES - 2 waves
> **Critical Path**: Task 1 → Task 4 → Task 5 → Task 6 → Task 7

---

## Context

### Original Request

User identified that `StartJoinSpaceAccess` in `SetupOrchestrator` is a placeholder returning `LifecycleFailed`. It needs to be wired to actually invoke the `SpaceAccessOrchestrator` join flow, which requires runtime construction of a `SpaceAccessExecutor` with 6 port dependencies — 3 of which have no production adapters yet.

### Interview Summary

**Key Discussions**:

- **Scope**: Complete end-to-end — all missing adapters must be implemented, not stubs
- **DI Pattern**: Add `Arc<T>` port fields directly to `SetupOrchestrator` (not a factory trait)
- **Test Strategy**: Tests-after (not TDD), integration tests using mock adapters in uc-app

**Research Findings**:

- `SpaceAccessExecutor` uses `&'a mut dyn Trait` borrows — must construct at call site, not pre-cache
- `SpaceAccessTransportPort` trait methods return `()` but orphaned `network_adapter.rs` returns `Result<()>` — signature mismatch. The trait needs to return `anyhow::Result<()>` for proper error propagation
- `SpaceAccessCryptoAdapter.derive_master_key_from_keyslot()` returns `Err("not implemented")` — must be filled for joiner side
- `InitializeNewSpace` use case shows the pattern for constructing `SpaceAccessExecutor` with real ports (uses `Arc<Mutex<dyn Port>>`)
- `network_adapter.rs` references non-existent `ProtocolMessage`, `SpaceAccessCodec` types — needs rewrite
- `PersistencePort` needs `persist_joiner_access` and `persist_sponsor_access` — these map to storing encryption state and device authorization

### Metis Review

**Identified Gaps** (addressed):

- `derive_master_key_from_keyslot()` stub — added as dedicated task (Task 2)
- `SpaceAccessTransportPort` trait returns `()` but real network ops can fail — Task 1 addresses trait signature fix
- No production `ProofPort` adapter — Task 3 creates one
- No production `PersistencePort` adapter — Task 3 creates one
- Bootstrap wiring for new ports not planned — Task 5 addresses this
- Metis asked about crypto key derivation source for joiner — resolved: joiner receives keyslot blob from sponsor via transport, derives master key using passphrase

---

## Work Objectives

### Core Objective

Make `StartJoinSpaceAccess` in `SetupOrchestrator` invoke the `SpaceAccessOrchestrator` join flow with real, production-grade port adapters.

### Concrete Deliverables

- Working `start_join_space_access()` method in `SetupOrchestrator`
- Production `SpaceAccessTransportPort` adapter using `NetworkPort`
- Production `ProofPort` adapter using HMAC-SHA256
- Production `PersistencePort` adapter using existing encryption state/key material ports
- Complete `derive_master_key_from_keyslot()` implementation
- Updated bootstrap wiring in `SetupRuntimePorts`
- Integration tests for join space access flow

### Definition of Done

- [x] `cargo check --workspace` passes with no new warnings
- [x] `cargo test -p uc-app --lib` passes (all existing + new tests)
- [x] `SetupOrchestrator::StartJoinSpaceAccess` calls `SpaceAccessOrchestrator` instead of returning error
- [x] `network_adapter.rs` is integrated in `mod.rs` and compiles
- [x] `bun run build` passes

### Must Have

- All 3 missing port adapters implemented with real logic
- `derive_master_key_from_keyslot()` implemented for joiner crypto flow
- Error propagation with proper `tracing` logging at all boundaries
- Hexagonal architecture boundaries respected (ports in uc-core, adapters in uc-app/uc-tauri)

### Must NOT Have (Guardrails)

- ❌ Do NOT modify `uc-core` state machines (`state_machine.rs`, `event.rs`, `state.rs`, `action.rs`)
- ❌ Do NOT use `unwrap()` / `expect()` in production code
- ❌ Do NOT modify `PairingOrchestrator` internal protocol logic
- ❌ Do NOT add new Tauri commands (the join flow is driven by existing setup commands)
- ❌ Do NOT introduce new dependencies on `uc-infra` from `uc-app` (use ports)
- ❌ Do NOT touch `uc-core/src/ports/space/transport.rs` signature without updating ALL consumers
- ❌ Do NOT implement real network message encoding yet — use `NetworkPort.send_pairing_on_session()` with serialized bytes
- ❌ Do NOT add `is_encryption_initialized` calls in UI

---

## Verification Strategy

> **UNIVERSAL RULE: ZERO HUMAN INTERVENTION**
>
> ALL tasks are verifiable WITHOUT any human action.

### Test Decision

- **Infrastructure exists**: YES
- **Automated tests**: YES (Tests-after)
- **Framework**: cargo test (tokio-based)

### Agent-Executed QA Scenarios (MANDATORY — ALL tasks)

Every task includes build verification + targeted test scenarios.
All verification executed by agent using Bash commands.

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately — Independent adapter implementations):
├── Task 1: Fix SpaceAccessTransportPort trait + rewrite network_adapter.rs
├── Task 2: Implement derive_master_key_from_keyslot() in CryptoAdapter
└── Task 3: Implement ProofPort + PersistencePort production adapters

Wave 2 (After Wave 1 — Integration):
├── Task 4: Wire Arc ports into SetupOrchestrator + implement start_join_space_access()
├── Task 5: Update SetupRuntimePorts bootstrap wiring
├── Task 6: Integration tests for join space access flow
└── Task 7: Final verification + cleanup
```

### Dependency Matrix

| Task | Depends On | Blocks  | Can Parallelize With |
| ---- | ---------- | ------- | -------------------- |
| 1    | None       | 4, 5, 6 | 2, 3                 |
| 2    | None       | 4, 6    | 1, 3                 |
| 3    | None       | 4, 5, 6 | 1, 2                 |
| 4    | 1, 2, 3    | 5, 6    | None                 |
| 5    | 4          | 7       | 6                    |
| 6    | 4          | 7       | 5                    |
| 7    | 5, 6       | None    | None (final)         |

---

## TODOs

- [x] 1. Fix SpaceAccessTransportPort trait signature and rewrite network_adapter.rs

  **What to do**:
  - Update `uc-core/src/ports/space/transport.rs`: Change all methods to return `anyhow::Result<()>` instead of `()`. The methods `send_offer`, `send_proof`, `send_result` all perform network I/O which can fail.
  - Update ALL consumers of `SpaceAccessTransportPort` to handle `Result`:
    - `uc-app/src/usecases/space_access/orchestrator.rs` — the `execute_actions` match arms for `SendOffer`, `SendProof`, `SendResult` currently call `.await` without `?`. Add error propagation.
    - `uc-app/src/usecases/space_access/orchestrator.rs` tests — `MockTransport` implementation needs updated signatures
  - Rewrite `uc-app/src/usecases/space_access/network_adapter.rs`:
    - Remove references to non-existent `ProtocolMessage` and `SpaceAccessCodec` types
    - Implement `SpaceAccessTransportPort` using `NetworkPort.send_pairing_on_session()`
    - For now, serialize context data (offer/proof/result) to bytes using `serde_json` or simple byte encoding
    - The adapter needs access to `SpaceAccessContext` (shared via `Arc<Mutex<SpaceAccessContext>>`) and `NetworkPort`
  - Register `network_adapter` module in `uc-app/src/usecases/space_access/mod.rs`
  - Export `SpaceAccessNetworkAdapter` from `mod.rs`

  **Must NOT do**:
  - Do NOT change the trait name or add new methods
  - Do NOT introduce new dependencies to uc-core

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Requires understanding cross-crate trait consumers and careful signature migration
  - **Skills**: [`test-driven-development`]
    - `test-driven-development`: Ensures existing tests are updated alongside trait changes

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 2, 3)
  - **Blocks**: Tasks 4, 5, 6
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `src-tauri/crates/uc-app/src/usecases/space_access/network_adapter.rs` — Current orphaned adapter (structure to reference, but signatures need fixing)
  - `src-tauri/crates/uc-app/src/usecases/space_access/orchestrator.rs:124-201` — All match arms calling transport methods (need `?` after trait change)
  - `src-tauri/crates/uc-app/src/usecases/space_access/orchestrator.rs:767-785` — MockTransport impl (needs signature update)

  **API/Type References**:
  - `src-tauri/crates/uc-core/src/ports/space/transport.rs` — Trait definition to modify
  - `src-tauri/crates/uc-core/src/network/session.rs` — `SessionId` type used in transport methods

  **External References**:
  - `src-tauri/crates/uc-core/src/ports/space/mod.rs` — Port module exports

  **WHY Each Reference Matters**:
  - `network_adapter.rs` shows the intended adapter structure but needs signature alignment with updated trait
  - `orchestrator.rs:124-201` shows all call sites that MUST be updated when trait signature changes
  - `MockTransport` must be updated or tests will fail to compile

  **Acceptance Criteria**:
  - [ ] `uc-core/src/ports/space/transport.rs` methods return `anyhow::Result<()>`
  - [ ] `cargo check --workspace` passes (all consumers updated)
  - [ ] `network_adapter.rs` is registered in `mod.rs` and compiles
  - [ ] `SpaceAccessNetworkAdapter` is exported from space_access module
  - [ ] `cargo test -p uc-app --lib` passes (MockTransport updated, all existing tests green)

  **Agent-Executed QA Scenarios**:

  ```
  Scenario: Trait signature change compiles across workspace
    Tool: Bash
    Preconditions: None
    Steps:
      1. cd src-tauri && cargo check --workspace 2>&1
      2. Assert: exit code 0
      3. Assert: no errors related to SpaceAccessTransportPort
    Expected Result: Clean compilation
    Evidence: cargo check output captured

  Scenario: Existing space_access tests still pass
    Tool: Bash
    Preconditions: None
    Steps:
      1. cd src-tauri && cargo test -p uc-app --lib -- space_access 2>&1
      2. Assert: "test result: ok" in output
      3. Assert: 0 failed
    Expected Result: All space_access tests pass
    Evidence: test output captured
  ```

  **Commit**: YES
  - Message: `feat(core): update SpaceAccessTransportPort to return Result and integrate network adapter`
  - Files: `uc-core/src/ports/space/transport.rs`, `uc-app/src/usecases/space_access/network_adapter.rs`, `uc-app/src/usecases/space_access/mod.rs`, `uc-app/src/usecases/space_access/orchestrator.rs`
  - Pre-commit: `cd src-tauri && cargo test -p uc-app --lib`

---

- [x] 2. Implement derive_master_key_from_keyslot() in SpaceAccessCryptoAdapter

  **What to do**:
  - In `src-tauri/crates/uc-app/src/usecases/space_access/crypto_adapter.rs`, implement `derive_master_key_from_keyslot()`:
    1. Deserialize the keyslot blob bytes into a `KeySlot` struct
    2. Derive KEK from passphrase + keyslot salt using `self.encryption.derive_kek()`
    3. Unwrap master key using `self.encryption.unwrap_master_key(kek, keyslot.wrapped_master_key.blob)`
    4. Store KEK and keyslot via `self.key_material`
    5. Set master key in `self.encryption_session`
    6. Mark encryption as initialized via `self.encryption_state.persist_initialized()`
    7. Return the derived `MasterKey`
  - Add proper error handling with tracing spans and structured fields
  - Add rollback logic matching the pattern in `export_keyslot_blob()` (if store_kek succeeds but later step fails, rollback)
  - Add unit test for successful derivation and for failure rollback

  **Must NOT do**:
  - Do NOT change `CryptoPort` trait definition
  - Do NOT modify `export_keyslot_blob()` logic

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Crypto key derivation requires careful error handling and rollback logic
  - **Skills**: [`test-driven-development`]
    - `test-driven-development`: Critical for crypto code correctness

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 3)
  - **Blocks**: Tasks 4, 6
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `src-tauri/crates/uc-app/src/usecases/space_access/crypto_adapter.rs:109-189` — `export_keyslot_blob()` implementation — mirror this rollback pattern for the reverse operation
  - `src-tauri/crates/uc-app/src/usecases/space_access/crypto_adapter.rs:192-200` — Current stub to replace

  **API/Type References**:
  - `src-tauri/crates/uc-core/src/security/model.rs` — `KeySlot`, `MasterKey`, `Passphrase`, `Kek`, `WrappedMasterKey` types
  - `src-tauri/crates/uc-core/src/ports/space/crypto.rs` — `CryptoPort` trait definition
  - `src-tauri/crates/uc-core/src/ports/mod.rs` — `EncryptionPort`, `KeyMaterialPort`, `EncryptionSessionPort`

  **Test References**:
  - `src-tauri/crates/uc-app/src/usecases/space_access/crypto_adapter.rs:377-395` — Existing rollback test pattern

  **WHY Each Reference Matters**:
  - `export_keyslot_blob()` is the sponsor-side mirror operation — joiner does the reverse (unwrap instead of wrap)
  - The rollback pattern must be replicated to prevent partial state corruption
  - `KeySlot` struct must be deserialized from blob bytes to extract salt and wrapped key

  **Acceptance Criteria**:
  - [ ] `derive_master_key_from_keyslot()` no longer returns `Err("not implemented")`
  - [ ] Successful derivation: given valid keyslot blob + passphrase, returns `MasterKey`
  - [ ] Rollback: if `persist_initialized` fails, KEK and keyslot are cleaned up
  - [ ] `cargo test -p uc-app --lib -- space_access::crypto` passes with new tests
  - [ ] `cargo check --workspace` passes

  **Agent-Executed QA Scenarios**:

  ```
  Scenario: Successful key derivation from keyslot blob
    Tool: Bash
    Preconditions: None
    Steps:
      1. cd src-tauri && cargo test -p uc-app --lib -- crypto_adapter::tests 2>&1
      2. Assert: test for successful derivation passes
      3. Assert: "test result: ok" in output
    Expected Result: Key derivation works end-to-end
    Evidence: test output captured

  Scenario: Rollback on failure during derivation
    Tool: Bash
    Preconditions: None
    Steps:
      1. cd src-tauri && cargo test -p uc-app --lib -- crypto_adapter::tests::rollback 2>&1
      2. Assert: rollback test passes, KEK deletion confirmed
    Expected Result: Clean rollback on partial failure
    Evidence: test output captured
  ```

  **Commit**: YES
  - Message: `feat(app): implement derive_master_key_from_keyslot for joiner crypto flow`
  - Files: `uc-app/src/usecases/space_access/crypto_adapter.rs`
  - Pre-commit: `cd src-tauri && cargo test -p uc-app --lib`

---

- [x] 3. Implement ProofPort and PersistencePort production adapters

  **What to do**:
  - **ProofPort adapter** — Create `src-tauri/crates/uc-app/src/usecases/space_access/proof_adapter.rs`:
    - Implement `build_proof()`: Generate HMAC-SHA256 over `(pairing_session_id, space_id, challenge_nonce)` using `master_key` as HMAC key. Return `SpaceAccessProofArtifact` containing the HMAC bytes and input parameters.
    - Implement `verify_proof()`: Recompute HMAC from artifact fields and compare with stored HMAC. Return `Ok(true)` if match.
    - Use `hmac` and `sha2` crates (check if already in dependencies, otherwise add to `uc-app/Cargo.toml`)
    - Name: `HmacProofAdapter`
  - **PersistencePort adapter** — Create `src-tauri/crates/uc-app/src/usecases/space_access/persistence_adapter.rs`:
    - Implement `persist_joiner_access()`: Mark encryption state as initialized (joiner now has access to space). Use `EncryptionStatePort` to persist.
    - Implement `persist_sponsor_access()`: Record that a peer has been granted access. Use `PairedDeviceRepositoryPort` to update the paired device record.
    - Adapter needs `Arc<dyn EncryptionStatePort>` and `Arc<dyn PairedDeviceRepositoryPort>`
    - Name: `SpaceAccessPersistenceAdapter`
  - Register both modules in `mod.rs` and export adapter types

  **Must NOT do**:
  - Do NOT modify the port trait definitions in `uc-core`
  - Do NOT introduce direct database dependencies — use existing ports

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Crypto proof construction requires careful HMAC implementation
  - **Skills**: [`test-driven-development`]
    - `test-driven-development`: Crypto and persistence correctness needs test coverage

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 2)
  - **Blocks**: Tasks 4, 5, 6
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `src-tauri/crates/uc-app/src/usecases/space_access/crypto_adapter.rs` — Adapter pattern: struct holds Arc ports, implements uc-core trait
  - `src-tauri/crates/uc-app/src/usecases/space_access/orchestrator.rs:720-740` — MockProof impl shows expected method signatures
  - `src-tauri/crates/uc-app/src/usecases/space_access/orchestrator.rs:807-825` — MockStore impl shows expected method signatures

  **API/Type References**:
  - `src-tauri/crates/uc-core/src/ports/space/proof.rs` — ProofPort trait (build_proof, verify_proof)
  - `src-tauri/crates/uc-core/src/ports/space/persistence.rs` — PersistencePort trait (persist_joiner_access, persist_sponsor_access)
  - `src-tauri/crates/uc-core/src/security/space_access/mod.rs` — `SpaceAccessProofArtifact` type
  - `src-tauri/crates/uc-core/src/ports/security/encryption_state.rs` — `EncryptionStatePort`
  - `src-tauri/crates/uc-core/src/ports/mod.rs` — `PairedDeviceRepositoryPort`

  **WHY Each Reference Matters**:
  - `crypto_adapter.rs` is the model adapter pattern to follow (struct with Arc ports + trait impl)
  - MockProof/MockStore show the exact method shapes the production adapter must match
  - `SpaceAccessProofArtifact` type determines what data build_proof must return

  **Acceptance Criteria**:
  - [ ] `proof_adapter.rs` compiles and is registered in `mod.rs`
  - [ ] `persistence_adapter.rs` compiles and is registered in `mod.rs`
  - [ ] `HmacProofAdapter.build_proof()` produces valid HMAC
  - [ ] `HmacProofAdapter.verify_proof()` returns `true` for valid proof, `false` for tampered
  - [ ] `SpaceAccessPersistenceAdapter.persist_joiner_access()` calls `EncryptionStatePort.persist_initialized()`
  - [ ] `cargo check --workspace` passes
  - [ ] Unit tests for proof build/verify round-trip

  **Agent-Executed QA Scenarios**:

  ```
  Scenario: Proof build and verify round-trip
    Tool: Bash
    Preconditions: None
    Steps:
      1. cd src-tauri && cargo test -p uc-app --lib -- proof_adapter::tests 2>&1
      2. Assert: round-trip test passes
      3. Assert: tamper detection test passes
    Expected Result: HMAC proof is correctly constructed and verified
    Evidence: test output captured

  Scenario: Persistence adapter compiles with port dependencies
    Tool: Bash
    Preconditions: None
    Steps:
      1. cd src-tauri && cargo check -p uc-app 2>&1
      2. Assert: exit code 0
      3. Assert: no errors in persistence_adapter
    Expected Result: Clean compilation
    Evidence: cargo check output
  ```

  **Commit**: YES
  - Message: `feat(app): add HmacProofAdapter and SpaceAccessPersistenceAdapter`
  - Files: `uc-app/src/usecases/space_access/proof_adapter.rs`, `uc-app/src/usecases/space_access/persistence_adapter.rs`, `uc-app/src/usecases/space_access/mod.rs`, `uc-app/Cargo.toml` (if hmac/sha2 needed)
  - Pre-commit: `cd src-tauri && cargo test -p uc-app --lib`

---

- [x] 4. Wire Arc ports into SetupOrchestrator and implement start_join_space_access()

  **What to do**:
  - Add new fields to `SetupOrchestrator` struct in `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs`:
    ```rust
    // Ports for constructing SpaceAccessExecutor at runtime
    crypto_factory: Arc<dyn SpaceAccessCryptoFactory>,
    network_port: Arc<dyn NetworkPort>,
    transport_port: Arc<Mutex<dyn SpaceAccessTransportPort>>,
    proof_port: Arc<dyn ProofPort>,
    timer_port: Arc<Mutex<dyn TimerPort>>,
    persistence_port: Arc<Mutex<dyn PersistencePort>>,
    ```
  - Update `SetupOrchestrator::new()` constructor to accept these new parameters
  - Implement `start_join_space_access()` private method:
    1. Get passphrase from `self.passphrase` (stored during `SubmitPassphrase` event)
    2. Build `CryptoPort` via `self.crypto_factory.build(passphrase)`
    3. Get `pairing_session_id` from `self.pairing_session_id`
    4. Construct `SpaceAccessExecutor` with borrowed ports
    5. Call `self.space_access_orchestrator.dispatch(executor, event, session_id)`
    6. Handle result, map errors to `SetupError`
  - Replace the placeholder in `execute_actions` match arm for `StartJoinSpaceAccess`
  - Update all test helpers that construct `SetupOrchestrator` to pass the new parameters (using mock/noop ports)

  **Must NOT do**:
  - Do NOT change the state machine or event definitions
  - Do NOT add new public methods to SetupOrchestrator beyond what's needed

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Core integration point connecting setup flow to space access flow
  - **Skills**: [`test-driven-development`]
    - `test-driven-development`: Integration wiring needs careful test updates

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (Wave 2, first)
  - **Blocks**: Tasks 5, 6
  - **Blocked By**: Tasks 1, 2, 3

  **References**:

  **Pattern References**:
  - `src-tauri/crates/uc-app/src/usecases/space_access/initialize_new_space.rs:57-85` — Shows how to construct `SpaceAccessExecutor` and call `orchestrator.initialize_new_space()` — follow this pattern for join flow
  - `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs:42-84` — Current struct definition and constructor
  - `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs:205-210` — Placeholder to replace

  **API/Type References**:
  - `src-tauri/crates/uc-app/src/usecases/space_access/executor.rs` — `SpaceAccessExecutor` struct (6 fields, all borrows)
  - `src-tauri/crates/uc-app/src/usecases/space_access/orchestrator.rs:78-98` — `dispatch()` method signature
  - `src-tauri/crates/uc-app/src/usecases/space_access/context.rs` — `SpaceAccessContext` and `SpaceAccessJoinerOffer`

  **Test References**:
  - `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs:860-920` — Existing test helpers for constructing SetupOrchestrator with mocks
  - `src-tauri/crates/uc-app/tests/setup_flow_integration_test.rs:200-250` — Integration test helpers

  **WHY Each Reference Matters**:
  - `initialize_new_space.rs` is the EXACT pattern to follow — it constructs `SpaceAccessExecutor` from Arc ports
  - The placeholder code at line 205-210 is what gets replaced
  - Test helpers must be updated to match new constructor signature

  **Acceptance Criteria**:
  - [ ] `SetupOrchestrator::new()` accepts 6 additional port parameters
  - [ ] `StartJoinSpaceAccess` action calls `space_access_orchestrator.dispatch()` instead of returning error
  - [ ] Passphrase and session_id are correctly extracted from stored state
  - [ ] `SpaceAccessExecutor` is constructed at call site with borrowed ports
  - [ ] All existing tests compile and pass with updated constructor
  - [ ] `cargo check --workspace` passes
  - [ ] `cargo test -p uc-app --lib` passes

  **Agent-Executed QA Scenarios**:

  ```
  Scenario: StartJoinSpaceAccess no longer returns LifecycleFailed
    Tool: Bash
    Preconditions: None
    Steps:
      1. cd src-tauri && grep -n "StartJoinSpaceAccess invoked without" crates/uc-app/src/usecases/setup/orchestrator.rs
      2. Assert: no matches found (placeholder removed)
    Expected Result: Placeholder log message is gone
    Evidence: grep output

  Scenario: Full workspace compiles
    Tool: Bash
    Preconditions: Tasks 1-3 complete
    Steps:
      1. cd src-tauri && cargo check --workspace 2>&1
      2. Assert: exit code 0
    Expected Result: Clean compilation with new wiring
    Evidence: cargo check output

  Scenario: All existing tests pass with new constructor
    Tool: Bash
    Preconditions: None
    Steps:
      1. cd src-tauri && cargo test -p uc-app --lib 2>&1
      2. Assert: "test result: ok" in output
      3. Assert: 0 failed
    Expected Result: No regression
    Evidence: test output
  ```

  **Commit**: YES
  - Message: `feat(app): wire StartJoinSpaceAccess to SpaceAccessOrchestrator with runtime executor`
  - Files: `uc-app/src/usecases/setup/orchestrator.rs`, `uc-app/tests/setup_flow_integration_test.rs`
  - Pre-commit: `cd src-tauri && cargo test -p uc-app --lib`

---

- [x] 5. Update SetupRuntimePorts bootstrap wiring

  **What to do**:
  - In `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`:
    - Add new fields to `SetupRuntimePorts`: `crypto_factory`, `transport_port`, `proof_port`, `timer_port`, `persistence_port`
    - Update `SetupRuntimePorts::new()` and `from_network()` constructors
    - Update `placeholder()` to create noop/empty versions of new ports
    - Update `build_setup_orchestrator()` to pass new ports to `SetupOrchestrator::new()`
  - In `src-tauri/src/main.rs`:
    - If `SetupRuntimePorts::from_network()` is called in main, update the call site
  - Create production instances of new adapters using `AppDeps` ports:
    - `DefaultSpaceAccessCryptoFactory` using encryption/key_material/key_scope/encryption_state/encryption_session
    - `SpaceAccessNetworkAdapter` wrapper using `AppDeps.network`
    - `HmacProofAdapter` (no external deps needed)
    - `SpaceAccessPersistenceAdapter` using encryption_state + paired_device_repo
    - For `TimerPort` — use `uc_infra::Timer` or create a new instance

  **Must NOT do**:
  - Do NOT add business logic to bootstrap — only wiring
  - Do NOT create real Timer instances without understanding the existing timer lifecycle

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Mostly mechanical wiring — passing ports from AppDeps to constructors
  - **Skills**: [`git-master`]
    - `git-master`: Clean atomic commit for bootstrap changes

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 6)
  - **Parallel Group**: Wave 2 (with Task 6)
  - **Blocks**: Task 7
  - **Blocked By**: Task 4

  **References**:

  **Pattern References**:
  - `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs:97-137` — Current `SetupRuntimePorts` definition to extend
  - `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs:195-243` — `build_setup_orchestrator()` that creates and wires SetupOrchestrator
  - `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs:245-255` — `placeholder_pairing_orchestrator()` pattern for placeholder creation

  **API/Type References**:
  - `src-tauri/crates/uc-app/src/usecases/space_access/crypto_adapter.rs:62-86` — `DefaultSpaceAccessCryptoFactory` constructor
  - `src-tauri/crates/uc-app/src/usecases/space_access/mod.rs` — All exported adapter types

  **WHY Each Reference Matters**:
  - `runtime.rs:97-137` is the struct to modify — add new port fields
  - `build_setup_orchestrator()` is where the new ports flow into `SetupOrchestrator::new()`
  - `DefaultSpaceAccessCryptoFactory` already exists and just needs to be instantiated in bootstrap

  **Acceptance Criteria**:
  - [ ] `SetupRuntimePorts` has fields for all SpaceAccess ports
  - [ ] `build_setup_orchestrator()` passes all ports to `SetupOrchestrator::new()`
  - [ ] `placeholder()` works with noop ports (no crash on default construction)
  - [ ] `cargo check --workspace` passes
  - [ ] `bun run build` passes (frontend not broken)

  **Agent-Executed QA Scenarios**:

  ```
  Scenario: Full workspace compiles with updated bootstrap
    Tool: Bash
    Preconditions: Task 4 complete
    Steps:
      1. cd src-tauri && cargo check --workspace 2>&1
      2. Assert: exit code 0
      3. Assert: no new warnings related to bootstrap
    Expected Result: Clean compilation
    Evidence: cargo check output

  Scenario: Frontend still builds
    Tool: Bash
    Preconditions: None
    Steps:
      1. bun run build 2>&1
      2. Assert: "built in" in output
    Expected Result: Frontend build succeeds
    Evidence: bun build output
  ```

  **Commit**: YES
  - Message: `feat(bootstrap): wire SpaceAccess port adapters into SetupRuntimePorts`
  - Files: `uc-tauri/src/bootstrap/runtime.rs`, `src/main.rs` (if needed)
  - Pre-commit: `cd src-tauri && cargo check --workspace`

---

- [x] 6. Integration tests for join space access flow

  **What to do**:
  - In `src-tauri/crates/uc-app/tests/setup_flow_integration_test.rs`:
    - Update `build_space_access_orchestrator()` helper if needed
    - Add test: `join_space_access_invokes_space_access_orchestrator` — verify that dispatching `StartJoinSpaceAccess` through SetupOrchestrator actually calls into SpaceAccessOrchestrator (not returning error)
    - Add test: `join_space_access_propagates_space_access_error` — verify error propagation
  - In `src-tauri/crates/uc-app/src/usecases/space_access/orchestrator.rs` tests:
    - Add test: `joiner_side_key_derivation` — verify the joiner flow through key derivation using mock crypto
  - Ensure all tests use mock ports (no real network/crypto)

  **Must NOT do**:
  - Do NOT require real network connections
  - Do NOT use real crypto keys — mock everything

  **Recommended Agent Profile**:
  - **Category**: `unspecified-low`
    - Reason: Test writing with established patterns
  - **Skills**: [`test-driven-development`]
    - `test-driven-development`: Test-focused task

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 5)
  - **Parallel Group**: Wave 2 (with Task 5)
  - **Blocks**: Task 7
  - **Blocked By**: Task 4

  **References**:

  **Pattern References**:
  - `src-tauri/crates/uc-app/tests/setup_flow_integration_test.rs` — Existing integration test structure
  - `src-tauri/crates/uc-app/src/usecases/space_access/orchestrator.rs:232-830` — Existing space_access test patterns with mock ports
  - `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs:830-920` — Existing setup test helpers

  **WHY Each Reference Matters**:
  - Integration test file already has helpers for constructing mock orchestrators — extend them
  - Space access tests show the mock pattern for all 6 ports

  **Acceptance Criteria**:
  - [ ] At least 2 new integration tests for join flow
  - [ ] Tests verify `StartJoinSpaceAccess` action invokes SpaceAccessOrchestrator
  - [ ] Tests verify error propagation from SpaceAccess to Setup
  - [ ] `cargo test -p uc-app` passes (all tests including new ones)

  **Agent-Executed QA Scenarios**:

  ```
  Scenario: New join flow tests pass
    Tool: Bash
    Preconditions: Tasks 1-4 complete
    Steps:
      1. cd src-tauri && cargo test -p uc-app -- join_space_access 2>&1
      2. Assert: "test result: ok" in output
      3. Assert: at least 2 tests run
    Expected Result: Join flow integration tests pass
    Evidence: test output

  Scenario: Full test suite regression check
    Tool: Bash
    Preconditions: None
    Steps:
      1. cd src-tauri && cargo test -p uc-app --lib 2>&1
      2. Assert: 0 failed
      3. Assert: test count >= 99 (97 existing + 2 new minimum)
    Expected Result: No regression, new tests added
    Evidence: test output
  ```

  **Commit**: YES
  - Message: `test(app): add join space access integration tests`
  - Files: `uc-app/tests/setup_flow_integration_test.rs`, `uc-app/src/usecases/space_access/orchestrator.rs`
  - Pre-commit: `cd src-tauri && cargo test -p uc-app`

---

- [x] 7. Final verification and cleanup

  **What to do**:
  - Run full workspace verification:
    - `cargo check --workspace`
    - `cargo test -p uc-app --lib`
    - `cargo test -p uc-app --test setup_flow_integration_test`
    - `bun run build`
  - Remove `#[allow(dead_code)]` from `space_access_orchestrator` field in SetupOrchestrator if still present
  - Verify no `unwrap()`/`expect()` in new production code
  - Check for any remaining `TODO` or `unimplemented!()` markers in new code
  - Update `.sisyphus/boulder.json` if needed

  **Must NOT do**:
  - Do NOT make functional changes — only cleanup and verification

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Verification and minor cleanup only
  - **Skills**: [`verification-before-completion`]
    - `verification-before-completion`: Final check before declaring done

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (final)
  - **Blocks**: None (final task)
  - **Blocked By**: Tasks 5, 6

  **References**:

  **Pattern References**:
  - AGENTS.md — Commit rules and verification commands

  **Acceptance Criteria**:
  - [ ] `cargo check --workspace` — 0 new warnings
  - [ ] `cargo test -p uc-app --lib` — all pass
  - [ ] `cargo test -p uc-app --test setup_flow_integration_test` — all pass
  - [ ] `bun run build` — succeeds
  - [ ] No `unwrap()`/`expect()` in new production code
  - [ ] No remaining `TODO` or `unimplemented!()` in new code
  - [ ] `dead_code` warning for `space_access_orchestrator` is gone

  **Agent-Executed QA Scenarios**:

  ```
  Scenario: Complete verification sweep
    Tool: Bash
    Preconditions: All prior tasks complete
    Steps:
      1. cd src-tauri && cargo check --workspace 2>&1
      2. Assert: exit code 0
      3. cd src-tauri && cargo test -p uc-app --lib 2>&1
      4. Assert: "test result: ok", 0 failed
      5. cd src-tauri && cargo test -p uc-app --test setup_flow_integration_test 2>&1
      6. Assert: "test result: ok"
      7. bun run build 2>&1
      8. Assert: "built in" present
    Expected Result: Everything green
    Evidence: All command outputs captured

  Scenario: No unwrap/expect in new code
    Tool: Bash
    Preconditions: None
    Steps:
      1. grep -rn "unwrap()\|expect(" src-tauri/crates/uc-app/src/usecases/space_access/proof_adapter.rs src-tauri/crates/uc-app/src/usecases/space_access/persistence_adapter.rs src-tauri/crates/uc-app/src/usecases/space_access/network_adapter.rs 2>&1
      2. Assert: no matches (or only in test modules)
    Expected Result: Production code is unwrap-free
    Evidence: grep output
  ```

  **Commit**: YES (if cleanup changes made)
  - Message: `chore(app): cleanup dead_code warnings and verify join space access wiring`
  - Files: Any files with minor cleanup
  - Pre-commit: `cd src-tauri && cargo test -p uc-app`

---

## Commit Strategy

| After Task | Message                                                                                      | Files                                                     | Verification                         |
| ---------- | -------------------------------------------------------------------------------------------- | --------------------------------------------------------- | ------------------------------------ |
| 1          | `feat(core): update SpaceAccessTransportPort to return Result and integrate network adapter` | transport.rs, network_adapter.rs, mod.rs, orchestrator.rs | cargo test -p uc-app --lib           |
| 2          | `feat(app): implement derive_master_key_from_keyslot for joiner crypto flow`                 | crypto_adapter.rs                                         | cargo test -p uc-app --lib           |
| 3          | `feat(app): add HmacProofAdapter and SpaceAccessPersistenceAdapter`                          | proof_adapter.rs, persistence_adapter.rs, mod.rs          | cargo test -p uc-app --lib           |
| 4          | `feat(app): wire StartJoinSpaceAccess to SpaceAccessOrchestrator with runtime executor`      | setup/orchestrator.rs, setup_flow_integration_test.rs     | cargo test -p uc-app --lib           |
| 5          | `feat(bootstrap): wire SpaceAccess port adapters into SetupRuntimePorts`                     | bootstrap/runtime.rs, main.rs                             | cargo check --workspace              |
| 6          | `test(app): add join space access integration tests`                                         | setup_flow_integration_test.rs, orchestrator.rs           | cargo test -p uc-app                 |
| 7          | `chore(app): cleanup dead_code warnings and verify join space access wiring`                 | various                                                   | cargo test -p uc-app + bun run build |

---

## Success Criteria

### Verification Commands

```bash
cd src-tauri && cargo check --workspace  # Expected: 0 new errors/warnings
cd src-tauri && cargo test -p uc-app --lib  # Expected: all tests pass (99+)
cd src-tauri && cargo test -p uc-app --test setup_flow_integration_test  # Expected: all pass
bun run build  # Expected: successful build
grep -rn "StartJoinSpaceAccess invoked without" src-tauri/  # Expected: no matches
```

### Final Checklist

- [x] All "Must Have" present
- [x] All "Must NOT Have" absent
- [x] All tests pass
- [x] `StartJoinSpaceAccess` calls `SpaceAccessOrchestrator` (not placeholder)
- [x] All 3 missing adapters implemented with production logic
- [x] `derive_master_key_from_keyslot()` implemented
- [x] Bootstrap wiring complete
- [x] No `unwrap()`/`expect()` in production code
- [x] `dead_code` warning for `space_access_orchestrator` resolved
