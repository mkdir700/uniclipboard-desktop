# Secure Storage + Unlock-Gated Network Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Finish the SecureStoragePort migration and enforce unlock-gated identity/network startup without any secure storage reads before unlock success.

**Architecture:** Clean up remaining KeyringPort references, ensure a single SecureStoragePort instance is wired through uc-tauri, then move libp2p identity loading to the unlock-success path (use case + command). Startup must not touch secure storage; network start is triggered only after unlock success.

**Tech Stack:** Rust, Tauri, uc-core/uc-platform/uc-infra/uc-app crates, keyring crate, mockall tests.

---

### Task 1: Remove remaining KeyringPort references

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`
- Modify: `src-tauri/crates/uc-tauri/src/commands/clipboard.rs`
- Modify: `src-tauri/crates/uc-tauri/tests/bootstrap_integration_test.rs`
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`
- Modify: `src-tauri/crates/uc-platform/src/secure_storage.rs`
- Modify: `src-tauri/crates/uc-core/src/ports/security/mod.rs` (if stale exports)
- Delete or keep unused: `src-tauri/crates/uc-platform/src/keyring.rs`
- Delete or keep unused: `src-tauri/crates/uc-platform/src/file_keyring.rs`
- Delete or keep unused: `src-tauri/crates/uc-core/src/ports/security/keyring.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn wiring_exposes_secure_storage_not_keyring() {
    let config = AppConfig::empty();
    let (cmd_tx, _cmd_rx) = mpsc::channel(10);
    let result = wire_dependencies_with_identity_store(&config, cmd_tx, Some(test_identity_store()));
    assert!(result.is_ok());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p uc-tauri wiring_exposes_secure_storage_not_keyring`
Expected: FAIL if any KeyringPort references remain or test helpers still reference keyring.

**Step 3: Write minimal implementation**

- Replace any `KeyringPort` or `keyring` field usage with `SecureStoragePort` / `secure_storage`.
- Update Noop ports in `runtime.rs` and `commands/clipboard.rs` to implement `SecureStoragePort`.
- Update wiring tests and integration tests to reference `deps.secure_storage`.
- Update `secure_storage.rs` docstrings to remove keyring wording.
- Decide whether to delete legacy keyring files; if deleting, remove mod exports and any references.

**Step 4: Run test to verify it passes**

Run: `cargo test -p uc-tauri wiring_exposes_secure_storage_not_keyring`
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs \
  src-tauri/crates/uc-tauri/src/commands/clipboard.rs \
  src-tauri/crates/uc-tauri/tests/bootstrap_integration_test.rs \
  src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs \
  src-tauri/crates/uc-platform/src/secure_storage.rs
git commit -m "refactor: remove keyring references"
```

### Task 2: Remove identity load from Libp2pNetworkAdapter::new

**Files:**

- Modify: `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs`
- Modify: `src-tauri/crates/uc-platform/src/identity_store.rs` (if needed)
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn libp2p_adapter_new_does_not_touch_identity() {
    let store = Arc::new(TestIdentityStore::new());
    let _ = Libp2pNetworkAdapter::new(store);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p uc-platform libp2p_adapter_new_does_not_touch_identity`
Expected: FAIL if `new()` still loads identity.

**Step 3: Write minimal implementation**

- Move identity loading out of `Libp2pNetworkAdapter::new` into a `start_network` / `spawn_swarm` method.
- Ensure `new()` is pure (no secure storage reads).
- Update wiring to call `new()` without triggering identity access.

**Step 4: Run test to verify it passes**

Run: `cargo test -p uc-platform libp2p_adapter_new_does_not_touch_identity`
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs \
  src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs
git commit -m "refactor: defer libp2p identity load"
```

### Task 3: Add unlock-triggered network start use case

**Files:**

- Create: `src-tauri/crates/uc-app/src/usecases/network/start_network_after_unlock.rs`
- Modify: `src-tauri/crates/uc-app/src/usecases/network/mod.rs`
- Modify: `src-tauri/crates/uc-app/src/app.rs` or `src-tauri/crates/uc-app/src/app_builder.rs` (where use cases are registered)

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn start_network_after_unlock_invokes_network_port() {
    let deps = TestAppDeps::new();
    let usecase = StartNetworkAfterUnlock::new(deps.network.clone());
    usecase.execute().await.expect("start network");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p uc-app start_network_after_unlock_invokes_network_port`
Expected: FAIL because use case does not exist.

**Step 3: Write minimal implementation**

- Implement use case to call `NetworkPort::start` (or existing control port) that triggers identity load.
- Ensure error propagation is explicit (no silent failures).

**Step 4: Run test to verify it passes**

Run: `cargo test -p uc-app start_network_after_unlock_invokes_network_port`
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/network/start_network_after_unlock.rs \
  src-tauri/crates/uc-app/src/usecases/network/mod.rs
git commit -m "feat: add start network after unlock"
```

### Task 4: Trigger network start on unlock success

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/encryption.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn unlock_command_triggers_network_start_on_success() {
    let app = TestApp::new_with_unlock_success();
    let _ = unlock_encryption_session(app.state()).await;
    assert!(app.network_started());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p uc-tauri unlock_command_triggers_network_start_on_success`
Expected: FAIL because unlock command does not start network.

**Step 3: Write minimal implementation**

- Call `StartNetworkAfterUnlock` after unlock succeeds.
- Do not call when auto-unlock fails or unlock returns error.

**Step 4: Run test to verify it passes**

Run: `cargo test -p uc-tauri unlock_command_triggers_network_start_on_success`
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/encryption.rs
git commit -m "feat: start network on unlock success"
```

### Task 5: Verification

**Step 1: Run diagnostics**

Run: `cargo check -p uc-core`
Run: `cargo check -p uc-platform`
Run: `cargo check -p uc-infra`
Run: `cargo check -p uc-app`
Run: `cargo check -p uc-tauri`

Expected: 0 errors.

**Step 2: Run focused tests**

Run: `cargo test -p uc-platform libp2p_adapter_new_does_not_touch_identity`
Run: `cargo test -p uc-app start_network_after_unlock_invokes_network_port`
Run: `cargo test -p uc-tauri wiring_exposes_secure_storage_not_keyring unlock_command_triggers_network_start_on_success`

Expected: all PASS.

**Step 3: Run full suite**

Run: `cargo test -p uc-core`
Run: `cargo test -p uc-platform`
Run: `cargo test -p uc-infra`
Run: `cargo test -p uc-app`
Run: `cargo test -p uc-tauri`

Expected: all PASS.
