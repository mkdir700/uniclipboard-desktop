# Cross-device Clipboard Transfer (MVP, single-machine development)

## TL;DR

> **Quick Summary**: Implement text-only cross-device clipboard transfer over the existing libp2p Business stream, gated by existing trust policy, with deterministic tests that simulate two logical devices on one machine.
>
> **Deliverables**:
>
> - App-layer outbound sync (capture/restore/manual sync → encrypt → send)
> - App-layer inbound sync (receive → decrypt → apply to local clipboard with echo protection)
> - uc-tauri background clipboard receive loop (consumes `NetworkPort::subscribe_clipboard`)
> - Tauri command `sync_clipboard_items` (frontend “Sync now” button works)
> - Dev support: multi-instance + per-profile AppDirs isolation for debugging (env-driven)
> - Automated tests (unit + integration) that validate protocol bytes, encryption wiring, and echo-loop protection
>
> **Estimated Effort**: Large
> **Parallel Execution**: YES (2 waves)
> **Critical Path**: Protocol+payload spec → uc-app usecases (send/receive) → uc-tauri wiring/commands → verification

---

## Context

### Original Request

- Design and implement cross-device clipboard content transfer.
- Constraint: only one physical device available; needs a viable single-machine development + verification approach.

### Codebase Findings (existing building blocks)

- Local capture already persists history:
  - `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs` implements `ClipboardChangeHandler::on_clipboard_changed`.
  - Calls `src-tauri/crates/uc-app/src/usecases/internal/capture_clipboard.rs`.

- Restore-to-system-clipboard exists (and already prevents recapture):
  - `src-tauri/crates/uc-app/src/usecases/clipboard/restore_clipboard_selection.rs` sets `ClipboardChangeOrigin::LocalRestore`.
  - `src-tauri/crates/uc-tauri/src/commands/clipboard.rs` has TODO(sync) after restore.

- Network port and protocol types already exist:
  - Port: `src-tauri/crates/uc-core/src/ports/network.rs` (`send_clipboard`, `subscribe_clipboard`).
  - Protocol envelope: `src-tauri/crates/uc-core/src/network/protocol/protocol_message.rs` (JSON `to_bytes/from_bytes`).
  - Clipboard message DTO: `src-tauri/crates/uc-core/src/network/protocol/clipboard.rs`.
  - libp2p adapter:
    - inbound: `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs` forwards `ProtocolMessage::Clipboard` to `subscribe_clipboard()`.
    - outbound: `send_clipboard(peer_id, bytes)` writes **raw bytes** to the Business stream (so app must send bytes that decode as `ProtocolMessage`).

- Clipboard write semantics: platform write currently expects exactly 1 representation:
  - `src-tauri/crates/uc-platform/src/clipboard/common.rs` enforces `snapshot.representations.len() == 1`.

- Loop-prevention primitives exist:
  - `src-tauri/crates/uc-core/src/clipboard/change.rs` (`ClipboardChangeOrigin::{LocalCapture, LocalRestore, RemotePush}`).
  - `src-tauri/crates/uc-infra/src/clipboard/change_origin.rs` (`InMemoryClipboardChangeOrigin`).

- Frontend already calls a missing command:
  - `src/api/clipboardItems.ts` calls `invokeWithTrace('sync_clipboard_items')`.
  - `src/components/layout/ActionBar.tsx` uses it.

### Metis Review (gaps to lock down)

- Single-machine simulation: two Tauri processes share the same OS clipboard, so “two-device clipboard independence” cannot be validated by manual dual-instance alone.
- Must-have guardrails: text-only MVP; no pairing UX changes; no crypto redesign; no offline queue/acks/retries.
- High-risk seams:
  - outbound serialization must be compatible with inbound decoding (`ProtocolMessage::Clipboard`).
  - prevent echo loops/storms: ignore self-origin messages; never broadcast `RemotePush`/`LocalRestore`; no-op apply when clipboard already matches.
- Dev collisions: need profile-based AppDirs isolation + env flag to bypass single-instance plugin.

---

## Work Objectives

### Core Objective

- Enable trusted peers to transfer clipboard **text** securely and deterministically, with a development workflow that works on a single physical machine.

### Concrete Deliverables

- New uc-app clipboard sync usecases (send + receive)
- uc-tauri background receive loop + new command `sync_clipboard_items`
- Dev multi-instance toggle + per-profile storage isolation
- Tests proving:
  - outbound bytes are decodable as clipboard protocol
  - inbound apply writes exactly one representation with `RemotePush` origin
  - restore triggers outbound sync even though capture is suppressed

### Definition of Done

- `cd src-tauri && cargo test --workspace` passes
- `bun run test` passes
- Frontend “Sync now” button no longer errors (command exists and returns expected type)
- New sync logic does not create echo loops (covered by tests)

### Must Have

- Text-only MVP (exactly one representation written)
- Works only for trusted/allowed peers (reuse existing connection policy)
- Encryption session gating:
  - If `EncryptionSessionPort` is not ready, outbound/inbound sync must be a best-effort no-op (log with context, return Ok)
- Echo safety:
  - Ignore inbound message if `origin_device_id` matches local device id
  - Skip outbound sync for `RemotePush` and `LocalRestore` origins
  - No-op apply if local clipboard already matches incoming content hash

### Must NOT Have (Guardrails)

- No non-text clipboard types (images, files, HTML, RTF) in MVP
- No changes to pairing UX or pairing protocol
- No switching protocol framing away from existing JSON envelope
- No offline message queue/ACK/retry in MVP
- No logging of decrypted clipboard payload or secrets

---

## Verification Strategy (MANDATORY)

> UNIVERSAL RULE: all verification is agent-executed (commands/tests). No “user manually tests”.

### Test Decision

- **Infrastructure exists**: YES (Rust tests + Vitest)
- **Automated tests**: YES (TDD recommended for uc-app sync usecases)
- **Frameworks**:
  - Rust: `cargo test` (run from `src-tauri/` only)
  - Frontend: `bun run test`

### Primary Verification Approach (single-machine)

- Prefer in-process dual-peer tests using mocked/in-memory `SystemClipboardPort` (two logical devices), rather than relying on the OS clipboard (shared across processes).
- Optional manual dual-instance is for debugging only (profile isolation + single-instance bypass).

---

## Execution Strategy

### Parallel Execution Waves

Wave 1 (foundation):

- Task 1 (protocol payload + AAD)
- Task 8 (multi-instance/profile isolation)

Wave 2 (sync core):

- Task 2 (outbound send usecase)
- Task 3 (inbound apply usecase)
- Task 9 (in-process dual-peer tests)

Wave 3 (wiring + commands):

- Task 4 (uc-tauri receive loop)
- Task 5 (on-capture outbound integration)
- Task 6 (sync_clipboard_items command)
- Task 7 (restore triggers sync)

Wave 4 (transport compatibility):

- Task 10 (libp2p clipboard wire integration test)

Critical Path: 1 → 2+3 → 4+5+6+7 → 10

---

## TODOs

> Notes:
>
> - Cargo commands must run from `src-tauri/`.
> - Keep boundaries strict: `uc-app → uc-core ← uc-platform/uc-infra`.
> - New Tauri event payloads (if any) must use `#[serde(rename_all = "camelCase")]`.

### 1) Define network clipboard plaintext payload + AAD helper

**What to do**:

- Define the plaintext payload schema used _inside_ `ClipboardMessage.encrypted_content` (MVP: text only).
  - Recommended: `ClipboardTextPayloadV1 { text: String, mime: "text/plain", ts_ms: i64 }`.
  - Keep it in `uc-core` as a pure protocol DTO (no policy).
- Add a new AAD generator for network clipboard encryption/decryption.
  - Location: `src-tauri/crates/uc-core/src/security/aad.rs`.
  - Format suggestion: `uc:net_clipboard:v1|{message_id}`.
- Add unit tests for determinism and input sensitivity.

**Must NOT do**:

- Do not change `ProtocolMessage` framing away from JSON.
- Do not add non-text variants.

**Recommended Agent Profile**:

- Category: `quick`
- Skills: `test-driven-development`

**Parallelization**:

- Can Run In Parallel: YES (with Task 8)

**References**:

- `src-tauri/crates/uc-core/src/network/protocol/clipboard.rs` - existing `ClipboardMessage` envelope.
- `src-tauri/crates/uc-core/src/network/protocol/protocol_message.rs` - JSON framing used on wire.
- `src-tauri/crates/uc-core/src/security/aad.rs` - existing AAD patterns (`for_inline`, `for_blob`).

**Acceptance Criteria**:

- New payload type lives under `src-tauri/crates/uc-core/src/network/protocol/` and is serde-serializable.
- `cd src-tauri && cargo test -p uc-core` → PASS.

**Agent-Executed QA Scenarios**:

```
Scenario: AAD determinism
  Tool: Bash
  Steps:
    1. cd src-tauri
    2. cargo test -p uc-core aad
  Expected Result: tests pass; AAD bytes stable across invocations
  Evidence: .sisyphus/evidence/task-1-uc-core-tests.txt
```

### 2) Implement uc-app outbound sync usecase (snapshot → encrypt → send)

**What to do**:

- Create an app-layer usecase that:
  - Input: `SystemClipboardSnapshot` + `ClipboardChangeOrigin`
  - Gating:
    - only send when origin is `LocalCapture` (and optionally explicit manual/restore actions)
    - respect settings: `settings.sync.auto_sync` (default behavior must be documented)
    - content type: MVP sends text only; if `settings.sync.content_types.text` is false AND all other content type flags are also false (current default), treat it as "text allowed" for MVP to avoid silent no-op
    - encryption session: if `EncryptionSessionPort::is_ready()` is false, no-op (do not error)
  - Selection: pick a single text/plain representation; if none, no-op.
  - Build `ClipboardMessage`:
    - `id`: new UUID
    - `content_hash`: compute from the selected text payload (recommended: snapshot hash of 1-rep snapshot)
    - `origin_device_id`: `DeviceIdentityPort`
    - `origin_device_name`: from `SettingsPort` (fallback “Unknown Device”)
    - `encrypted_content`: serialize plaintext payload → encrypt via `EncryptionPort` + `EncryptionSessionPort` + AAD → serialize `EncryptedBlob` bytes
  - Send strategy: unicast to all `NetworkPort::get_connected_peers()` via `NetworkPort::send_clipboard(peer_id, bytes)` where bytes decode as `ProtocolMessage::Clipboard`.

**Must NOT do**:

- Do not call libp2p adapter directly; only use ports.
- Do not log decrypted/plaintext.

**Recommended Agent Profile**:

- Category: `unspecified-high`
- Skills: `test-driven-development`, `systematic-debugging`

**Parallelization**:

- Can Run In Parallel: YES (with Task 3 after Task 1)

**References**:

- `src-tauri/crates/uc-core/src/ports/network.rs` - `send_clipboard`, `get_connected_peers`.
- `src-tauri/crates/uc-core/src/security/aad.rs` - use the new network-clipboard AAD helper.
- `src-tauri/crates/uc-core/src/security/model.rs` - `EncryptedBlob` container.
- Test patterns:
  - `src-tauri/crates/uc-app/src/usecases/pairing/list_connected_peers.rs` (mocking `NetworkPort`).
  - `src-tauri/crates/uc-infra/src/security/encrypting_clipboard_event_writer.rs` (encryption+session usage).

**Acceptance Criteria**:

- New tests cover:
  - sends exactly once for `LocalCapture` when a connected peer exists
  - does not send for `RemotePush` / `LocalRestore`
  - no-op when encryption session is not ready
  - outbound bytes decode as `ProtocolMessage::Clipboard(ClipboardMessage{...})`
- `cd src-tauri && cargo test -p uc-app` → PASS.

**Agent-Executed QA Scenarios**:

```
Scenario: Outbound usecase sends protocol bytes
  Tool: Bash
  Preconditions: tests added under src-tauri/crates/uc-app/tests/
  Steps:
    1. cd src-tauri
    2. cargo test -p uc-app clipboard_sync_outbound
  Expected Result: test asserts sent bytes decode to ProtocolMessage::Clipboard
  Evidence: .sisyphus/evidence/task-2-uc-app-outbound-tests.txt
```

### 3) Implement uc-app inbound apply usecase (receive → decrypt → apply)

**What to do**:

- Create an app-layer usecase that consumes `uc_core::network::ClipboardMessage` and:
  - Ignore if `origin_device_id == local_device_id`.
  - If `EncryptionSessionPort::is_ready()` is false, no-op (do not error)
  - Decrypt:
    - parse `encrypted_content` bytes into `EncryptedBlob`
    - decrypt with session master key + AAD (message id)
    - deserialize plaintext payload (text-only)
  - Build a 1-representation `SystemClipboardSnapshot` with `mime=text/plain`.
  - No-op apply if current clipboard already matches incoming `content_hash`.
  - Apply:
    - set `ClipboardChangeOrigin::RemotePush` for a short TTL via `ClipboardChangeOriginPort`
    - call `SystemClipboardPort::write_snapshot(snapshot)`

**Must NOT do**:

- Do not broadcast anything from inbound apply.
- Do not write more than 1 representation (platform enforces this).

**Recommended Agent Profile**:

- Category: `unspecified-high`
- Skills: `test-driven-development`, `systematic-debugging`

**Parallelization**:

- Can Run In Parallel: YES (with Task 2 after Task 1)

**References**:

- `src-tauri/crates/uc-platform/src/clipboard/common.rs` - write expects exactly one representation.
- `src-tauri/crates/uc-core/src/ports/clipboard/clipboard_change_origin.rs` - origin TTL contract.
- `src-tauri/crates/uc-infra/src/clipboard/change_origin.rs` - in-memory origin implementation.
- Test patterns:
  - `src-tauri/crates/uc-platform/src/runtime/runtime.rs` (test `SystemClipboardPort` impl capturing writes).

**Acceptance Criteria**:

- Tests cover:
  - valid inbound message applies exactly one text/plain snapshot
  - sets origin to `RemotePush` before write
  - no-op when clipboard already matches
  - ignores self-origin messages
  - no-op when encryption session is not ready
- `cd src-tauri && cargo test -p uc-app` → PASS.

**Agent-Executed QA Scenarios**:

```
Scenario: Inbound apply writes snapshot with RemotePush origin
  Tool: Bash
  Steps:
    1. cd src-tauri
    2. cargo test -p uc-app clipboard_sync_inbound
  Expected Result: tests pass; write_snapshot called once; origin set to RemotePush
  Evidence: .sisyphus/evidence/task-3-uc-app-inbound-tests.txt
```

### 4) Wire uc-tauri background clipboard receive loop

**What to do**:

- In `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`, add a new background task started from `start_background_tasks`:
  - call `deps.network.subscribe_clipboard()` exactly once
  - loop over receiver; for each message call the uc-app inbound apply usecase
  - handle channel close gracefully

**Must NOT do**:

- Do not subscribe to `subscribe_events()` a second time (receiver is single-take in libp2p adapter).
- No direct DB mutations from bootstrap.

**Recommended Agent Profile**:

- Category: `quick`
- Skills: `systematic-debugging`

**Parallelization**:

- Can Run In Parallel: NO (depends on Task 3 existing)

**References**:

- `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs` - existing loop patterns (`run_pairing_event_loop`).
- `src-tauri/crates/uc-core/src/ports/network.rs` - `subscribe_clipboard()` semantics.
- `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs` - `subscribe_clipboard` uses take-once receiver.

**Acceptance Criteria**:

- `cd src-tauri && cargo check -p uc-tauri` → PASS.
- `cd src-tauri && cargo test -p uc-tauri` → PASS.

**Agent-Executed QA Scenarios**:

```
Scenario: uc-tauri builds with clipboard loop wired
  Tool: Bash
  Steps:
    1. cd src-tauri
    2. cargo test -p uc-tauri
  Expected Result: tests pass; no deadlocks on startup
  Evidence: .sisyphus/evidence/task-4-uc-tauri-tests.txt
```

### 5) Integrate outbound sync into clipboard change callback (LocalCapture only)

**What to do**:

- In `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs` inside `on_clipboard_changed`:
  - keep existing capture pipeline
  - after successful capture, invoke outbound sync usecase with the same `snapshot` and the resolved `origin`
  - ensure outbound is a no-op for `RemotePush` and `LocalRestore`

**Must NOT do**:

- Do not block the clipboard watcher thread with long network waits (use async, best-effort).

**Recommended Agent Profile**:

- Category: `unspecified-low`
- Skills: `systematic-debugging`

**Parallelization**:

- Can Run In Parallel: NO (depends on Task 2)

**References**:

- `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs` - current callback and origin consumption.
- `src-tauri/crates/uc-app/src/usecases/internal/capture_clipboard.rs` - capture behavior and LocalRestore skip.

**Acceptance Criteria**:

- `cd src-tauri && cargo check -p uc-tauri` → PASS.
- New/updated tests prove RemotePush does not cause broadcast.

**Agent-Executed QA Scenarios**:

```
Scenario: RemotePush origin does not broadcast
  Tool: Bash
  Steps:
    1. cd src-tauri
    2. cargo test -p uc-tauri clipboard
  Expected Result: tests pass; outbound sync is not invoked for RemotePush
  Evidence: .sisyphus/evidence/task-5-uc-tauri-clipboard-tests.txt
```

### 6) Implement Tauri command: sync_clipboard_items (manual push)

**What to do**:

- Add `#[tauri::command] sync_clipboard_items` in `src-tauri/crates/uc-tauri/src/commands/clipboard.rs`.
- Register it in `src-tauri/src/main.rs` `generate_handler![]` list.
- Semantics (MVP): read current clipboard snapshot and invoke outbound sync usecase (best-effort). Return `bool` for frontend.
  - Recommended return: `true` on best-effort completion (including no peers / session not ready), `Err(String)` only on unexpected failures.

**Must NOT do**:

- Do not attempt “sync all history”. Keep MVP = push current clipboard.

**Recommended Agent Profile**:

- Category: `quick`
- Skills: `test-driven-development`

**Parallelization**:

- Can Run In Parallel: YES (after Task 2; parallel with Task 7)

**References**:

- Frontend call site: `src/api/clipboardItems.ts` (`syncClipboardItems`).
- UI trigger: `src/components/layout/ActionBar.tsx`.
- Command wiring: `src-tauri/src/main.rs` `generate_handler![]`.

**Acceptance Criteria**:

- Frontend build still passes: `bun run build`.
- Rust builds/tests pass: `cd src-tauri && cargo test -p uc-tauri`.

**Agent-Executed QA Scenarios**:

```
Scenario: Command is registered
  Tool: Bash
  Steps:
    1. cd src-tauri
    2. cargo test -p uc-tauri commands
  Expected Result: tests pass; sync_clipboard_items symbol is reachable
  Evidence: .sisyphus/evidence/task-6-command-tests.txt
```

### 7) Make restore propagate to peers (explicit send)

**What to do**:

- In `src-tauri/crates/uc-tauri/src/commands/clipboard.rs` restore path:
  - After `restore_uc.restore_snapshot(snapshot)` succeeds, call outbound sync usecase with origin `LocalRestore` (explicitly allowed for this flow).
  - Ensure this does not re-capture locally (restore already sets `LocalRestore`).

**Must NOT do**:

- Do not rely on capture callback to sync restore (capture intentionally skips LocalRestore).

**Recommended Agent Profile**:

- Category: `unspecified-low`
- Skills: `test-driven-development`

**Parallelization**:

- Can Run In Parallel: YES (after Task 2; parallel with Task 6)

**References**:

- `src-tauri/crates/uc-tauri/src/commands/clipboard.rs` - TODO(sync) marker.
- `src-tauri/crates/uc-app/src/usecases/clipboard/restore_clipboard_selection.rs` - origin TTL.

**Acceptance Criteria**:

- New test proves restore triggers outbound send even though capture is suppressed.
- `cd src-tauri && cargo test -p uc-tauri` → PASS.

**Agent-Executed QA Scenarios**:

```
Scenario: Restore triggers sync
  Tool: Bash
  Steps:
    1. cd src-tauri
    2. cargo test -p uc-tauri restore
  Expected Result: tests pass; outbound send observed
  Evidence: .sisyphus/evidence/task-7-restore-sync-tests.txt
```

### 8) Enable single-machine multi-instance debugging (env-driven)

**What to do**:

- Add an env flag to bypass `tauri_plugin_single_instance` in `src-tauri/src/main.rs`.
  - Suggested: `UC_DISABLE_SINGLE_INSTANCE=1`.
- Add per-profile AppDirs isolation:
  - Suggested: `UC_PROFILE=peerA|peerB` appends to both `app_data_root` and `app_cache_root`.
  - Implement in `src-tauri/crates/uc-platform/src/app_dirs.rs` so both main + wiring share it.

**Must NOT do**:

- Do not change default production behavior when env vars are unset.

**Recommended Agent Profile**:

- Category: `quick`
- Skills: `systematic-debugging`

**Parallelization**:

- Can Run In Parallel: YES (with Task 1)

**References**:

- Single-instance plugin site: `src-tauri/src/main.rs`.
- AppDirs adapter: `src-tauri/crates/uc-platform/src/app_dirs.rs`.
- Default path wiring: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs` (`derive_default_paths`).

**Acceptance Criteria**:

- Unit test demonstrates `UC_PROFILE=a` and `UC_PROFILE=b` produce different dirs.
- `cd src-tauri && cargo test -p uc-platform` → PASS.

**Agent-Executed QA Scenarios**:

```
Scenario: Profile isolation test
  Tool: Bash
  Steps:
    1. cd src-tauri
    2. cargo test -p uc-platform app_dirs
  Expected Result: tests pass; app_data_root differs per profile
  Evidence: .sisyphus/evidence/task-8-uc-platform-tests.txt
```

### 9) Add in-process dual-peer sync tests (single-machine correctness)

**What to do**:

- Add a test harness that simulates two logical devices without OS clipboard:
  - Two `SystemClipboardPort` in-memory instances
  - A fake `NetworkPort` that routes outbound bytes from A to B by decoding `ProtocolMessage` and delivering `ClipboardMessage` to B
  - Assert: A capture triggers send; B receive triggers apply; B does not re-send.

**Must NOT do**:

- Do not require launching two GUI processes.

**Recommended Agent Profile**:

- Category: `unspecified-high`
- Skills: `test-driven-development`

**Parallelization**:

- Can Run In Parallel: NO (depends on Tasks 2 and 3)

**References**:

- `src-tauri/crates/uc-core/src/network/protocol/protocol_message.rs` - decode bytes in fake network.
- `src-tauri/crates/uc-platform/src/runtime/runtime.rs` test `SystemClipboardPort` pattern.

**Acceptance Criteria**:

- `cd src-tauri && cargo test -p uc-app clipboard_sync_e2e` → PASS.

**Agent-Executed QA Scenarios**:

```
Scenario: E2E in-process dual-peer sync
  Tool: Bash
  Steps:
    1. cd src-tauri
    2. cargo test -p uc-app clipboard_sync_e2e
  Expected Result: tests pass; inbound apply results in one write_snapshot on peer B
  Evidence: .sisyphus/evidence/task-9-e2e-tests.txt
```

### 10) Add libp2p adapter clipboard wire compatibility test

**What to do**:

- In `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs` tests:
  - Start two adapters’ swarms
  - Wait for discovery
  - Send a `ProtocolMessage::Clipboard(...)` bytes from A to B using `send_clipboard(peer_id, bytes)`
  - Assert B’s `subscribe_clipboard()` receiver yields the expected `ClipboardMessage`

**Must NOT do**:

- Do not expand protocol features (still JSON framing).

**Recommended Agent Profile**:

- Category: `unspecified-low`
- Skills: `systematic-debugging`

**Parallelization**:

- Can Run In Parallel: YES (late; after core sync is stable)

**References**:

- Existing mdns e2e test: `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs` (`mdns_e2e_discovers_peers`).
- Inbound payload handling: `handle_business_payload` in same file.

**Acceptance Criteria**:

- `cd src-tauri && cargo test -p uc-platform libp2p_network` → PASS.

**Agent-Executed QA Scenarios**:

```
Scenario: libp2p clipboard message roundtrip
  Tool: Bash
  Steps:
    1. cd src-tauri
    2. cargo test -p uc-platform libp2p_network::tests::
  Expected Result: tests pass; receiver yields ClipboardMessage
  Evidence: .sisyphus/evidence/task-10-libp2p-tests.txt
```

---

## Commit Strategy (Atomic, boundary-safe)

- Commit A: `arch:` or `impl:` add protocol payload DTO + network AAD helper (uc-core only)
- Commit B: `impl:` outbound sync usecase + tests (uc-app only)
- Commit C: `impl:` inbound apply usecase + tests (uc-app only)
- Commit D: `impl:` uc-tauri wiring + commands (`sync_clipboard_items`, restore propagation)
- Commit E: `chore:` dev multi-instance/profile isolation (src-tauri main + uc-platform app_dirs)
- Commit F: `test:` libp2p wire compatibility test (uc-platform only)

Each commit must respect repo rules: never mix `uc-core` with `uc-platform/uc-infra` in the same commit.

---

## Success Criteria

### Verification Commands

```bash
# from repo root
bun run test

# from src-tauri/
cargo test --workspace
```

### Final Checklist

- [x] `sync_clipboard_items` command exists and matches frontend expectations (`Promise<boolean>`)
- [x] Outbound sends `ProtocolMessage::Clipboard` bytes over `NetworkPort::send_clipboard`
- [x] Inbound applies to system clipboard with `RemotePush` origin and no echo broadcast
- [x] Tests cover: outbound encode, inbound apply, restore-triggered sync, in-process dual-peer simulation
- [x] No non-text clipboard sync in MVP
