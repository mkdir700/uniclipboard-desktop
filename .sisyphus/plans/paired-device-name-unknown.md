# Persist Paired Device Name

## TL;DR

> **Quick Summary**: Persist peer device names at pairing time and plumb the field through uc-core → uc-infra → uc-tauri so `get_paired_peers_with_status` returns real names instead of "Unknown Device".
>
> **Deliverables**:
>
> - PairedDevice includes `device_name` and is populated from pairing context.
> - DB migration + Diesel schema/mappers/repo updated for `paired_device.device_name`.
> - `get_paired_peers_with_status` uses persisted device_name (with safe fallback).
> - TDD tests in uc-core and uc-infra (and a mapping test in uc-tauri if needed).
>
> **Estimated Effort**: Medium
> **Parallel Execution**: NO - sequential (model → DB → command)
> **Critical Path**: Task 1 → Task 2 → Task 3

---

## Context

### Original Request

Fix the issue where paired devices show `Unknown` name after pairing completes.

### Interview Summary

**Key Discussions**:

- Persist paired device names (not just discovery cache) to fix root cause.
- Use TDD for changes.

**Research Findings**:

- `get_paired_peers_with_status` (src-tauri/crates/uc-tauri/src/commands/pairing.rs) derives name from discovered peers only; missing → `"Unknown Device"`.
- `PairedDevice` (src-tauri/crates/uc-core/src/network/paired_device.rs) has no device_name field today.
- Pairing state machine already tracks `peer_device_name` in context (src-tauri/crates/uc-core/src/network/pairing_state_machine.rs).
- `paired_device` DB table lacks device_name (src-tauri/crates/uc-infra/src/db/schema.rs and migrations).

### Metis Review

**Skipped**: Sub-agent execution unavailable in current runtime. Manual gap analysis applied.

---

## Work Objectives

### Core Objective

Persist peer device names at pairing time and return them via `get_paired_peers_with_status` so the Devices page shows real names immediately after pairing.

### Concrete Deliverables

- Add `device_name` to `uc-core::network::PairedDevice` and populate from pairing context.
- Add DB column + migration for `paired_device.device_name` and update Diesel schema/mappers/repo.
- Update `get_paired_peers_with_status` to use persisted name (fallback only if missing).

### Definition of Done

- [ ] `cargo test -p uc-core` passes (run from `src-tauri/`).
- [ ] `cargo test -p uc-infra` passes (run from `src-tauri/`).
- [ ] `cargo test -p uc-tauri` passes if mapping test added (run from `src-tauri/`).
- [ ] Devices list shows actual paired device names after pairing (no `Unknown Device`).

### Must Have

- Persisted device_name for new pairings.
- No Hexagonal boundary violations (core stays free of infra/platform).

### Must NOT Have (Guardrails)

- No new unwrap/expect in production Rust code.
- No UI-only hacks that bypass backend persistence.
- No fixed-pixel layouts or unrelated UI changes.

---

## Verification Strategy (MANDATORY)

### Test Decision

- **Infrastructure exists**: YES (Vitest + Cargo)
- **User wants tests**: TDD
- **Framework**: `cargo test` (Rust), `vitest` (frontend if needed)

### TDD Structure (for each Rust task)

1. **RED**: Write failing test for new device_name behavior.
2. **GREEN**: Implement minimal code to pass.
3. **REFACTOR**: Clean up while keeping tests green.

> **Note**: All Cargo commands must run from `src-tauri/`.

---

## Execution Strategy

### Parallel Execution Waves

Wave 1:
└── Task 1 (core model + state machine + tests)

Wave 2 (after Wave 1):
└── Task 2 (DB migration + mappers/repo + tests)

Wave 3 (after Wave 2):
└── Task 3 (command mapping + optional tests)

Critical Path: Task 1 → Task 2 → Task 3

---

## TODOs

- [x] 1. Extend PairedDevice with device_name and populate from pairing context (TDD)

  **What to do**:
  - Add `device_name: String` to `uc-core::network::PairedDevice`.
  - Use `PairingContext.peer_device_name` in `build_paired_device()` (pairing_state_machine).
  - Update all PairedDevice constructors in uc-core/uc-app tests or helpers to include device_name.
  - Ensure fallback if peer_device_name is missing (default to `"Unknown Device"` or chosen constant).

  **Must NOT do**:
  - Do not add infra/platform dependencies to uc-core.

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: core domain changes + tests
  - **Skills**: `test-driven-development`
    - `test-driven-development`: required TDD workflow
  - **Skills Evaluated but Omitted**:
    - `systematic-debugging`: not a runtime bug investigation

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 1
  - **Blocks**: Task 2, Task 3
  - **Blocked By**: None

  **References**:
  - `src-tauri/crates/uc-core/src/network/paired_device.rs` - PairedDevice definition (add device_name)
  - `src-tauri/crates/uc-core/src/network/pairing_state_machine.rs` - PairingContext.peer_device_name + build_paired_device()
  - `src-tauri/crates/uc-app/src/usecases/pairing/resolve_connection_policy.rs` - PairedDevice constructor needs new field

  **Acceptance Criteria (TDD)**:
  - [x] Add failing test in `pairing_state_machine.rs` proving PairedDevice contains peer_device_name.
  - [x] `cargo test -p uc-core` (from `src-tauri/`) → PASS

- [x] 2. Persist device_name in DB layer (migration + schema + mapper + repo) with TDD

  **What to do**:
  - Create migration under `src-tauri/crates/uc-infra/migrations/` to add `device_name` to `paired_device`.
  - Update `schema.rs`, `paired_device_row.rs`, `paired_device_mapper.rs`, and `paired_device_repo.rs`.
  - Extend repository tests to assert device_name round-trips.

  **Must NOT do**:
  - Avoid breaking existing records; use a safe default for existing rows.

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: schema + mapper + repository updates
  - **Skills**: `test-driven-development`
    - `test-driven-development`: required TDD workflow
  - **Skills Evaluated but Omitted**:
    - `systematic-debugging`: no runtime debugging required

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 3
  - **Blocked By**: Task 1

  **References**:
  - `src-tauri/crates/uc-infra/migrations/2026-01-24-000000_create_paired_device/up.sql` - existing table schema
  - `src-tauri/crates/uc-infra/src/db/schema.rs` - Diesel schema update
  - `src-tauri/crates/uc-infra/src/db/models/paired_device_row.rs` - row struct
  - `src-tauri/crates/uc-infra/src/db/mappers/paired_device_mapper.rs` - row ↔ domain mapping
  - `src-tauri/crates/uc-infra/src/db/repositories/paired_device_repo.rs` - upsert + tests

  **Acceptance Criteria (TDD)**:
  - [x] Add failing test in `paired_device_repo.rs` for device_name persistence.
  - [x] Migration adds `device_name` with safe default for existing rows.
  - [x] `cargo test -p uc-infra` (from `src-tauri/`) → PASS

- [x] 3. Return persisted device_name in get_paired_peers_with_status (TDD if feasible)

  **What to do**:
  - Update `get_paired_peers_with_status` to use `device.device_name` instead of discovery-only name.
  - Keep addresses/connected derived from discovery + connection status.
  - Optional: add a small pure mapping helper and a unit test to assert name selection.

  **Must NOT do**:
  - Do not reintroduce `"Unknown Device"` unless device_name is actually missing.

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: small mapping adjustment
  - **Skills**: `test-driven-development`
    - `test-driven-development`: maintain TDD discipline
  - **Skills Evaluated but Omitted**:
    - `systematic-debugging`: no runtime debugging required

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 3
  - **Blocks**: None
  - **Blocked By**: Task 2

  **References**:
  - `src-tauri/crates/uc-tauri/src/commands/pairing.rs` - `get_paired_peers_with_status` mapping logic
  - `src-tauri/crates/uc-tauri/tests/commands_test.rs` - existing test pattern (command exposure)

  **Acceptance Criteria (TDD)**:
  - [x] If a unit test is added, it fails before mapping change and passes after.
  - [x] `cargo test -p uc-tauri` (from `src-tauri/`) → PASS (if tests added).

---

## Commit Strategy

| After Task | Message                                             | Files                 | Verification             |
| ---------- | --------------------------------------------------- | --------------------- | ------------------------ |
| 1          | `feat(pairing): persist peer device name in core`   | uc-core files         | `cargo test -p uc-core`  |
| 2          | `feat(storage): add paired device_name persistence` | uc-infra + migrations | `cargo test -p uc-infra` |
| 3          | `fix(pairing): return persisted device names`       | uc-tauri command      | `cargo test -p uc-tauri` |

---

## Success Criteria

### Verification Commands (run from `src-tauri/`)

```bash
cargo test -p uc-core
cargo test -p uc-infra
cargo test -p uc-tauri
```

### Final Checklist

- [x] PairedDevice includes device_name and is set during pairing.
- [x] DB schema and migration include device_name.
- [x] `get_paired_peers_with_status` returns real device names.
- [x] Tests pass (TDD).
