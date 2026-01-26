# SecureStoragePort Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Introduce a SecureStoragePort used by KEK and identity storage with system/file implementations and a single wired instance, without touching unlock gating.

**Architecture:** Add a new port in uc-core, implement platform adapters (system keychain + file fallback) in uc-platform, and inject a single SecureStoragePort instance into uc-infra key material and uc-platform identity store via uc-tauri wiring. Keep key names and service name unchanged, and avoid keychain reads in constructors.

**Tech Stack:** Rust, Tauri, uc-core/uc-platform/uc-infra/uc-app crates, keyring crate, filesystem adapters, mockall tests.

---

### Task 1: Add SecureStoragePort to uc-core

**Files:**

- Create: `src-tauri/crates/uc-core/src/ports/security/secure_storage.rs`
- Modify: `src-tauri/crates/uc-core/src/ports/security/mod.rs`
- Modify: `src-tauri/crates/uc-core/src/ports/mod.rs`
- Modify: `src-tauri/crates/uc-core/src/ports/security/keyring.rs` (only if needed to keep exports consistent)

**Step 1: Write the failing test**

```rust
#[test]
fn secure_storage_mock_compiles_and_is_callable() {
    let storage = MockSecureStorage::new();
    let _ = storage.get("kek:v1:profile:demo");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p uc-core secure_storage_mock_compiles_and_is_callable`
Expected: FAIL with missing `SecureStoragePort` / `MockSecureStorage`.

**Step 3: Write minimal implementation**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SecureStorageError {
    #[error("secure storage unavailable: {0}")]
    Unavailable(String),
    #[error("secure storage access denied: {0}")]
    PermissionDenied(String),
    #[error("secure storage data corrupt: {0}")]
    Corrupt(String),
    #[error("secure storage failed: {0}")]
    Other(String),
}

pub trait SecureStoragePort: Send + Sync {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, SecureStorageError>;
    fn set(&self, key: &str, value: &[u8]) -> Result<(), SecureStorageError>;
    fn delete(&self, key: &str) -> Result<(), SecureStorageError>;
}

#[cfg(test)]
mockall::mock! {
    pub SecureStorage {}
    impl SecureStoragePort for SecureStorage {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, SecureStorageError>;
        fn set(&self, key: &str, value: &[u8]) -> Result<(), SecureStorageError>;
        fn delete(&self, key: &str) -> Result<(), SecureStorageError>;
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p uc-core secure_storage_mock_compiles_and_is_callable`
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-core/src/ports/security/secure_storage.rs \
  src-tauri/crates/uc-core/src/ports/security/mod.rs \
  src-tauri/crates/uc-core/src/ports/mod.rs
git commit -m "feat: add secure storage port"
```

### Task 2: Implement platform secure storage adapters

**Files:**

- Modify: `src-tauri/crates/uc-platform/src/secure_storage.rs`
- Modify: `src-tauri/crates/uc-platform/src/keyring.rs`
- Modify: `src-tauri/crates/uc-platform/src/file_keyring.rs`
- Create: `src-tauri/crates/uc-platform/src/system_secure_storage.rs`
- Create: `src-tauri/crates/uc-platform/src/file_secure_storage.rs`
- Modify: `src-tauri/crates/uc-platform/src/lib.rs`
- Test: `src-tauri/crates/uc-platform/src/system_secure_storage.rs` (unit tests)
- Test: `src-tauri/crates/uc-platform/src/file_secure_storage.rs` (unit tests)

**Step 1: Write the failing tests**

```rust
#[test]
fn system_secure_storage_uses_service_name_and_key() {
    let storage = SystemSecureStorage::new();
    let _ = storage.set("libp2p-identity:v1", b"id");
}

#[test]
fn file_secure_storage_roundtrip() {
    let dir = tempfile::TempDir::new().unwrap();
    let storage = FileSecureStorage::new_in_app_data_root(dir.path().to_path_buf()).unwrap();
    storage.set("kek:v1:profile:demo", b"kek").unwrap();
    let loaded = storage.get("kek:v1:profile:demo").unwrap();
    assert_eq!(loaded, Some(b"kek".to_vec()));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p uc-platform system_secure_storage_uses_service_name_and_key`
Expected: FAIL with missing `SystemSecureStorage`.

Run: `cargo test -p uc-platform file_secure_storage_roundtrip`
Expected: FAIL with missing `FileSecureStorage`.

**Step 3: Write minimal implementation**

```rust
pub const SERVICE_NAME: &str = "UniClipboard";

pub struct SystemSecureStorage;

impl SystemSecureStorage {
    pub fn new() -> Self { Self }
}

impl SecureStoragePort for SystemSecureStorage {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, SecureStorageError> { /* keyring entry */ }
    fn set(&self, key: &str, value: &[u8]) -> Result<(), SecureStorageError> { /* keyring entry */ }
    fn delete(&self, key: &str) -> Result<(), SecureStorageError> { /* idempotent */ }
}

pub struct FileSecureStorage { base_dir: PathBuf }

impl FileSecureStorage {
    pub fn new_in_app_data_root(app_data_root: PathBuf) -> Result<Self, io::Error> {
        let base_dir = app_data_root.join("secure-storage");
        fs::create_dir_all(&base_dir)?;
        Ok(Self { base_dir })
    }
}

impl SecureStoragePort for FileSecureStorage {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, SecureStorageError> { /* read file */ }
    fn set(&self, key: &str, value: &[u8]) -> Result<(), SecureStorageError> { /* write + chmod 0600 */ }
    fn delete(&self, key: &str) -> Result<(), SecureStorageError> { /* remove file */ }
}
```

Update `secure_storage.rs` factory functions to return `Arc<dyn SecureStoragePort>` instead of `KeyringPort`, keep `SERVICE_NAME` and key names unchanged, and avoid keychain reads in constructors.

**Step 4: Run tests to verify they pass**

Run: `cargo test -p uc-platform system_secure_storage_uses_service_name_and_key`
Expected: PASS.

Run: `cargo test -p uc-platform file_secure_storage_roundtrip`
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-platform/src/system_secure_storage.rs \
  src-tauri/crates/uc-platform/src/file_secure_storage.rs \
  src-tauri/crates/uc-platform/src/secure_storage.rs \
  src-tauri/crates/uc-platform/src/keyring.rs \
  src-tauri/crates/uc-platform/src/file_keyring.rs \
  src-tauri/crates/uc-platform/src/lib.rs
git commit -m "feat: add secure storage adapters"
```

### Task 3: Update key material service to use SecureStoragePort

**Files:**

- Modify: `src-tauri/crates/uc-infra/src/security/key_material.rs`
- Test: `src-tauri/crates/uc-infra/src/security/key_material.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn load_kek_reads_from_secure_storage_key() {
    let (storage, state) = TestSecureStorage::new();
    let (keyslot_store, _) = TestKeySlotStore::new();
    let service = DefaultKeyMaterialService::new(
        Arc::new(storage) as Arc<dyn SecureStoragePort>,
        Arc::new(keyslot_store) as Arc<dyn KeySlotStore>,
    );
    let scope = sample_scope("profile-1");
    state.lock().expect("lock").get_key = Some("kek:v1:profile:profile-1".to_string());
    let _ = service.load_kek(&scope).await;
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p uc-infra load_kek_reads_from_secure_storage_key`
Expected: FAIL with missing `SecureStoragePort` usage.

**Step 3: Write minimal implementation**

- Replace `KeyringPort` with `SecureStoragePort` in `DefaultKeyMaterialService`.
- Use key names `kek:v1:profile:<id>` unchanged.
- Map `SecureStorageError` to `EncryptionError` in one place.

**Step 4: Run test to verify it passes**

Run: `cargo test -p uc-infra load_kek_reads_from_secure_storage_key`
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-infra/src/security/key_material.rs
git commit -m "feat: use secure storage for key material"
```

### Task 4: Update identity store to use SecureStoragePort

**Files:**

- Modify: `src-tauri/crates/uc-platform/src/identity_store.rs`
- Test: `src-tauri/crates/uc-platform/src/identity_store.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn identity_store_uses_secure_storage_key() {
    let (storage, state) = TestSecureStorage::new();
    let store = SystemIdentityStore::new(Arc::new(storage));
    store.store_identity(&[1u8, 2u8]).unwrap();
    let guard = state.lock().expect("lock");
    assert_eq!(guard.last_set_key.as_deref(), Some("libp2p-identity:v1"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p uc-platform identity_store_uses_secure_storage_key`
Expected: FAIL with missing secure storage injection.

**Step 3: Write minimal implementation**

- Change `SystemIdentityStore::new` to accept `Arc<dyn SecureStoragePort>`.
- Replace keyring backend with secure storage calls.
- Preserve key name `libp2p-identity:v1` and `SERVICE_NAME` usage.

**Step 4: Run test to verify it passes**

Run: `cargo test -p uc-platform identity_store_uses_secure_storage_key`
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-platform/src/identity_store.rs
git commit -m "feat: use secure storage for identity store"
```

### Task 5: Wire a single SecureStoragePort instance

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`
- Modify: `src-tauri/crates/uc-app/src/deps.rs`
- Test: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn wiring_injects_secure_storage_once() {
    let config = AppConfig::empty();
    let (cmd_tx, _cmd_rx) = mpsc::channel(10);
    let result = wire_dependencies_with_identity_store(&config, cmd_tx, test_identity_store());
    assert!(result.is_ok());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p uc-tauri wiring_injects_secure_storage_once`
Expected: FAIL due to missing secure storage wiring.

**Step 3: Write minimal implementation**

- Create one `secure_storage` via `uc_platform::secure_storage::create_default_*`.
- Pass the same `Arc<dyn SecureStoragePort>` to `DefaultKeyMaterialService` and `SystemIdentityStore`.
- Update `AppDeps` to expose `secure_storage` if needed by use cases; remove or keep `keyring` based on actual usage.
- Do not touch `Libp2pNetworkAdapter` or unlock flow.

**Step 4: Run test to verify it passes**

Run: `cargo test -p uc-tauri wiring_injects_secure_storage_once`
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs \
  src-tauri/crates/uc-app/src/deps.rs
git commit -m "feat: wire secure storage once"
```

### Task 6: Verification

**Step 1: Run diagnostics**

Run: `cargo check -p uc-core`
Run: `cargo check -p uc-platform`
Run: `cargo check -p uc-infra`
Run: `cargo check -p uc-tauri`

Expected: 0 errors.

**Step 2: Run focused tests**

Run: `cargo test -p uc-core secure_storage_mock_compiles_and_is_callable`
Run: `cargo test -p uc-platform system_secure_storage_uses_service_name_and_key file_secure_storage_roundtrip identity_store_uses_secure_storage_key`
Run: `cargo test -p uc-infra load_kek_reads_from_secure_storage_key`
Run: `cargo test -p uc-tauri wiring_injects_secure_storage_once`

Expected: all PASS.

**Step 3: Commit**

```bash
git status --short
```

Ensure only intended files are modified.
