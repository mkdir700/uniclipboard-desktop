# Join Space Space-Access Wire-up

## TL;DR

> **Quick Summary**: Complete the production wiring for Sponsor/Joiner space-access so join flow no longer stalls after passphrase submission.
>
> **Deliverables**:
>
> - Busy payload routing (`offer/proof/result`) in bootstrap wiring
> - Sponsor auto-start for `start_sponsor_authorization(...)` (renamed from `initialize_new_space(...)`)
> - Joiner convergence driven by Sponsor result (no local forced success)
> - Setup completion bridge + session lifecycle policy
> - Wiring-focused tests (happy/failure/timeout/lifecycle)
>
> **Estimated Effort**: Medium
> **Parallel Execution**: YES - 4 waves
> **Critical Path**: Task 1 -> Task 2 -> Task 4 -> Task 6 -> Task 7

---

## Context

### Original Request

Fix join-space flow where Joiner enters long wait after passphrase and does not receive expected key verification success.

### Interview Summary

**Key Discussions**:

- Root cause is not one bug but a missing wire-up chain across `uc-app` and `uc-tauri`.
- Reuse pairing session via `PairingMessage::Busy`.
- Sponsor should auto-trigger after pairing success.

**Research Findings**:

- State machine in `uc-core` is already complete; production routing is incomplete.
- Critical gaps: Busy parsing/routing, sponsor start trigger, joiner local forced success, completion-to-setup bridge.

### Metis Review

**Identified Gaps (addressed)**:

- Metis session produced no additional structured bullets; gaps were validated through subagent reviews and draft hardening.
- Added explicit guardrails for race/order, session lifecycle, and no-local-success convergence.

---

## Work Objectives

### Core Objective

Ensure Joiner and Sponsor complete the full space-access handshake over production transport with deterministic convergence and no infinite waiting.

### Concrete Deliverables

- `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs`
- `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`
- `src-tauri/crates/uc-app/tests/setup_flow_integration_test.rs`
- `src-tauri/crates/uc-tauri` wiring-level tests for Busy route parsing/dispatch

### Definition of Done

- [ ] Joiner no longer dispatches local `AccessGranted` before Sponsor result.
- [ ] Busy payloads (`space_access_offer`, `space_access_proof`, `space_access_result`) are parsed and routed in wiring.
- [ ] Sponsor auto-starts `start_sponsor_authorization(...)` only for sponsor path.
- [ ] Session closes only on failure/terminal/timeout policy.
- [ ] Happy + failure + timeout tests pass.

### Must Have

- Keep business logic in `uc-app`; keep `uc-tauri` as wiring/operator.
- Maintain `uc-core` state-machine semantics.

### Must NOT Have (Guardrails)

- Do not add protocol DTOs to `uc-core` for transport-only Busy JSON.
- Do not hardcode unrecoverable `space_id` source without explicit provenance.
- Do not close pairing session on pairing success before space-access terminal state.
- Do not accept malformed Busy payload silently.
- Do not keep ambiguous naming (`initialize_new_space`) in join-space sponsor authorization path.

---

## Verification Strategy (MANDATORY)

> **UNIVERSAL RULE: ZERO HUMAN INTERVENTION**
>
> All verification in this plan is agent-executed by commands/tools only.

### Test Decision

- **Infrastructure exists**: YES
- **Automated tests**: Tests-after
- **Framework**: cargo test (+ existing Rust unit/integration tests)

### Agent-Executed QA Scenarios (MANDATORY - ALL tasks)

Scenario: Joiner waits for sponsor result (no local forced success)
Tool: Bash (cargo test)
Preconditions: test fixture with setup orchestrator and mocked sponsor delayed result
Steps: 1. Trigger join-space flow to `SubmitPassphrase` 2. Assert space-access state reaches `WaitingDecision` 3. Assert no immediate `AccessGranted` transition without result payload
Expected Result: Joiner remains pending until result route is received
Failure Indicators: immediate success without sponsor result
Evidence: test output in cargo log

Scenario: Busy offer/proof/result routing
Tool: Bash (cargo test)
Preconditions: wiring test harness with synthetic `PairingMessage::Busy`
Steps: 1. Inject Busy payload with `kind=space_access_offer` 2. Inject Busy payload with `kind=space_access_proof` 3. Inject Busy payload with `kind=space_access_result` 4. Assert corresponding orchestrator dispatch path is invoked
Expected Result: all 3 payload kinds route correctly
Failure Indicators: unknown payload, no dispatch, parse error without logging
Evidence: test assertions and captured logs

Scenario: Timeout convergence
Tool: Bash (cargo test)
Preconditions: Joiner enters `WaitingDecision`, sponsor intentionally does not respond
Steps: 1. Start join flow and submit passphrase 2. Do not inject sponsor result 3. Advance timer or wait test timeout window 4. Assert setup enters observable failure state
Expected Result: no infinite wait
Failure Indicators: state remains `ProcessingJoinSpace` indefinitely
Evidence: timeout test assertions

---

## Execution Strategy

### Parallel Execution Waves

Wave 1 (Start Immediately):

- Task 0: Rename sponsor API from `initialize_new_space` to `start_sponsor_authorization`
- Task 1: Remove Joiner local forced success path
- Task 2: Define Busy payload DTO/parser in wiring layer

Wave 2 (After Wave 1):

- Task 3: Implement Busy offer route
- Task 4: Implement Busy proof route
- Task 5: Implement Busy result route

Wave 3 (After Wave 2):

- Task 6: Sponsor auto-trigger + setup completion bridge

Wave 4 (After Wave 3):

- Task 7: Verification tests (wiring + integration + timeout + lifecycle)

Critical Path: 0 -> 1 -> 2 -> 4 -> 6 -> 7

### Dependency Matrix

| Task | Depends On    | Blocks  | Can Parallelize With |
| ---- | ------------- | ------- | -------------------- |
| 0    | None          | 1,2,6,7 | None                 |
| 1    | 0             | 6,7     | 2                    |
| 2    | 0             | 3,4,5,7 | 1                    |
| 3    | 2             | 6,7     | 4,5                  |
| 4    | 2             | 6,7     | 3,5                  |
| 5    | 2             | 6,7     | 3,4                  |
| 6    | 0,1,3,4,5     | 7       | None                 |
| 7    | 0,1,2,3,4,5,6 | None    | None                 |

### Agent Dispatch Summary

| Wave | Tasks | Recommended Agents                                                                          |
| ---- | ----- | ------------------------------------------------------------------------------------------- |
| 1    | 0,1,2 | `task(category="unspecified-high", load_skills=["systematic-debugging"])`                   |
| 2    | 3,4,5 | `task(category="unspecified-high", load_skills=["executing-plans"])`                        |
| 3    | 6     | `task(category="unspecified-high", load_skills=["executing-plans","systematic-debugging"])` |
| 4    | 7     | `task(category="unspecified-high", load_skills=["test-driven-development"])`                |

---

## TODOs

- [x] 0. Rename Sponsor API for Clarity

  **What to do**:
  - Rename `SpaceAccessOrchestrator::initialize_new_space(...)` to `start_sponsor_authorization(...)`.
  - Rename use-case wrapper `InitializeNewSpace` to `StartSponsorAuthorization` (or equivalent unambiguous name).
  - Update all call sites, exports, and related tests.

  **Must NOT do**:
  - Do not change business semantics during rename.
  - Do not mix rename with behavioral changes.

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: `executing-plans`, `systematic-debugging`

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 1 (sequential prerequisite)
  - **Blocks**: 1,2,6,7
  - **Blocked By**: None

  **References**:
  - `src-tauri/crates/uc-app/src/usecases/space_access/orchestrator.rs` - current API name and sponsor event dispatch.
  - `src-tauri/crates/uc-app/src/usecases/space_access/initialize_new_space.rs` - wrapper use-case naming.
  - `src-tauri/crates/uc-app/src/usecases/space_access/mod.rs` - public exports.
  - `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs` - sponsor auto-trigger call site.

  **Acceptance Criteria**:
  - [ ] No sponsor authorization path uses `initialize_new_space` naming.
  - [ ] All references compile with new name.
  - [ ] Tests updated with no semantic drift.

  **Commit**: YES
  - Message: `refactor(space-access): rename sponsor authorization entrypoints`

- [x] 1. Remove Joiner Local Forced Success

  **What to do**:
  - Update `start_join_space_access` to stop after `PassphraseSubmitted` path.
  - Remove local dispatch of `SpaceAccessEvent::AccessGranted` in join path.
  - Remove direct session close in this function.

  **Must NOT do**:
  - Do not change `uc-core` state machine transitions.

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: cross-layer correctness and race-sensitive behavior.
  - **Skills**: `systematic-debugging`, `executing-plans`
    - `systematic-debugging`: prevents hidden state-order regressions.
    - `executing-plans`: keeps ordered execution and validation.
  - **Skills Evaluated but Omitted**:
    - `frontend-ui-ux`: no UI/design scope.

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Task 2)
  - **Blocks**: 6, 7
  - **Blocked By**: 0

  **References**:
  - `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs` - `start_join_space_access` current sequence.
  - `src-tauri/crates/uc-core/src/security/space_access/state_machine.rs` - valid Joiner transitions and terminal states.

  **Acceptance Criteria**:
  - [ ] `start_join_space_access` no longer dispatches local `AccessGranted`.
  - [ ] Joiner can remain in `WaitingDecision` pending sponsor result.

  **Commit**: YES
  - Message: `fix(setup): remove local joiner access-granted convergence`

- [x] 2. Define Busy Payload DTO + Parser in Wiring

  **What to do**:
  - Add wiring-layer DTO(s) for Busy `reason` JSON parsing.
  - Normalize parse errors with structured logs including `session_id`, `peer_id`, payload kind.

  **Must NOT do**:
  - Do not place Busy JSON DTO in `uc-core`.

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: `executing-plans`, `systematic-debugging`
  - **Skills Evaluated but Omitted**:
    - `writing`: implementation-first task.

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Task 1)
  - **Blocks**: 3,4,5,7
  - **Blocked By**: 0

  **References**:
  - `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs` - `handle_pairing_message` dispatch point.
  - `src-tauri/crates/uc-app/src/usecases/space_access/network_adapter.rs` - sender payload schema (`space_access_offer/proof/result`).

  **Acceptance Criteria**:
  - [ ] Wiring parser accepts all 3 known Busy kinds.
  - [ ] Unknown/malformed payloads are logged and ignored safely.

  **Commit**: YES
  - Message: `impl(wiring): add busy payload dto and parser`

- [x] 3. Route `space_access_offer` to Joiner Flow

  **What to do**:
  - In `handle_pairing_message`, route `space_access_offer` to the joiner handling path.
  - Ensure challenge nonce length validation (32 bytes) before dispatch.
  - Treat `space_id` in Sponsor offer payload as authority for Joiner-side `OfferAccepted` dispatch.

  **Must NOT do**:
  - Do not silently coerce invalid challenge lengths.

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: `executing-plans`, `systematic-debugging`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 4,5)
  - **Blocks**: 6,7
  - **Blocked By**: 2

  **References**:
  - `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs` - pairing message handler.
  - `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs` - keyslot/offer capture points.

  **Acceptance Criteria**:
  - [ ] Valid offer payload reaches Joiner state transition path.
  - [ ] Invalid challenge length emits warning and does not crash.
  - [ ] Joiner `space_id` used for transition equals payload `space_id` from sponsor offer.

  **Commit**: YES
  - Message: `impl(wiring): route space_access_offer payload`

- [x] 4. Route `space_access_proof` and Drive Sponsor Decision

  **What to do**:
  - Parse proof payload and dispatch sponsor-side verification event.
  - On verification outcome, dispatch `ProofVerified` or `ProofRejected`.

  **Must NOT do**:
  - Do not bypass orchestrator by writing final state directly.

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: `executing-plans`, `systematic-debugging`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 3,5)
  - **Blocks**: 6,7
  - **Blocked By**: 2

  **References**:
  - `src-tauri/crates/uc-core/src/security/space_access/event.rs` - proof events contract.
  - `src-tauri/crates/uc-app/src/usecases/space_access/orchestrator.rs` - dispatch and completion event emit.

  **Acceptance Criteria**:
  - [ ] Sponsor receives proof and resolves verified/rejected deterministically.
  - [ ] Result send action is triggered by sponsor-side transition.

  **Commit**: YES
  - Message: `impl(space-access): route and verify sponsor proof payload`

- [x] 5. Route `space_access_result` and Finalize Joiner

  **What to do**:
  - Parse result payload and dispatch `AccessGranted` or `AccessDenied` on Joiner.
  - Ensure this is the only terminal success trigger for joiner flow.

  **Must NOT do**:
  - Do not keep alternative local success path.

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: `executing-plans`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 3,4)
  - **Blocks**: 6,7
  - **Blocked By**: 2

  **References**:
  - `src-tauri/crates/uc-core/src/security/space_access/state_machine.rs` - result transition from `WaitingDecision`.
  - `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs` - join state propagation path.

  **Acceptance Criteria**:
  - [ ] Joiner terminal state is result-driven.
  - [ ] Denied path surfaces failure in setup state.

  **Commit**: YES
  - Message: `impl(space-access): route result payload to joiner terminal state`

- [x] 6. Sponsor Auto-Trigger + Completion Bridge + Session Policy

  **What to do**:
  - Trigger sponsor `start_sponsor_authorization(...)` on pairing success for sponsor role only.
  - Bridge `SpaceAccessCompletedEvent` to setup convergence events.
  - Enforce session close policy: close only failure/terminal/timeout/cancel.

  **Must NOT do**:
  - Do not trigger sponsor init for joiner role.
  - Do not close session on pairing success alone.

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: `executing-plans`, `systematic-debugging`

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 3 (sequential)
  - **Blocks**: 7
  - **Blocked By**: 0,1,3,4,5

  **References**:
  - `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs` - pairing action loop and space-access completion loop.
  - `src-tauri/crates/uc-app/src/usecases/space_access/events.rs` - completion event contract.

  **Acceptance Criteria**:
  - [ ] Sponsor auto-start occurs only on sponsor branch.
  - [ ] Setup receives success/failure convergence from completion bridge.
  - [ ] Session is not closed before space-access terminal state.

  **Commit**: YES
  - Message: `fix(wiring): bridge completion and enforce session close lifecycle`

- [ ] 7. Add Wiring-Focused and End-to-End Tests

  **What to do**:
  - Add/update tests for Busy routing (offer/proof/result).
  - Add happy/failure timeout convergence tests.
  - Add lifecycle assertion for close timing.

  **Must NOT do**:
  - Do not rely only on orchestrator-only tests with no wiring path.

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: `test-driven-development`, `systematic-debugging`

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 4 (final)
  - **Blocks**: None
  - **Blocked By**: 0,1,2,3,4,5,6

  **References**:
  - `src-tauri/crates/uc-app/tests/setup_flow_integration_test.rs` - current integration baseline.
  - `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs` - routing code under test.

  **Acceptance Criteria**:
  - [ ] Happy path: offer -> proof -> result(granted) completes.
  - [ ] Failure path: proof rejected -> denied surface.
  - [ ] Timeout path: no sponsor result -> observable setup failure.
  - [ ] Session close occurs after terminal state only.

  **Commit**: YES
  - Message: `test(space-access): add wiring route and convergence coverage`

---

## Commit Strategy

| After Task | Message                                                            | Files                         | Verification                                              |
| ---------- | ------------------------------------------------------------------ | ----------------------------- | --------------------------------------------------------- |
| 0          | `refactor(space-access): rename sponsor authorization entrypoints` | `uc-app + wiring`             | `cargo check`                                             |
| 1          | `fix(setup): remove local joiner access-granted convergence`       | `uc-app setup orchestrator`   | `cargo test -p uc-app setup::orchestrator -- --nocapture` |
| 2          | `impl(wiring): add busy payload dto and parser`                    | `uc-tauri wiring`             | `cargo check`                                             |
| 3-5        | `impl(space-access): route busy payloads`                          | `wiring + setup/space-access` | targeted tests                                            |
| 6          | `fix(wiring): bridge completion and session lifecycle`             | `wiring`                      | integration tests                                         |
| 7          | `test(space-access): add routing and convergence tests`            | `uc-app/uc-tauri tests`       | full listed test commands                                 |

---

## Success Criteria

### Verification Commands

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test -p uc-app setup::orchestrator -- --nocapture
cd src-tauri && cargo test -p uc-app --test setup_flow_integration_test -- --nocapture
cd src-tauri && cargo test -p uc-tauri wiring -- --nocapture
```

### Final Checklist

- [ ] All Must Have items implemented
- [ ] All Must NOT Have guardrails respected
- [ ] No local forced success path on joiner
- [ ] Busy routing complete for offer/proof/result
- [ ] Sponsor auto-trigger and completion bridge verified
- [ ] Session lifecycle policy enforced and tested
