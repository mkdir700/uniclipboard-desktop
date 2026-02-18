# Learnings - Clipboard Cross-Device Transfer

## Session Log

### 2026-02-14 - Session Start

- Plan: Cross-device Clipboard Transfer (MVP)
- Total Tasks: 10
- Starting from: Task 1 (protocol payload + AAD)

## Codebase Conventions

### Cargo Commands

- All Rust commands must run from `src-tauri/`

### Hexagonal Architecture Boundaries

- uc-app → uc-core ← uc-platform/uc-infra
- uc-core must NOT depend on external implementations

### Serialization

- All Tauri event payloads must use `#[serde(rename_all = "camelCase")]`

### Protocol Framing

- Uses JSON envelope via `ProtocolMessage::to_bytes/from_bytes`
- Clipboard messages wrapped in `ClipboardMessage` envelope

## Key Existing Components

1. ClipboardMessage envelope: `src-tauri/crates/uc-core/src/network/protocol/clipboard.rs`
2. Network ports: `src-tauri/crates/uc-core/src/ports/network.rs`
3. AAD helpers: `src-tauri/crates/uc-core/src/security/aad.rs`
4. Encryption model: `src-tauri/crates/uc-core/src/security/model.rs`

## Guardrails

- Text-only MVP (no images, files, HTML, RTF)
- Echo loop prevention via origin checks
- Encryption session gating (best-effort no-op if not ready)

### 2026-02-14 - Task 1 (protocol payload + AAD)

- Added protocol DTO `ClipboardTextPayloadV1` under `uc-core/src/network/protocol/` with fields `{ text, mime, ts_ms }` and serde derive.
- Kept network clipboard AAD format aligned with existing pattern using `uc:net_clipboard:v1|{message_id}`.
- Added deterministic and input-sensitivity unit tests for `for_network_clipboard` in `security/aad.rs`.

### 2026-02-14 - Task 1 verification follow-up

- `cargo check -p uc-core` passes from `src-tauri/`.
- `cargo test -p uc-core` still has one unrelated pre-existing failure in `setup::state_machine::tests::setup_state_machine_table_driven`.
- Focused tests pass: `security::aad::tests` and `network::protocol::clipboard_payload::tests`.

### 2026-02-14 - app_dirs test isolation fix

- Root cause: only one test used `with_uc_profile` locking/restore, so parallel tests observed transient `UC_PROFILE` values.
- Fix: wrapped default-path tests with `with_uc_profile(None, ...)` so all app_dirs tests participate in the same env lock and start from clean env.
- Verification: `cargo test -p uc-platform app_dirs` passes with all three app_dirs tests green.

### 2026-02-14 - Task 2 (uc-app outbound sync usecase)

- Added `usecases/clipboard/sync_outbound.rs` with sync `execute(snapshot, origin)` that internally drives async port calls via `futures::executor::block_on`.
- Outbound gating is strict for `ClipboardChangeOrigin::LocalCapture`; `RemotePush` and `LocalRestore` paths return no-op.
- Session readiness is checked first via `EncryptionSessionPort::is_ready()` and returns no-op with context log when unavailable.
- Text-only selection uses a single `text/plain` representation, serializes `ClipboardTextPayloadV1`, encrypts with `aad::for_network_clipboard(message_id)`, and wraps outbound bytes as JSON `ProtocolMessage::Clipboard`.
- Added unit tests covering send-on-local-capture, no-send for remote/restore, no-op when session not ready, and protocol decode verification for outbound bytes.
- Verification: `cargo test -p uc-app sync_outbound::tests`, `cargo check -p uc-app`, and full `cargo test -p uc-app` all pass from `src-tauri/`.

### 2026-02-14 - Task 3 (uc-app inbound apply usecase)

- Added `usecases/clipboard/sync_inbound.rs` with inbound flow: self-origin guard -> session-ready guard -> encrypted blob parse -> decrypt with `aad::for_network_clipboard(message_id)` -> `ClipboardTextPayloadV1` decode -> dedupe by `content_hash` -> apply one `text/plain` representation.
- Apply path sets `ClipboardChangeOrigin::RemotePush` with short TTL (`100ms`) before `SystemClipboardPort::write_snapshot`, and clears staged origin via `consume_origin_or_default` if write fails.
- Dedupe compares incoming envelope `content_hash` against current local snapshot representation hashes to no-op duplicate inbound messages.
- Added tests for: valid apply with exactly one `text/plain` representation, origin set before write, no-op on local hash match, self-origin ignore, and session-not-ready no-op.
- Verification: `cargo test -p uc-app sync_inbound`, `cargo check -p uc-app`, and full `cargo test -p uc-app` pass from `src-tauri/`.

### 2026-02-15 - Task 4 (uc-tauri background receive loop)

- Wired a dedicated clipboard receive background task in `start_background_tasks` that subscribes via `deps.network.subscribe_clipboard()` once and processes incoming messages.
- Added `run_clipboard_receive_loop` to consume the receiver until close, execute `SyncInboundClipboardUseCase` per message, and log graceful shutdown when the channel closes.
- Added tracing spans for task and per-message processing (`loop.clipboard.receive_task`, `loop.clipboard.receive_message`) to keep inbound flow observable.
- Verification: `cd src-tauri && cargo check -p uc-tauri` passes.

### 2026-02-15 - Tasks 5/6/7 (runtime + commands outbound integration)

- Added `UseCases::sync_outbound_clipboard()` in `uc-tauri` runtime accessor to centralize command/runtime wiring for outbound sync usecase construction.
- Runtime clipboard callback (`on_clipboard_changed`) now reuses the consumed origin and captured snapshot for outbound sync after successful capture; outbound send is dispatched via `tauri::async_runtime::spawn` + `tokio::task::spawn_blocking` to avoid blocking clipboard watcher flow.
- Added Tauri command `sync_clipboard_items` in `commands/clipboard.rs`: reads current system clipboard snapshot and executes outbound sync with `ClipboardChangeOrigin::LocalCapture` in best-effort mode; command always returns `Ok(true)` and logs failures.
- Restore flow (`restore_clipboard_entry_impl`) now invokes outbound sync after successful `restore_snapshot`, using origin `ClipboardChangeOrigin::LocalRestore` (which currently no-ops in outbound usecase by policy), and keeps restore success semantics unchanged.
- Registered `uc_tauri::commands::clipboard::sync_clipboard_items` in `src-tauri/src/main.rs` `generate_handler![]` list.
- Verification: `cargo check -p uc-tauri` passes; `bun run build` passes; `cargo test -p uc-tauri` has one unrelated pre-existing failing test in `bootstrap::tracing::tests::test_build_filter_directives`.

### 2026-02-15 - Task 9 (in-process dual-peer sync tests)

- Added `src-tauri/crates/uc-app/tests/clipboard_sync_e2e_test.rs` with a dual-device in-process harness (A/B) that avoids OS clipboard usage.
- Implemented two in-memory `SystemClipboardPort` instances plus fake `NetworkPort` endpoints that decode outbound bytes with `ProtocolMessage::from_bytes` and route `ClipboardMessage` to the peer's `SyncInboundClipboardUseCase`.
- Verified end-to-end flow: A `LocalCapture` outbound send triggers B inbound apply (`write_snapshot` once), and B-origin `RemotePush` consumption prevents re-send when B outbound is executed.
- Verification: `cd src-tauri && cargo test -p uc-app clipboard_sync_e2e` passes; `cd src-tauri && cargo check -p uc-app --tests` passes.

### 2026-02-15 - Task 10 (libp2p clipboard wire compatibility test)

- Added `libp2p_network_clipboard_wire_roundtrip_delivers_clipboard_message` in `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs` tests.
- Test boots two adapters, starts both swarms, waits for mdns discovery, opens a pairing session for connectivity warmup, sends `ProtocolMessage::Clipboard(...)` bytes via `send_clipboard(peer_id, bytes)`, and asserts `subscribe_clipboard()` on peer B receives the expected `ClipboardMessage` fields.
- Kept protocol framing unchanged (JSON envelope via `ProtocolMessage::to_bytes/from_bytes`).
- Verification: `cd src-tauri && cargo test -p uc-platform libp2p_network` passes.

### 2026-02-15 - Task 10 re-verification

- Re-ran `cd src-tauri && cargo test -p uc-platform libp2p_network`; all 26 `libp2p_network` tests passed, including `libp2p_network_clipboard_wire_roundtrip_delivers_clipboard_message`.

### 2026-02-16 - Dual-instance local-path isolation fix

- Root cause of one-instance unlock failure + mixed directories: app-data override from `config.toml` was not used as `app_data_root` during wiring, so identity/secure-storage-dependent paths still pointed to platform defaults.
- Updated `derive_default_paths_from_app_dirs` to derive `app_data_root` from configured `database_path` parent when present.
- Added profile suffix support for configured roots via `UC_PROFILE`, producing `.app_data_a` / `.app_data_b` from base `.app_data`.
- Adjusted db/settings/vault paths to be rooted under the resolved profile-aware `app_data_root`.
- Added config-path discovery fallback for repo-root runs: `resolve_config_path()` now also checks `src-tauri/config.toml` and supports explicit `UC_CONFIG_PATH`.
- Updated bun dual scripts to use `UC_PROFILE=a|b` so peer roots map cleanly to `.app_data_a` / `.app_data_b`.

### 2026-02-16 - Root cause for peerB unlock-after-delete

- The startup gate to Setup vs Unlock is driven by files under `vault_dir` (`setup_status.json`, `encryption_state.json`, `keyslot.json`) rather than only database presence.
- With `src-tauri/config.toml` configured as `.app_data/*`, `database_path` was profile-isolated but explicit `vault_key_path` stayed unsuffixed, so peerA/peerB shared the same vault state.
- Deleting only `.app_data_b` did not remove shared setup/encryption state in `.app_data/vault`, causing peerB to continue entering unlock flow.
- Fixed by remapping configured `vault_key_path` root to the resolved profile-specific `app_data_root` when vault path is under configured database root.
