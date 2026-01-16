# Auto Keyring Selection Refactor Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task.

**Goal:** Move secure-storage capability probing + keyring backend selection into `uc-platform`, so `uc-tauri` wiring only assembles ports and never matches on backend types.

**Architecture:** Keep `KeyringPort` in `uc-core`. Add a `uc-platform` factory that returns `Arc<dyn KeyringPort>` using internal capability detection. `uc-tauri` wiring calls the factory and injects the returned port into infra + app deps. Infra must not construct platform adapters.

**Tech Stack:** Rust, `Arc<dyn Trait>`, `thiserror`, existing `uc_platform::capability`.

## Context / Motivation

Current code in `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs` selects between `SystemKeyring` and `FileBasedKeyring` using `detect_storage_capability()`.

Problems:

- `wiring.rs` now knows keyring engines and selection policy, which is platform policy.
- The comment about needing a concrete keyring type for `DefaultKeyMaterialService` is outdated: `DefaultKeyMaterialService::new` accepts `Arc<dyn KeyringPort>`.
- Infra currently constructs a platform adapter (keyring), which blurs boundaries.

Target behavior:

- `uc-platform` owns capability probing and adapter selection.
- `uc-tauri` wiring depends only on the `KeyringPort` abstraction.
- A single `Arc<dyn KeyringPort>` instance is injected into both `DefaultKeyMaterialService` and `AppDeps`.

---

## Task 1: Add platform factory API for keyring selection

### Design specifics (validated)

Factory API details are part of the refactor scope (so we don’t leave the interface vague).

**Module placement (recommended):**

- Create: `src-tauri/crates/uc-platform/src/secure_storage.rs`
- Modify: `src-tauri/crates/uc-platform/src/lib.rs` to add `pub mod secure_storage;`

**Public API (single entrypoint):**

```rust
pub fn create_default_keyring(
) -> Result<std::sync::Arc<dyn uc_core::ports::KeyringPort>, KeyringFactoryError>
```

**Behavior:**

- `SecureStorageCapability::SystemKeyring` → return `SystemKeyring`
- `SecureStorageCapability::FileBasedKeystore` → default auto-fallback to `FileBasedKeyring::new()`
- `SecureStorageCapability::Unsupported` → return error (no silent fallback)

**Error type:**

```rust
#[derive(Debug, thiserror::Error)]
pub enum KeyringFactoryError {
    #[error("secure storage unsupported: {capability:?}")]
    Unsupported { capability: SecureStorageCapability },

    #[error("failed to initialize file-based keyring: {0}")]
    FileBasedInit(#[from] std::io::Error),
}
```

**Logging policy (platform-owned; warn on fallback):**

- `debug!`: log detected capability
- `info!`: using system keyring
- `warn!`: using file-based keyring (WSL/headless fallback)
- `error!`: unsupported capability

**Note on security semantics:**

- `FileBasedKeyring` currently writes KEK bytes to disk (permissions 0600 on Unix). Treat it as an insecure dev fallback in messaging/logs.

**Files:**

- Create: `src-tauri/crates/uc-platform/src/secure_storage.rs`
- Modify: `src-tauri/crates/uc-platform/src/lib.rs`
- Modify: `src-tauri/crates/uc-platform/src/keyring.rs` (if shared helpers are needed)
- Modify: `src-tauri/crates/uc-platform/src/file_keyring.rs` (only if needed for testability)

### Step 1: Write the failing test (deterministic)

Runtime environment (WSL/headless/desktop) is hard to test reliably. Create an internal helper that takes a capability enum and is testable.

Add in `src-tauri/crates/uc-platform/src/secure_storage.rs`:

- `fn keyring_from_capability(cap: SecureStorageCapability) -> Result<Arc<dyn KeyringPort>, KeyringFactoryError>`

Write unit tests:

- `Unsupported` → returns `Err(...)`
- `SystemKeyring` → returns `Ok(_)`
- `FileBasedKeystore` → returns `Ok(_)`

For the file-based case, avoid writing into the real user config dir. Prefer one of:

- Construct `FileBasedKeyring::with_base_dir(tempdir)` (recommended), OR
- Factor `keyring_from_capability` to accept `Option<PathBuf>` for base dir in tests.

### Step 2: Run test to verify it fails

Run:

- `cargo test -p uc-platform keyring_from_capability -- --nocapture`

Expected:

- FAIL because helper does not exist yet.

Expected:

- FAIL because helper/factory does not exist yet.

### Step 3: Implement minimal factory

In `src-tauri/crates/uc-platform/src/secure_storage.rs`:

- Add `KeyringFactoryError` (use `thiserror::Error`).
- Implement `keyring_from_capability`.
- Implement public API:

```rust
pub fn create_default_keyring() -> Result<Arc<dyn KeyringPort>, KeyringFactoryError>
```

Behavior:

- Call `detect_storage_capability()`.
- Map capability to adapter.
- Keep logs in platform (no silent fallback).
- `Unsupported` → return a clear error.

### Step 4: Run tests

Run:

- `cargo test -p uc-platform`

Expected:

- PASS.

### Step 5: Commit

```bash
git add src-tauri/crates/uc-platform/src/lib.rs src-tauri/crates/uc-platform/src/secure_storage.rs
git commit -m "feat(platform): add default keyring factory"
```

---

## Task 2: Refactor `wiring.rs` to use platform factory

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`

### Step 1: Update infra creation signature

Change:

- `fn create_infra_layer(...) -> WiringResult<(InfraLayer, Arc<dyn KeyringPort>)>`

To:

- `fn create_infra_layer(..., keyring: Arc<dyn KeyringPort>) -> WiringResult<InfraLayer>`

Inside infra creation:

- Remove `detect_storage_capability` / `SystemKeyring` / `FileBasedKeyring` usage.
- Remove the `(keyring_for_key_material, keyring)` tuple.
- Use `DefaultKeyMaterialService::new(keyring.clone(), keyslot_store)`.

### Step 2: Update `wire_dependencies` assembly order

In `wire_dependencies`:

- Create the keyring early:

```rust
let keyring = uc_platform::secure_storage::create_default_keyring()
    .map_err(|e| WiringError::KeyringInit(e.to_string()))?;
```

- Call `create_infra_layer(..., keyring.clone())`.
- Call `create_platform_layer(keyring, ...)`.

### Step 3: Remove obsolete comment

Delete the misleading comment in wiring about needing a concrete type for `DefaultKeyMaterialService`.

### Step 4: Run tests

Run:

- `cargo test -p uc-tauri`

Expected:

- PASS (note: clipboard init may fail in headless environments; don’t “fix around” it here).

### Step 5: Commit

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs
git commit -m "refactor(wiring): delegate keyring selection to platform"
```

---

## Task 3: Clean imports + ensure no boundary leaks

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`

### Step 1: Remove unused imports

Remove now-unneeded imports in wiring:

- `uc_platform::capability::detect_storage_capability`
- `uc_platform::capability::SecureStorageCapability`
- `uc_platform::keyring::SystemKeyring`
- `uc_platform::file_keyring::FileBasedKeyring`

### Step 2: Run formatting and tests

Run:

- `cargo fmt`
- `cargo test -p uc-tauri`

### Step 3: Commit

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs
git commit -m "chore: clean wiring imports and comments"
```

---

## Verification Checklist

- `uc-tauri` wiring contains no `match` over `SecureStorageCapability`.
- `uc-infra` does not construct any `uc-platform` adapters.
- A single `Arc<dyn KeyringPort>` instance is used for:
  - `DefaultKeyMaterialService`
  - `AppDeps.keyring`
- Errors in keyring init are surfaced (returned as `WiringError::KeyringInit` and logged).
