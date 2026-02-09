- Deferred final paired-device commit by changing pairing finalization persistence from `Trusted` to `Pending` in `pairing_state_machine`.
- Kept verification-complete commit point at `ProofVerified -> Granted` through `SpaceAccessAction::PersistSponsorAccess`, which promotes `Pending` to `Trusted`.
- Added regression coverage: core now asserts pending persistence action, and app verifies promotion to trusted only after sponsor proof verification persistence path.
- Replaced pre-verification repo write in pairing orchestrator with in-memory staging (`staged_paired_device_store`) and immediate `PersistOk`, so pairing domain events keep flowing without DB commit side effects.
- Moved the actual commit to verification boundary by consuming staged device in `SpaceAccessPersistenceAdapter::persist_sponsor_access`, promoting it to `Trusted` via `upsert` only after proof-verification success.
- Added tests proving both semantics: no pre-verification `upsert` on `PersistPairedDevice`, and staged pending device is committed as trusted at proof-complete boundary.

- Busy pairing message handling now uses injected runtime `SpaceAccessNetworkAdapter` + `HmacProofAdapter` + `SpaceAccessPersistenceAdapter` via a shared executor path, removing production `NoopSpaceAccessProof/Transport/Persistence` instantiation from busy `Offer/Proof/Result` branches.
- `dispatch_space_access_busy_event` now returns `Result<(), SpaceAccessError>` to align with `SpaceAccessOrchestrator::dispatch` and avoid cross-type error coercion bugs during targeted test builds.

- 2026-02-09: Busy-path runtime wiring now injects a shared `RuntimeSpaceAccessPorts` bundle (real `SpaceAccessNetworkAdapter`, `HmacProofAdapter`, `SpaceAccessPersistenceAdapter`, and timer) from `start_background_tasks` into pairing event/action loops instead of creating Noop ports inline.
- 2026-02-09: `handle_pairing_message` Offer/Proof/Result branches now dispatch through one helper (`dispatch_space_access_busy_event`) that consistently builds the executor from injected runtime ports.
- 2026-02-09: Busy proof routing test now validates both paths: invalid nonce keeps `WaitingJoinerProof`; valid nonce routes proof branch and transitions to `Denied(InvalidProof)` with real proof adapter wiring.

- 2026-02-09: Verification matrix passed from `src-tauri/`: `cargo check`, `cargo test -p uc-core pairing_state_machine -- --nocapture`, `cargo test -p uc-app pairing -- --nocapture`, `cargo test -p uc-app setup::orchestrator -- --nocapture`, `cargo test -p uc-tauri busy_proof_payload_routes_to_proof_branch_and_validates_nonce_length -- --nocapture`, `cargo test -p uc-tauri pairing_action_loop_starts_sponsor_authorization_for_responder_role -- --nocapture`, `cargo test -p uc-tauri pairing_action_loop_closes_session_only_for_failed_emit_result -- --nocapture`, and `cargo test -p uc-tauri bootstrap -- --nocapture`.
- 2026-02-09: Exhaustive search confirms no `NoopSpaceAccessProof/Transport/Persistence` usage remains in production busy path (`uc-tauri/bootstrap/wiring.rs`); remaining noop ports are test scaffolds in `uc-app` test modules.
- 2026-02-09: Project-level Rust LSP diagnostics are unavailable in current toolchain (no rust-analyzer server configured), so `cargo check` is used as the authoritative compile-time gate.
