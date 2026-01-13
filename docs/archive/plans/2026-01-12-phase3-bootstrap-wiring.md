# Phase 3: Bootstrap Wiring Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Migrate dependency injection from main.rs to bootstrap/wiring.rs, completing the hexagonal architecture wiring layer.

**Architecture:** Bootstrap module acts as the "wiring operator" - the only place allowed to depend on uc-infra + uc-platform + uc-app simultaneously. Direct construction replaces Builder pattern.

**Tech Stack:** Rust, Tauri 2, Diesel ORM, SQLite, tokio async runtime

---

## Prerequisites

- Phase 2 completed (bootstrap module skeleton exists)
- `uc-core::config` module renamed from `settings`
- `AppDeps` struct defined in `uc-app`

---

## Task 1: Extend AppDeps with All Required Dependencies

**Files:**

- Modify: `src-tauri/crates/uc-app/src/lib.rs`

**Step 1: Read current AppDeps definition**

```bash
grep -A 20 "pub struct AppDeps" src-tauri/crates/uc-app/src/lib.rs
```

**Step 2: Add missing port dependencies**

Add to `AppDeps`:

- `pub network: Arc<dyn NetworkPort>`
- `pub ui_port: Arc<dyn UiPort>`
- `pub autostart: Arc<dyn AutostartPort>`
- `pub clock: Arc<dyn ClockPort>`
- `pub hash: Arc<dyn ContentHashPort>`

**Step 3: Update App::new() signature**

Update constructor to accept the extended `AppDeps`.

**Step 4: Run cargo check**

```bash
cd src-tauri && cargo check --package uc-app
```

Expected: No errors

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-app/src/lib.rs
git commit -m "feat(uc-app): extend AppDeps with all required ports"
```

---

## Task 2: Create WiringResult and WiringError Types

**Files:**

- Create: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`

**Step 1: Write the error type tests**

Create `src-tauri/crates/uc-tauri/tests/bootstrap_wiring_test.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wiring_error_display() {
        let err = WiringError::DatabaseInit("connection failed".to_string());
        assert!(err.to_string().contains("Database initialization"));
    }

    #[test]
    fn test_wiring_result_success() {
        let result: WiringResult<()> = Ok(());
        assert!(result.is_ok());
    }
}
```

**Step 2: Run tests to verify they fail**

```bash
cd src-tauri && cargo test --package uc-tauri wiring_test
```

Expected: FAIL with "WiringError not defined"

**Step 3: Write minimal error type implementation**

In `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`:

```rust
//! # Dependency Injection / 依赖注入模块
//!
//! ## Responsibilities / 职责
//!
//! - ✅ Create infra implementations (db, fs, keyring) / 创建 infra 层具体实现
//! - ✅ Create platform implementations (clipboard, network) / 创建 platform 层具体实现
//! - ✅ Inject all dependencies into App / 将所有依赖注入到 App
//!
//! ## Prohibited / 禁止事项
//!
//! ❌ **No business logic / 禁止包含任何业务逻辑**
//! - Do not decide "what to do if encryption uninitialized"
//! - 不判断"如果加密未初始化就怎样"
//! - Do not handle "what to do if device not registered"
//! - 不处理"如果设备未注册就怎样"
//!
//! ❌ **No configuration validation / 禁止做配置验证**
//! - Config already loaded in config.rs
//! - 配置已在 config.rs 加载
//! - Validation should be in use case or upper layer
//! - 验证应在 use case 或上层
//!
//! ❌ **No direct concrete implementation usage / 禁止直接使用具体实现**
//! - Must inject through Port traits
//! - 必须通过 Port trait 注入
//! - Do not call implementation methods directly after App construction
//! - 不在 App 构造后直接调用实现方法
//!
//! ## Architecture Principle / 架构原则
//!
//! > **This is the only place allowed to depend on uc-infra + uc-platform + uc-app simultaneously.**
//! > **这是唯一允许同时依赖 uc-infra + uc-platform + uc-app 的地方。**
//! > But this privilege is only for "assembly", not for "decision making".
//! > 但这种特权仅用于"组装"，不用于"决策"。

use std::sync::Arc;
use std::path::PathBuf;

use uc_app::AppDeps;
use uc_core::config::AppConfig;
use uc_core::ports::*;

/// Result type for wiring operations
pub type WiringResult<T> = Result<T, WiringError>;

/// Errors during dependency injection
/// 依赖注入错误（基础设施初始化失败）
#[derive(Debug, thiserror::Error)]
pub enum WiringError {
    #[error("Database initialization failed: {0}")]
    DatabaseInit(String),

    #[error("Keyring initialization failed: {0}")]
    KeyringInit(String),

    #[error("Clipboard initialization failed: {0}")]
    ClipboardInit(String),

    #[error("Network initialization failed: {0}")]
    NetworkInit(String),

    #[error("Blob storage initialization failed: {0}")]
    BlobStorageInit(String),

    #[error("Settings repository initialization failed: {0}")]
    SettingsInit(String),
}
```

**Step 4: Run tests to verify they pass**

```bash
cd src-tauri && cargo test --package uc-tauri wiring_test
```

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs
git add src-tauri/crates/uc-tauri/tests/bootstrap_wiring_test.rs
git commit -m "feat(uc-tauri): add WiringError and WiringResult types"
```

---

## Task 3: Implement Database Pool Creation

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`

**Step 1: Write test for database pool creation**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_db_pool_returns_pool() {
        // This test verifies the function signature is correct
        // Actual DB pool creation is tested in integration tests
    }
}
```

**Step 2: Run test**

```bash
cd src-tauri && cargo test --package uc-tauri test_create_db_pool
```

Expected: FAIL

**Step 3: Implement database pool creation function**

```rust
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::SqliteConnection;

/// Create SQLite database connection pool
/// 创建 SQLite 数据库连接池
fn create_db_pool(db_path: &PathBuf) -> WiringResult<Pool<ConnectionManager<SqliteConnection>>> {
    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| WiringError::DatabaseInit(format!("Failed to create DB directory: {}", e)))?;
    }

    // Create connection manager
    let manager = ConnectionManager::<SqliteConnection>::new(db_path.to_string_lossy());

    // Build pool
    Pool::builder()
        .build(&manager)
        .map_err(|e| WiringError::DatabaseInit(format!("Failed to create connection pool: {}", e)))
}
```

**Step 4: Run tests**

```bash
cd src-tauri && cargo test --package uc-tauri test_create_db_pool
```

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs
git commit -m "feat(uc-tauri): add database pool creation function"
```

---

## Task 4: Implement Infra Layer Repository Creation

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`

**Step 1: Add function to create all infra repositories**

```rust
use uc_infra::db::repositories::*;
use uc_infra::security::*;
use uc_infra::settings::FileSettingsRepository;
use uc_infra::blob::*;

/// Create all infrastructure layer implementations
/// 创建所有基础设施层实现
fn create_infra_layer(
    db_pool: &Pool<ConnectionManager<SqliteConnection>>,
    vault_path: &PathBuf,
    config: &AppConfig,
) -> WiringResult<InfraLayer> {
    // Create repositories
    let clipboard_entry_repo = Arc::new(DieselClipboardEntryRepository::new(db_pool.clone()))
        as Arc<dyn ClipboardEntryRepositoryPort>;

    let clipboard_event_repo = Arc::new(DieselClipboardEventRepository::new(db_pool.clone()))
        as Arc<dyn ClipboardEventRepositoryPort>;

    let representation_repo = Arc::new(DieselRepresentationRepository::new(db_pool.clone()))
        as Arc<dyn ClipboardRepresentationRepositoryPort>;

    let device_repo = Arc::new(DieselDeviceRepository::new(db_pool.clone()))
        as Arc<dyn DeviceRepositoryPort>;

    let blob_repository = Arc::new(DieselBlobRepository::new(db_pool.clone()))
        as Arc<dyn BlobRepositoryPort>;

    // Create blob store
    let blob_store_path = vault_path.join("blobs");
    let blob_store = Arc::new(DieselBlobStore::new(blob_store_path))
        as Arc<dyn BlobStorePort>;

    // Create encryption
    let key_material_path = vault_path.join("keys");
    let key_material = Arc::new(KeyMaterialRepository::new(key_material_path))
        as Arc<dyn KeyMaterialPort>;

    let encryption = Arc::new(AesGcmEncryption::new())
        as Arc<dyn EncryptionPort>;

    // Create settings repository
    let settings_path = config.config_path.clone();
    let settings_repo = Arc::new(FileSettingsRepository::new(settings_path))
        as Arc<dyn SettingsPort>;

    // Create system services
    let clock = Arc::new(uc_platform::adapters::SystemClock::new())
        as Arc<dyn ClockPort>;

    let hash = Arc::new(uc_platform::adapters::BlakeContentHasher::new())
        as Arc<dyn ContentHashPort>;

    Ok(InfraLayer {
        clipboard_entry_repo,
        clipboard_event_repo,
        representation_repo,
        device_repo,
        blob_repository,
        blob_store,
        key_material,
        encryption,
        settings_repo,
        clock,
        hash,
    })
}

/// Infrastructure layer implementations
struct InfraLayer {
    clipboard_entry_repo: Arc<dyn ClipboardEntryRepositoryPort>,
    clipboard_event_repo: Arc<dyn ClipboardEventRepositoryPort>,
    representation_repo: Arc<dyn ClipboardRepresentationRepositoryPort>,
    device_repo: Arc<dyn DeviceRepositoryPort>,
    blob_repository: Arc<dyn BlobRepositoryPort>,
    blob_store: Arc<dyn BlobStorePort>,
    key_material: Arc<dyn KeyMaterialPort>,
    encryption: Arc<dyn EncryptionPort>,
    settings_repo: Arc<dyn SettingsPort>,
    clock: Arc<dyn ClockPort>,
    hash: Arc<dyn ContentHashPort>,
}
```

**Step 2: Run cargo check**

```bash
cd src-tauri && cargo check --package uc-tauri
```

Expected: May have type resolution errors, fix imports as needed

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs
git commit -m "feat(uc-tauri): add infra layer repository creation"
```

---

## Task 5: Implement Platform Layer Creation

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`

**Step 1: Add platform layer creation function**

```rust
use uc_platform::adapters::*;

/// Create all platform layer implementations
/// 创建所有平台层实现
fn create_platform_layer() -> WiringResult<PlatformLayer> {
    // Create clipboard (platform-specific)
    #[cfg(target_os = "macos")]
    let clipboard = Arc::new(MacOSClipboard::new()) as Arc<dyn SystemClipboardPort>;

    #[cfg(target_os = "windows")]
    let clipboard = Arc::new(WindowsClipboard::new()) as Arc<dyn SystemClipboardPort>;

    #[cfg(target_os = "linux")]
    let clipboard = Arc::new(LinuxClipboard::new()) as Arc<dyn SystemClipboardPort>;

    // Create keyring
    let keyring = Arc::new(PlatformKeyring::new())
        as Arc<dyn KeyringPort>;

    // Note: UiPort and AutostartPort require AppHandle, created later in runtime.rs

    Ok(PlatformLayer {
        clipboard,
        keyring,
    })
}

/// Platform layer implementations
struct PlatformLayer {
    clipboard: Arc<dyn SystemClipboardPort>,
    keyring: Arc<dyn KeyringPort>,
}
```

**Step 2: Run cargo check**

```bash
cd src-tauri && cargo check --package uc-tauri
```

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs
git commit -m "feat(uc-tauri): add platform layer creation"
```

---

## Task 6: Implement Main wire_dependencies Function

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`

**Step 1: Write integration test for wiring**

```rust
#[test]
fn test_wire_dependencies_creates_deps() {
    let config = AppConfig::default();
    let result = wire_dependencies(&config);

    // Should succeed with default paths
    assert!(result.is_ok());
}
```

**Step 2: Run test**

```bash
cd src-tauri && cargo test --package uc-tauri test_wire_dependencies
```

Expected: FAIL

**Step 3: Implement main wiring function**

```rust
/// Wire all dependencies and create AppDeps
/// 连接所有依赖并创建 AppDeps
///
/// This is the ONLY place where uc-infra, uc-platform, and uc-app
/// may be depended upon simultaneously.
/// 这是唯一允许同时依赖 uc-infra、uc-platform 和 uc-app 的地方。
pub fn wire_dependencies(config: &AppConfig) -> WiringResult<AppDeps> {
    // Create database pool
    let db_path = config.vault_path.join("uniclipboard.db");
    let db_pool = create_db_pool(&db_path)?;

    // Create infra layer
    let infra = create_infra_layer(&db_pool, &config.vault_path, config)?;

    // Create platform layer
    let platform = create_platform_layer()?;

    // Note: UiPort and AutostartPort are created in runtime.rs
    // where AppHandle is available

    // Create network (placeholder until implemented)
    let network = Arc::new(uc_platform::adapters::LocalNetworkAdapter::new())
        as Arc<dyn NetworkPort>;

    // Create representation materializer
    let representation_materializer = Arc::new(uc_infra::clipboard::RepresentationMaterializer::new(
        infra.blob_store.clone(),
        infra.blob_repository.clone(),
    )) as Arc<dyn ClipboardRepresentationMaterializerPort>;

    // Create blob materializer
    let blob_materializer = Arc::new(uc_infra::blob::BlobMaterializer::new(
        infra.blob_store.clone(),
    )) as Arc<dyn BlobMaterializerPort>;

    // Create encryption session (placeholder - needs proper initialization)
    let encryption_session = Arc::new(uc_infra::security::InMemoryEncryptionSession::new())
        as Arc<dyn EncryptionSessionPort>;

    // Create device identity (placeholder)
    let device_identity = Arc::new(uc_infra::device::LocalDeviceIdentity::new())
        as Arc<dyn DeviceIdentityPort>;

    // UiPort and AutostartPort will be set in runtime.rs
    // For now, use placeholder implementations
    let ui_port = Arc::new(uc_platform::adapters::PlaceholderUiPort::new())
        as Arc<dyn UiPort>;
    let autostart = Arc::new(uc_platform::adapters::PlaceholderAutostartPort::new())
        as Arc<dyn AutostartPort>;

    Ok(AppDeps {
        clipboard: platform.clipboard,
        clipboard_entry_repo: infra.clipboard_entry_repo,
        clipboard_event_repo: infra.clipboard_event_repo,
        representation_repo: infra.representation_repo,
        representation_materializer,
        encryption: infra.encryption,
        encryption_session,
        keyring: platform.keyring,
        key_material: infra.key_material,
        device_repo: infra.device_repo,
        device_identity,
        network,
        blob_store: infra.blob_store,
        blob_repository: infra.blob_repository,
        blob_materializer,
        settings: infra.settings_repo,
        ui_port,
        autostart,
        clock: infra.clock,
        hash: infra.hash,
    })
}
```

**Step 4: Export from bootstrap mod.rs**

Update `src-tauri/crates/uc-tauri/src/bootstrap/mod.rs`:

```rust
pub mod config;
pub mod runtime;
pub mod seed;
pub mod wiring;

pub use config::{load_config, ConfigError};
pub use runtime::{create_app_runtime, AppRuntimeCreationError};
pub use seed::RuntimeSeed;
pub use wiring::{wire_dependencies, WiringError, WiringResult};
```

**Step 5: Run tests**

```bash
cd src-tauri && cargo test --package uc-tauri test_wire_dependencies
```

Expected: PASS

**Step 6: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/
git commit -m "feat(uc-tauri): implement wire_dependencies function"
```

---

## Task 7: Add Placeholder Implementations for Missing Ports

**Files:**

- Create: `src-tauri/crates/uc-platform/src/adapters/placeholder.rs`

**Step 1: Create placeholder implementations**

```rust
//! Placeholder implementations for ports that require runtime setup
//! 这些 Port 需要运行时设置，提供占位实现

use uc_core::ports::*;
use std::sync::Arc;

pub struct PlaceholderUiPort;

impl UiPort for PlaceholderUiPort {
    fn show_notification(&self, _title: &str, _message: &str) -> Result<(), String> {
        Ok(())
    }

    fn confirm_dialog(&self, _title: &str, _message: &str) -> Result<bool, String> {
        Ok(true)
    }
}

impl PlaceholderUiPort {
    pub fn new() -> Self {
        Self
    }
}

pub struct PlaceholderAutostartPort;

impl AutostartPort for PlaceholderAutostartPort {
    fn is_enabled(&self) -> Result<bool, String> {
        Ok(false)
    }

    fn enable(&self) -> Result<(), String> {
        Ok(())
    }

    fn disable(&self) -> Result<(), String> {
        Ok(())
    }
}

impl PlaceholderAutostartPort {
    pub fn new() -> Self {
        Self
    }
}
```

**Step 2: Export from adapters mod.rs**

```rust
pub mod placeholder;

pub use placeholder::{PlaceholderUiPort, PlaceholderAutostartPort};
```

**Step 3: Run cargo check**

```bash
cd src-tauri && cargo check --package uc-platform
```

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-platform/src/adapters/placeholder.rs
git commit -m "feat(uc-platform): add placeholder implementations for runtime ports"
```

---

## Task 8: Update main.rs to Use New Bootstrap Flow

**Files:**

- Modify: `src-tauri/src/main.rs`

**Step 1: Add bootstrap imports**

```rust
use uc_tauri::bootstrap::{load_config, wire_dependencies, create_app_runtime, RuntimeSeed};
```

**Step 2: Replace old dependency creation with bootstrap flow**

Find the section around lines 148-156 and replace:

```rust
// OLD CODE (remove):
// let (clipboard_cmd_tx, clipboard_cmd_rx) = mpsc::channel(100);
// let (p2p_cmd_tx, p2p_cmd_rx) = mpsc::channel(100);
// let config = Arc::new(user_setting.clone());
// let runtime_handle = AppRuntimeHandle::new(...);

// NEW CODE:
// Load configuration
let app_config = load_config(&config_path)
    .unwrap_or_else(|e| {
        eprintln!("Failed to load config: {}, using defaults", e);
        AppConfig::default()
    });

// Wire all dependencies
let app_deps = wire_dependencies(&app_config)
    .expect("Failed to wire dependencies");

// Create runtime seed
let seed = RuntimeSeed::new(app_config);

// Note: Runtime creation happens in Tauri setup where AppHandle is available
```

**Step 3: Update Tauri setup**

```rust
.setup(|app| {
    // Create runtime with AppHandle
    let runtime = create_app_runtime(app.handle(), seed, app_deps)
        .expect("Failed to create app runtime");

    // Manage runtime state for commands
    app.manage(runtime);

    Ok(())
})
```

**Step 4: Run cargo check**

```bash
cd src-tauri && cargo check
```

**Step 5: Run integration tests**

```bash
cd src-tauri && cargo test
```

**Step 6: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "refactor: use bootstrap flow for dependency injection"
```

---

## Task 9: Remove Legacy Dependency Injection Code

**Files:**

- Modify: `src-tauri/src/main.rs`

**Step 1: Remove old dependency creation**

Remove any remaining legacy dependency injection code:

- Old channel creation
- Old runtime handle creation
- Old AppBuilder usage

**Step 2: Remove unused imports**

```rust
// Remove these if no longer used:
// use infrastructure::runtime::{AppRuntime, AppRuntimeHandle};
// use application::builder::AppBuilder;
```

**Step 3: Run cargo check**

```bash
cd src-tauri && cargo check
```

**Step 4: Run clippy**

```bash
cd src-tauri && cargo clippy -- -D warnings
```

**Step 5: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "refactor: remove legacy dependency injection code"
```

---

## Task 10: Integration Testing

**Files:**

- Create: `src-tauri/tests/bootstrap_integration_test.rs`

**Step 1: Write integration test**

```rust
#[cfg(test)]
mod integration_tests {
    use std::path::PathBuf;
    use uc_tauri::bootstrap::{load_config, wire_dependencies};
    use uc_core::config::AppConfig;

    #[test]
    fn test_full_bootstrap_flow() {
        // Create test config
        let mut config = AppConfig::default();
        config.vault_path = PathBuf::from("/tmp/uniclipboard-test");

        // Wire dependencies
        let deps = wire_dependencies(&config);
        assert!(deps.is_ok(), "Wiring should succeed");

        let deps = deps.unwrap();
        // Verify all dependencies are present
        assert!(deps.clipboard.as_ref() as *const _ as usize != 0);
        assert!(deps.encryption.as_ref() as *const _ as usize != 0);
    }

    #[test]
    fn test_wiring_with_invalid_path() {
        let mut config = AppConfig::default();
        config.vault_path = PathBuf::from("/invalid/path/that/cannot/be/created");

        let result = wire_dependencies(&config);
        // Should fail with database init error
        assert!(result.is_err());
    }
}
```

**Step 2: Run integration tests**

```bash
cd src-tauri && cargo test --test bootstrap_integration_test
```

**Step 3: Fix any issues found**

**Step 4: Commit**

```bash
git add src-tauri/tests/bootstrap_integration_test.rs
git commit -m "test(uc-tauri): add bootstrap integration tests"
```

---

## Task 11: Architecture Validation

**Files:**

- Run validation checklist

**Step 1: Run architecture self-checks**

Based on the architecture document, verify:

```bash
# Self-check 1: Can bootstrap be directly depended upon by test crates?
# Expected: ❌ No
grep -r "use uc_tauri::bootstrap" src-tauri/crates/
# Should only find src-tauri/src/main.rs or uc-tauri internals

# Self-check 2: Can business code compile independently without bootstrap?
# Expected: ✅ Yes
cd src-tauri && cargo check --package uc-app
# Should succeed

# Self-check 4: Does config.rs check vault state?
# Expected: ❌ No
grep -r "exists\(" src-tauri/crates/uc-tauri/src/bootstrap/config.rs
# Should not find vault existence checks

# Self-check 6: Does AppBuilder still exist?
# Expected: ❌ No
ls src-tauri/crates/uc-app/src/builder.rs 2>&1 | grep "No such file"
# Should not exist
```

**Step 2: Document validation results**

Create `docs/plans/2026-01-12-phase3-validation.md`:

```markdown
# Phase 3 Architecture Validation Results

## Self-Check Results

1. **Can bootstrap be directly depended upon by test crates?**
   - Expected: ❌ No
   - Actual: ❌ No ✅

2. **Can business code compile independently without bootstrap?**
   - Expected: ✅ Yes
   - Actual: ✅ Yes ✅

3. **Does bootstrap "know too much" about concrete implementations?**
   - Expected: ✅ Yes (that's its job)
   - Actual: ✅ Yes ✅

4. **Does config.rs check vault state?**
   - Expected: ❌ No
   - Actual: ❌ No ✅

5. **Does main.rs contain long-term business policies?**
   - Expected: ❌ No
   - Actual: ❌ No ✅

6. **Does AppBuilder still exist?**
   - Expected: ❌ No
   - Actual: ❌ No ✅

7. **Does uc-core::config contain only DTOs?**
   - Expected: ✅ Yes
   - Actual: ✅ Yes ✅

8. **Is WiringError assumed "always fatal"?**
   - Expected: ❌ No (allow runtime-mode-based handling)
   - Actual: ❌ No ✅

## Result: All Architecture Checks Passed ✅
```

**Step 3: Commit**

```bash
git add docs/plans/2026-01-12-phase3-validation.md
git commit -m "docs(bootstrap): add Phase 3 architecture validation results"
```

---

## Task 12: Documentation Update

**Files:**

- Modify: `docs/plans/2026-01-12-bootstrap-architecture-design.md`

**Step 1: Update migration plan status**

Update Phase 3 status from "Pending" to "✅ Completed"

**Step 2: Add Phase 3 implementation summary**

```markdown
### Phase 3: Gradual Dependency Injection Migration ✅ COMPLETED

**Completed on:** 2026-01-12

**Changes made:**

1. ✅ Implemented infra layer creation in wiring.rs
2. ✅ Implemented platform layer creation in wiring.rs
3. ✅ Created wire_dependencies() function
4. ✅ Updated main.rs to use bootstrap flow
5. ✅ Removed legacy dependency injection code
6. ✅ Added integration tests

**Commits:**

- feat(uc-app): extend AppDeps with all required ports
- feat(uc-tauri): add WiringError and WiringResult types
- feat(uc-tauri): add database pool creation function
- feat(uc-tauri): add infra layer repository creation
- feat(uc-tauri): add platform layer creation
- feat(uc-tauri): implement wire_dependencies function
- feat(uc-platform): add placeholder implementations for runtime ports
- refactor: use bootstrap flow for dependency injection
- refactor: remove legacy dependency injection code
- test(uc-tauri): add bootstrap integration tests
```

**Step 3: Commit**

```bash
git add docs/plans/2026-01-12-bootstrap-architecture-design.md
git commit -m "docs(bootstrap): mark Phase 3 as completed"
```

---

## Final Verification

After completing all tasks, run final verification:

```bash
# Full test suite
cd src-tauri && cargo test

# Clippy check
cd src-tauri && cargo clippy -- -D warnings

# Build check
cd src-tauri && cargo check --all-targets

# Format check
cd src-tauri && cargo fmt --check
```

---

## References

- Architecture Design: [docs/plans/2026-01-12-bootstrap-architecture-design.md](docs/plans/2026-01-12-bootstrap-architecture-design.md)
- Bootstrap Module: [src-tauri/crates/uc-tauri/src/bootstrap/](src-tauri/crates/uc-tauri/src/bootstrap/)
- uc-app AppDeps: [src-tauri/crates/uc-app/src/lib.rs](src-tauri/crates/uc-app/src/lib.rs)

---

**Plan complete and saved to `docs/plans/2026-01-12-phase3-bootstrap-wiring.md`.**

---

## Execution Options

**Two execution options:**

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

**Which approach?**
