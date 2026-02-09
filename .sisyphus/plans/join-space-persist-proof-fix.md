# Join Space Persist/Proof Fix Plan

## TL;DR

> **Quick Summary**: Fix join-space receiver reliability by removing runtime Noop space-access adapters from production event loops, then align persistence semantics so `paired_device` is committed only after setup/proof verification succeeds.
>
> **Deliverables**:
>
> - Runtime busy-message path uses real proof/transport/persistence adapters (no Noop in production path).
> - Pairing persistence is deferred until verification-complete boundary.
> - Regression tests prove: no noop-proof failures, no pre-verification final persistence.
>
> **Estimated Effort**: Large
> **Parallel Execution**: YES - 3 waves
> **Critical Path**: Task 1 -> Task 3 -> Task 5

---

## Context

### Original Request

User provided receiver-side logs and reported two issues:

1. Persistence happens too early (should wait until setup verification passes).
2. Proof verification fails with noop errors.

### Interview Summary

**Key Discussions**:

- User explicitly chose persistence policy: **persist only after verification**.
- User chose test strategy: **tests-after** (implement first, add tests afterward).
- Exhaustive parallel investigation completed across explore/librarian/oracle + direct searches.

**Research Findings**:

- `NoopSpaceAccessProof/Transport/Persistence` are instantiated on runtime busy-message handling path in `uc-tauri` wiring.
- Pairing flow currently persists paired device in pairing finalization phase (before setup/proof completion).
- Setup runtime wiring already has real adapters, but busy-message handling path does not consistently reuse that adapter set.

### Metis Review

**Identified Gaps (addressed in this plan)**:

- Lock down Noop adapter usage so test-only implementations cannot leak into production path.
- Define exact verification boundary for final persistence commit.
- Add explicit acceptance checks for both happy-path and negative-path behavior.

---

## Work Objectives

### Core Objective

Ensure join-space receiver flow is production-safe and semantically correct by (a) eliminating runtime Noop adapter execution, and (b) committing paired-device persistence only after setup/proof verification is successful.

### Concrete Deliverables

- `wiring.rs` busy payload path uses real runtime adapters for proof/transport/persistence.
- Pairing persistence no longer commits final `paired_device` before verification completion.
- New/updated tests covering proof verification path, busy message handling, and deferred persistence semantics.

### Definition of Done

- [x] No runtime logs contain `noop proof port cannot verify proof` or `noop transport port cannot send result` during join-space receiver flow.
- [x] Final paired-device persistence is absent before verification pass and present after verification pass.
- [x] Targeted crate tests pass from `src-tauri/` for modified domains.

### Must Have

- Production runtime path uses concrete adapters for space-access busy handling.
- Deferred final persistence policy implemented exactly as chosen by user.
- Regression coverage for happy + failure paths.

### Must NOT Have (Guardrails)

- No business decision leakage into bootstrap beyond wiring/validation.
- No Noop adapter usage in production runtime message path.
- No breaking of hexagonal boundaries (`uc-core` ports only; concrete implementations stay in `uc-infra`/`uc-tauri`).
- No hidden fallback that silently downgrades to Noop behavior.

---

## Verification Strategy (MANDATORY)

> **UNIVERSAL RULE: ZERO HUMAN INTERVENTION**
>
> All verification is agent-executed via commands and tool assertions. No manual click/visual confirmation required.

### Test Decision

- **Infrastructure exists**: YES
- **Automated tests**: Tests-after
- **Framework**: Rust test (`cargo test`, `#[test]`, `#[tokio::test]`)

### Agent-Executed QA Scenarios (applies to all tasks)

Scenario: Receiver busy-proof path uses real proof adapter
Tool: Bash (cargo test)
Preconditions: Working directory `src-tauri/`
Steps: 1. Run: `cargo test -p uc-tauri busy_proof_payload_routes_to_proof_branch_and_validates_nonce_length -- --nocapture` 2. Assert: test exits 0 3. Assert: output does not contain `noop proof port cannot verify proof`
Expected Result: busy-proof branch executes without noop-proof runtime failure
Failure Indicators: non-zero exit, noop-proof error string present
Evidence: terminal output capture

Scenario: Invalid proof/nonce is rejected safely
Tool: Bash (cargo test)
Preconditions: Working directory `src-tauri/`
Steps: 1. Run targeted negative-path test for malformed nonce / invalid proof payload 2. Assert: state transition reaches denied/rejected branch, process does not panic
Expected Result: deterministic failure path with explicit reason
Failure Indicators: panic/backtrace, unexpected granted state
Evidence: terminal output capture

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately):
├── Task 1: Replace runtime Noop usage in busy-message handling
└── Task 3: Implement deferred final paired-device persistence policy

Wave 2 (After Wave 1):
├── Task 2: Align sponsor-authorization execution path with real adapters
└── Task 4: Add fail-fast wiring guards for production

Wave 3 (After Wave 2):
└── Task 5: Add/adjust regression tests and run full targeted verification

Critical Path: Task 1 -> Task 3 -> Task 5
Parallel Speedup: ~30% vs pure sequential
```

### Dependency Matrix

| Task | Depends On | Blocks | Can Parallelize With |
| ---- | ---------- | ------ | -------------------- |
| 1    | None       | 2, 5   | 3                    |
| 2    | 1          | 5      | 4                    |
| 3    | None       | 5      | 1                    |
| 4    | 1          | 5      | 2                    |
| 5    | 1,2,3,4    | None   | None                 |

### Agent Dispatch Summary

| Wave | Tasks | Recommended Agents                                                                                             |
| ---- | ----- | -------------------------------------------------------------------------------------------------------------- |
| 1    | 1, 3  | task(category="deep", load_skills=["systematic-debugging","test-driven-development"], run_in_background=false) |
| 2    | 2, 4  | task(category="unspecified-high", load_skills=["systematic-debugging"], run_in_background=false)               |
| 3    | 5     | task(category="quick", load_skills=["verification-before-completion"], run_in_background=false)                |

---

## TODOs

- [x] 1. Replace runtime Busy-path Noop adapters with real adapter wiring

  **What to do**:
  - Refactor `run_pairing_event_loop`/`handle_pairing_message` to receive injected runtime space-access adapter dependencies.
  - Remove direct creation of `NoopSpaceAccessProof`, `NoopSpaceAccessTransport`, `NoopSpaceAccessPersistence` in production busy payload handling.
  - Ensure busy payload branches (`Offer`, `Proof`, `Result`) use a consistent executor built from real ports.

  **Must NOT do**:
  - Keep any production fallback to Noop on adapter errors.
  - Embed business policy decisions inside bootstrap wiring.

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: multi-file runtime wiring refactor with behavior-critical branching.
  - **Skills**: `systematic-debugging`, `test-driven-development`
    - `systematic-debugging`: protects against symptom-only fixes in async event flow.
    - `test-driven-development`: ensures behavior locks before/after wiring changes.
  - **Skills Evaluated but Omitted**:
    - `brainstorming`: discovery already completed; execution focus now.

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Task 3)
  - **Blocks**: 2, 5
  - **Blocked By**: None

  **References**:
  - `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs:1450` - `run_pairing_event_loop` entrypoint where network busy payloads are consumed.
  - `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs:1544` - `handle_pairing_message` currently dispatches space-access events.
  - `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs:1676` - Busy offer branch creates noop adapters.
  - `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs:1749` - Busy proof branch uses noop proof verifier.
  - `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs:1829` - Busy result branch still uses noop adapter set.
  - `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs:252` - Existing real adapter wiring pattern to replicate/route through.

  **Acceptance Criteria**:
  - [ ] Busy-path production code contains no direct `NoopSpaceAccess*` instantiation outside tests.
  - [ ] Busy offer/proof/result branches dispatch using real adapter-backed executor.
  - [ ] Targeted wiring/busy tests pass.

  **Agent-Executed QA Scenarios**:

  ```text
  Scenario: Busy proof path no longer uses noop verifier
    Tool: Bash
    Preconditions: cwd=src-tauri
    Steps:
      1. cargo test -p uc-tauri busy_proof_payload_routes_to_proof_branch_and_validates_nonce_length -- --nocapture
      2. Assert exit code == 0
      3. Assert output NOT contains "noop proof port cannot verify proof"
    Expected Result: test passes and no noop-proof error appears
    Failure Indicators: test failure, noop-proof string appears
    Evidence: terminal output capture

  Scenario: Unknown/invalid busy payload remains safely handled
    Tool: Bash
    Preconditions: cwd=src-tauri
    Steps:
      1. Run related parsing/branch tests covering unknown kind and malformed payload
      2. Assert no panic and expected warning/error branch executes
    Expected Result: robust defensive behavior retained
    Failure Indicators: panic, unexpected granted transition
    Evidence: terminal output capture
  ```

  **Commit**: YES
  - Message: `fix(tauri): route busy space-access handling through real adapters`
  - Files: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`
  - Pre-commit: `cargo test -p uc-tauri busy_proof_payload_routes_to_proof_branch_and_validates_nonce_length -- --nocapture`

- [x] 2. Align sponsor-authorization action loop with non-Noop execution path

  **What to do**:
  - Refactor sponsor authorization bootstrap in pairing action loop to avoid noop proof/timer/store in runtime path.
  - Ensure action-loop bootstrap and event-loop busy handling share compatible adapter lifecycle/context.

  **Must NOT do**:
  - Introduce duplicate competing state machines for same session without lock/ordering.

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: async orchestration and shared context correctness.
  - **Skills**: `systematic-debugging`
    - `systematic-debugging`: race/order risks are primary failure mode.
  - **Skills Evaluated but Omitted**:
    - `test-driven-development`: already covered in overall tests-after policy for this sequence.

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Task 4)
  - **Blocks**: 5
  - **Blocked By**: 1

  **References**:
  - `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs:2013` - `PairingAction::EmitResult` handling branch.
  - `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs:2056` - sponsor authorization currently injects noop proof/store/timer.
  - `src-tauri/crates/uc-app/src/usecases/space_access/orchestrator.rs:62` - `start_sponsor_authorization` expected execution contract.

  **Acceptance Criteria**:
  - [ ] Sponsor authorization runtime path does not depend on noop proof/store in production.
  - [ ] Existing sponsor authorization test coverage remains green or updated accordingly.

  **Agent-Executed QA Scenarios**:

  ```text
  Scenario: Responder success triggers sponsor authorization without noop errors
    Tool: Bash
    Preconditions: cwd=src-tauri
    Steps:
      1. cargo test -p uc-tauri pairing_action_loop_starts_sponsor_authorization_for_responder_role -- --nocapture
      2. Assert exit code == 0
      3. Assert output NOT contains "noop transport port cannot send result"
    Expected Result: sponsor authorization path starts successfully
    Failure Indicators: test failure, noop transport/proof error
    Evidence: terminal output capture

  Scenario: Failed pairing still closes session and does not start authorization
    Tool: Bash
    Preconditions: cwd=src-tauri
    Steps:
      1. cargo test -p uc-tauri pairing_action_loop_closes_session_only_for_failed_emit_result -- --nocapture
      2. Assert exit code == 0
    Expected Result: failure path behavior unchanged
    Failure Indicators: authorization triggered on failed pair
    Evidence: terminal output capture
  ```

  **Commit**: YES
  - Message: `fix(tauri): harden sponsor authorization execution path`
  - Files: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`
  - Pre-commit: `cargo test -p uc-tauri pairing_action_loop_starts_sponsor_authorization_for_responder_role -- --nocapture`

- [x] 3. Defer final paired-device persistence until verification-complete boundary

  **What to do**:
  - Remove/replace early final persistence in pairing finalization path.
  - Introduce deferred commit point tied to successful verification completion (setup/proof success boundary).
  - Ensure pairing still emits required domain events for downstream setup flow without final persistence side effect.

  **Must NOT do**:
  - Break pairing UI flow/event emissions required by existing frontend listeners.
  - Conflate pairing trust bootstrap and setup authorization semantics silently.

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: cross-layer semantic change (core state machine + app orchestration contract).
  - **Skills**: `systematic-debugging`, `test-driven-development`
    - `systematic-debugging`: prevents policy regression in event order.
    - `test-driven-development`: protects state transition semantics during refactor.
  - **Skills Evaluated but Omitted**:
    - `writing-plans`: already in execution phase.

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Task 1)
  - **Blocks**: 5
  - **Blocked By**: None

  **References**:
  - `src-tauri/crates/uc-core/src/network/pairing_state_machine.rs:1301` - current early `PersistPairedDevice` emission.
  - `src-tauri/crates/uc-core/src/network/pairing_state_machine.rs:1349` - `PersistOk` -> `EmitResult(success=true)` ordering.
  - `src-tauri/crates/uc-app/src/usecases/pairing/orchestrator.rs:858` - current immediate `device_repo.upsert` side effect.
  - `src-tauri/crates/uc-core/src/security/space_access/state_machine.rs:119` - verification success transition (`ProofVerified` -> `Granted`).
  - `src-tauri/crates/uc-app/src/usecases/space_access/orchestrator.rs:371` - sponsor persistence execution point.
  - `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs:277` - join-space action pipeline and setup integration boundary.

  **Acceptance Criteria**:
  - [ ] Before verification success, no final `paired_device` commit exists.
  - [ ] After verification success, final `paired_device` commit exists and setup flow remains functional.
  - [ ] Pairing domain events used by setup listener still fire in expected order.

  **Agent-Executed QA Scenarios**:

  ```text
  Scenario: No final persistence before verification
    Tool: Bash
    Preconditions: cwd=src-tauri, test covers pairing success but pre-verification state
    Steps:
      1. Run targeted unit/integration test for pairing-success-before-setup-verification
      2. Assert repository has no committed paired-device record at that checkpoint
    Expected Result: deferred persistence policy enforced
    Failure Indicators: paired-device record committed early
    Evidence: test output with assertions

  Scenario: Verification success commits paired device
    Tool: Bash
    Preconditions: cwd=src-tauri, deterministic proof-verified flow fixture
    Steps:
      1. Run targeted test for proof-verified join-space completion
      2. Assert paired-device record committed exactly once
    Expected Result: commit occurs only at chosen boundary
    Failure Indicators: no commit, duplicate commit, or pre-commit
    Evidence: test output with assertions
  ```

  **Commit**: YES (split by boundary)
  - Message: `arch(core): defer paired-device persistence trigger until verification`
  - Files: `src-tauri/crates/uc-core/src/network/pairing_state_machine.rs`
  - Pre-commit: `cargo test -p uc-core pairing_state_machine -- --nocapture`

- [x] 4. Add fail-fast Noop guardrails in production wiring

  **What to do**:
  - Restrict Noop adapter definitions/constructors to test scope (`#[cfg(test)]` or equivalent safe boundary).
  - Add startup/runtime wiring validation that fails fast if Noop adapters are wired into production path.

  **Must NOT do**:
  - Depend on log-only warnings for critical wiring mismatches.

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: guardrail-focused changes with clear acceptance checks.
  - **Skills**: `systematic-debugging`
    - `systematic-debugging`: ensures hard failure instead of silent fallback.
  - **Skills Evaluated but Omitted**:
    - `verification-before-completion`: used globally in final wave.

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Task 2)
  - **Blocks**: 5
  - **Blocked By**: 1

  **References**:
  - `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs:1234` - noop type declarations currently in shared compilation unit.
  - `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs:953` - `start_background_tasks` composition root for runtime loops.
  - `src-tauri/crates/uc-tauri/src/bootstrap/README.md` - bootstrap role/guardrail constraints.

  **Acceptance Criteria**:
  - [ ] Production path cannot instantiate Noop space-access adapters.
  - [ ] Wiring validation fails fast with explicit error when adapter contract is violated.

  **Agent-Executed QA Scenarios**:

  ```text
  Scenario: Wiring health checks reject invalid adapter composition
    Tool: Bash
    Preconditions: cwd=src-tauri
    Steps:
      1. Run wiring/bootstrap tests covering initialization validation
      2. Assert invalid composition produces deterministic startup error
    Expected Result: fail-fast behavior confirmed
    Failure Indicators: startup continues with invalid/noop composition
    Evidence: test output capture

  Scenario: Existing happy-path bootstrap still succeeds
    Tool: Bash
    Preconditions: cwd=src-tauri
    Steps:
      1. Run key bootstrap integration tests
      2. Assert exit code == 0
    Expected Result: guardrails do not break valid startup
    Failure Indicators: regression in healthy bootstrap
    Evidence: terminal output capture
  ```

  **Commit**: YES
  - Message: `fix(tauri): enforce fail-fast guardrails against noop runtime wiring`
  - Files: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`
  - Pre-commit: `cargo test -p uc-tauri bootstrap -- --nocapture`

- [x] 5. Add regression tests and execute final verification matrix

  **What to do**:
  - Add/adjust tests for deferred persistence and no-noop runtime behavior.
  - Run targeted test matrix across `uc-core`, `uc-app`, `uc-tauri`.
  - Capture evidence logs proving both original issues are fixed.

  **Must NOT do**:
  - Claim success without command output evidence.

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: verification-heavy, execution-oriented final sweep.
  - **Skills**: `verification-before-completion`
    - `verification-before-completion`: requires evidence-first completion gate.
  - **Skills Evaluated but Omitted**:
    - `requesting-code-review`: optional after local verification.

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 3 (final)
  - **Blocks**: None
  - **Blocked By**: 1,2,3,4

  **References**:
  - `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs:2863` - existing busy offer route test to extend/assert.
  - `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs:2908` - busy proof route test anchor.
  - `src-tauri/crates/uc-app/src/usecases/pairing/orchestrator.rs:1200` - pairing orchestrator test module anchor.
  - `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs:1495` - setup flow tests likely affected by persistence timing shift.

  **Acceptance Criteria**:
  - [ ] All new/updated targeted tests pass.
  - [ ] No noop proof/transport runtime strings appear in test logs for happy path.
  - [ ] Deferred persistence semantics are asserted in tests.

  **Agent-Executed QA Scenarios**:

  ```text
  Scenario: Full targeted matrix passes
    Tool: Bash
    Preconditions: cwd=src-tauri
    Steps:
      1. cargo test -p uc-core pairing_state_machine -- --nocapture
      2. cargo test -p uc-app pairing -- --nocapture
      3. cargo test -p uc-app setup::orchestrator -- --nocapture
      4. cargo test -p uc-tauri busy_proof_payload_routes_to_proof_branch_and_validates_nonce_length -- --nocapture
    Expected Result: all commands exit 0
    Failure Indicators: any non-zero exit or panic
    Evidence: terminal output capture files

  Scenario: Regression check for original failure signatures
    Tool: Bash
    Preconditions: test output logs available
    Steps:
      1. Search captured outputs for strings:
         - "noop proof port cannot verify proof"
         - "noop transport port cannot send result"
      2. Assert zero matches in happy-path runs
    Expected Result: original signatures absent in fixed path
    Failure Indicators: any match found
    Evidence: grep/search output capture
  ```

  **Commit**: YES
  - Message: `test(pairing): add regressions for deferred persistence and real proof path`
  - Files: targeted `*_test.rs` in touched crates
  - Pre-commit: targeted matrix above

---

## Commit Strategy

| After Task | Message                                                                       | Files                                      | Verification                                                                                                 |
| ---------- | ----------------------------------------------------------------------------- | ------------------------------------------ | ------------------------------------------------------------------------------------------------------------ |
| 1          | `fix(tauri): route busy space-access handling through real adapters`          | `uc-tauri/bootstrap/wiring.rs`             | `cargo test -p uc-tauri busy_proof_payload_routes_to_proof_branch_and_validates_nonce_length -- --nocapture` |
| 3 (core)   | `arch(core): defer paired-device persistence trigger until verification`      | `uc-core/network/pairing_state_machine.rs` | `cargo test -p uc-core pairing_state_machine -- --nocapture`                                                 |
| 3 (app)    | `impl(app): commit paired device only after verification-complete`            | `uc-app/usecases/*`                        | `cargo test -p uc-app pairing -- --nocapture`                                                                |
| 4          | `fix(tauri): enforce fail-fast guardrails against noop runtime wiring`        | `uc-tauri/bootstrap/*`                     | `cargo test -p uc-tauri bootstrap -- --nocapture`                                                            |
| 5          | `test(pairing): add regressions for deferred persistence and real proof path` | updated tests                              | matrix run                                                                                                   |

---

## Success Criteria

### Verification Commands

```bash
# run from src-tauri/
cargo test -p uc-core pairing_state_machine -- --nocapture
cargo test -p uc-app pairing -- --nocapture
cargo test -p uc-app setup::orchestrator -- --nocapture
cargo test -p uc-tauri busy_proof_payload_routes_to_proof_branch_and_validates_nonce_length -- --nocapture
cargo test -p uc-tauri pairing_action_loop_starts_sponsor_authorization_for_responder_role -- --nocapture
```

### Final Checklist

- [x] All "Must Have" items implemented
- [x] All "Must NOT Have" violations absent
- [x] Proof verification happy path does not touch Noop runtime adapters
- [x] Final paired-device persistence occurs only after verification-complete boundary
- [x] Targeted tests pass with evidence captured
