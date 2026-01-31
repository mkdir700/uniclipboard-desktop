# Pairing Initiator Timeout Logging Plan

## TL;DR

> **Quick Summary**: Add targeted, structured tracing logs across pairing action dispatch, stream framing read/write, and session close paths to pinpoint whether `PairingConfirm` is sent and where early EOF occurs. No behavior changes.
>
> **Deliverables**:
>
> - Frame-stage logs for pairing stream read/write (prefix vs body)
> - Action-loop send/close logs with session/peer identifiers
> - Session close reason logs across network adapter and pairing stream
>
> **Estimated Effort**: Short
> **Parallel Execution**: YES - 2 waves
> **Critical Path**: Task 1 → Task 2 → Task 3

---

## Context

### Original Request

User observed initiator timing out at `WaitingConfirm` with early EOF, while responder shows success. They want to add more logs first to diagnose root cause without changing behavior.

### Interview Summary

**Key Discussions**:

- Initiator logs: `pairing read failed ... early eof` then `Timeout(WaitingConfirm)`.
- Responder logs: accepted response, Finalizing, persisted device OK, but no explicit confirm-send log.
- Suspected risk: confirm send is enqueued then session closes before write loop flushes; length-prefixed write can be canceled mid-frame.
- Test strategy: manual verification only (no new automated tests).

**Research Findings**:

- Confirm send path: state machine → orchestrator → wiring action loop → network send → pairing_stream write_loop.
- `pairing_stream` read/write loops log failures but not stage-level details; EOF closes session without notifying orchestrator.
- `EmitResult` in wiring triggers `close_pairing_session` immediately.
- Logging conventions: `tracing` with structured fields and spans; avoid secrets.

### Metis Review

**Identified Gaps (addressed in plan)**:

- Guardrails: log-only, no behavior/timing/timeout changes; avoid sensitive payloads.
- Scope creep risks: no retries, no protocol changes, no close-order changes.
- Acceptance criteria: ensure compile success and log markers exist.

---

## Work Objectives

### Core Objective

Instrument the pairing flow with structured logs that reveal whether `PairingConfirm` was sent and whether early EOF occurred while reading length prefix vs body, without changing runtime behavior.

### Concrete Deliverables

- Stage-specific read/write logs in pairing framing and service loops
- Send/close logs in action loop with session/peer/message kind
- Close-reason logs for pairing sessions

### Definition of Done

- [ ] New logs compile with `cargo check` in `src-tauri/`
- [ ] Logs include `session_id` and `peer_id` fields at send, write, read, and close stages
- [ ] Read errors differentiate `len_prefix` vs `payload` stage
- [ ] No behavior changes (no retries, delays, close ordering, timeout tweaks)

### Must Have

- Structured `tracing` logs (no `println!`)
- Consistent fields: `session_id`, `peer_id`, `stage`, `reason` (where applicable)
- Log level defaults: `info` for lifecycle, `debug` for stages, `warn` for IO errors
- No sensitive payloads (PIN/hash/message content)

### Must NOT Have (Guardrails)

- No protocol changes, retries, or flush/close behavior changes
- No timeout adjustments
- No UI or frontend changes
- No logging of secrets or full payloads

---

## Verification Strategy (MANDATORY)

### Test Decision

- **Infrastructure exists**: YES (Rust `cargo test` + Vitest)
- **User wants tests**: Manual-only (logging-only change)
- **Framework**: Rust `cargo` (no new tests added)

### Manual Verification (no new tests)

**Compile check (agent-executable):**

```bash
# Run from src-tauri/ per project rule
cargo check -p uc-platform -p uc-app -p uc-tauri
# Expected: exit code 0
```

**Static log presence (agent-executable):**

```bash
rg "pairing\.stream\.(read|write|close)" src-tauri/crates/uc-platform
rg "pairing\.action\.(send|close)" src-tauri/crates/uc-tauri
# Expected: matches found with stage/field markers
```

**Runtime verification (manual):**

1. Run pairing between two devices.
2. Confirm logs show `pairing.action.send` for `PairingConfirm` on responder.
3. Confirm read logs on initiator show `stage=len_prefix` or `stage=payload` on EOF, plus close reason.

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately):
├── Task 1: Action-loop send/close logs (uc-tauri wiring)
└── Task 2: Framing read/write stage logs (uc-platform framing)

Wave 2 (After Wave 1):
└── Task 3: Session lifecycle + close reason logs (pairing_stream service + libp2p adapter)

Critical Path: Task 1 → Task 2 → Task 3
Parallel Speedup: ~30% faster than sequential
```

### Dependency Matrix

| Task | Depends On | Blocks | Can Parallelize With |
| ---- | ---------- | ------ | -------------------- |
| 1    | None       | 3      | 2                    |
| 2    | None       | 3      | 1                    |
| 3    | 1, 2       | None   | None                 |

---

## TODOs

### Task 1: Add send/close logs in pairing action loop

**What to do**:

- In `run_pairing_action_loop`, add `tracing` logs before/after sending pairing messages.
- Log `session_id`, `peer_id`, `message_kind`, and `stage=enqueue|send_result`.
- Log when `EmitResult` triggers `close_pairing_session`, including `reason` if present.

**Must NOT do**:

- Do not reorder actions or introduce delays/retries.

**Recommended Agent Profile**:

- **Category**: `quick`
  - Reason: single-file log instrumentation with minimal logic
- **Skills**: `systematic-debugging`, `verification-before-completion`
  - `systematic-debugging`: ensure evidence-oriented logging points
  - `verification-before-completion`: ensure compile/log checks run
- **Skills Evaluated but Omitted**:
  - `test-driven-development`: manual-only verification requested

**Parallelization**:

- **Can Run In Parallel**: YES
- **Parallel Group**: Wave 1 (with Task 2)
- **Blocks**: Task 3
- **Blocked By**: None

**References**:

- `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs` - `run_pairing_action_loop` action handling; currently logs send failures and closes session on `EmitResult`.
- `src-tauri/crates/uc-platform/src/adapters/network.rs` - Network port interface for `close_pairing_session` signature.

**Acceptance Criteria**:

- [x] `tracing` logs added for send/close stages with `session_id` and `peer_id` fields
- [x] `cargo check -p uc-tauri` succeeds in `src-tauri/`

---

### Task 2: Add framing stage logs for read/write (prefix vs payload)

**What to do**:

- In framing module, log stage-specific events for read/write:
  - `stage=read_len_prefix`, `stage=read_payload`, `stage=write_len_prefix`, `stage=write_payload`
- On `UnexpectedEof`, log which stage failed and expected length.
- Log only metadata (lengths, stage), never payloads.

**Must NOT do**:

- Do not change framing logic, buffer sizes, or IO behavior.

**Recommended Agent Profile**:

- **Category**: `unspecified-low`
  - Reason: low-risk instrumentation across a small module
- **Skills**: `systematic-debugging`, `verification-before-completion`
- **Skills Evaluated but Omitted**:
  - `test-driven-development`: manual-only verification requested

**Parallelization**:

- **Can Run In Parallel**: YES
- **Parallel Group**: Wave 1 (with Task 1)
- **Blocks**: Task 3
- **Blocked By**: None

**References**:

- `src-tauri/crates/uc-platform/src/adapters/pairing_stream/framing/mod.rs` - `read_length_prefixed` / `write_length_prefixed` framing IO.

**Acceptance Criteria**:

- [x] Logs differentiate prefix vs payload stages for both read and write
- [x] `cargo check -p uc-platform` succeeds in `src-tauri/`

---

### Task 3: Add session lifecycle + close reason logs

**What to do**:

- In pairing stream service, log session start, shutdown reason, and which loop triggered shutdown (read/write/timeout/explicit close).
- Ensure close logs include `session_id`, `peer_id`, and `reason` (if provided).
- Propagate or log close reasons at adapter boundary (`libp2p_network` → `pairing_stream`).

**Must NOT do**:

- Do not modify close behavior or timing.

**Recommended Agent Profile**:

- **Category**: `unspecified-low`
  - Reason: small changes across service boundaries
- **Skills**: `systematic-debugging`, `verification-before-completion`
- **Skills Evaluated but Omitted**:
  - `test-driven-development`: manual-only verification requested

**Parallelization**:

- **Can Run In Parallel**: NO
- **Parallel Group**: Wave 2 (after Tasks 1 & 2)
- **Blocks**: None
- **Blocked By**: Tasks 1, 2

**References**:

- `src-tauri/crates/uc-platform/src/adapters/pairing_stream/service.rs` - session spawn/close, read/write loops, existing warn logs.
- `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs` - `close_pairing_session` adapter boundary.

**Acceptance Criteria**:

- [x] Close logs include `session_id`, `peer_id`, and `reason` when present
- [x] `cargo check -p uc-platform -p uc-app` succeeds in `src-tauri/`

---

## Commit Strategy

No commits requested. If needed later, group all logging changes into a single commit with an English message.

---

## Success Criteria

### Verification Commands

```bash
# Run from src-tauri/
cargo check -p uc-platform -p uc-app -p uc-tauri
rg "pairing\.stream\.(read|write|close)" src-tauri/crates/uc-platform
rg "pairing\.action\.(send|close)" src-tauri/crates/uc-tauri
```

### Final Checklist

- [ ] All required log points present (send, write, read, close)
- [ ] Logs include `session_id` and `peer_id`
- [ ] No sensitive payloads logged
- [ ] Compile checks pass
