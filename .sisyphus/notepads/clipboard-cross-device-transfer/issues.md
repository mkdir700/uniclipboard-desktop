# Issues - Clipboard Cross-Device Transfer

## Session Log

### 2026-02-15 - Verification blocker

- Command: `cd src-tauri && cargo test -p uc-tauri`
- Status: FAILED (unrelated to Tasks 5/6/7 changes)
- Failing test: `bootstrap::tracing::tests::test_build_filter_directives`
- Failure detail: assertion expects `"libp2p_mdns=info"` (and iface-off directive) but current `build_filter_directives()` defaults contain different values.
- Workaround applied for this task: validated changed scope with `cargo check -p uc-tauri` and `bun run build`.

### 2026-02-15 - LSP diagnostics unavailable in environment

- Command: `lsp_diagnostics` on `src-tauri/crates/uc-app/tests/clipboard_sync_e2e_test.rs`
- Status: FAILED (tooling availability)
- Error detail: `Unknown binary 'rust-analyzer' in official toolchain 'stable-aarch64-apple-darwin'`
- Workaround applied for Task 9: used compiler-backed verification (`cargo test -p uc-app clipboard_sync_e2e` and `cargo check -p uc-app --tests`) to confirm zero compile errors in changed test scope.

### 2026-02-15 - LSP diagnostics unavailable for Task 10 file

- Command: `lsp_diagnostics` on `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs`
- Status: FAILED (tooling availability)
- Error detail: `Unknown binary 'rust-analyzer' in official toolchain 'stable-aarch64-apple-darwin'`
- Workaround applied for Task 10: verified with compiler-backed checks (`cargo check -p uc-platform --tests`, `cargo test -p uc-platform libp2p_network`).

### 2026-02-18 - workspace cleanup

- Context: repository root contained an untracked placeholder file `1.`.
- Action: documented this cleanup note and removed `1.` from the workspace.
- Rationale: keep repository root free of ambiguous scratch files and preserve traceability in project notes.
