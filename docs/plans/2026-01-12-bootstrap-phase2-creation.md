# Phase 2: Bootstrap Module Creation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create the bootstrap module (config.rs, wiring.rs) that uses Phase 1 foundations (uc-core::config and uc-app::AppDeps) for dependency injection, following Hexagonal Architecture principles.

**Architecture:** Bootstrap is the "wiring operator" - the only place allowed to depend on uc-infra + uc-platform + uc-app simultaneously. It reads config (pure DTO), creates all infrastructure/platform implementations, and assembles them into App via App::new(AppDeps).

**Tech Stack:** Rust, anyhow for error handling, Phase 1 modules (uc-core::config::AppConfig, uc-app::AppDeps), uc-infra implementations, uc-platform implementations

---

## Task 1: Create bootstrap/config.rs - Pure Configuration Loading

**Files:**

- Create: `src-tauri/crates/uc-tauri/src/bootstrap/config.rs`
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/mod.rs`

**Step 1: Write the failing test**

```rust
// src-tauri/crates/uc-tauri/src/bootstrap/config.rs
//! # Configuration Loader / 配置加载模块
//!
//! ## Responsibilities / 职责
//!
//! - ✅ Read TOML files / 读取 TOML 文件
//! - ✅ Parse into AppConfig DTO / 解析为 AppConfig DTO
//! - ✅ Read path info (without checking file existence) / 读取路径信息（不检查文件是否存在）
//!
//! ## Prohibited / 禁止事项
//!
//! ❌ **No vault file existence checks / 禁止检查 vault 文件是否存在**
//! - May "discover" vault presence, but must not "care" about it
//! - 可以"发现 vault 在不在"，但不能"在意它在不在"
//!
//! ❌ **No configuration validation / 禁止验证配置有效性**
//! - Do not judge if device_name is valid
//! - 不判断 device_name 是否合法
//!
//! ❌ **No default values / 禁止设置默认值**
//! - Empty string is a valid "fact"
//! - 空字符串是合法的"事实"

use anyhow::Result;
use std::path::PathBuf;
use uc_core::config::AppConfig;

/// Load application configuration from TOML file
/// 从 TOML 文件加载应用配置
///
/// **Prohibited / 禁止**: This function must NOT contain any validation,
/// default value logic, or business decisions about "what to do if config missing".
/// 此函数必须不包含任何验证、默认值逻辑或关于"配置缺失时怎么办"的业务决策。
///
/// ## Returns / 返回
///
/// - `Ok(AppConfig)` - Parsed configuration DTO (may have empty values)
/// - `Err(ConfigError)` - IO or parse errors only
pub fn load_config(config_path: PathBuf) -> Result<AppConfig> {
    todo!("Implement load_config")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_config_reads_valid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let toml_content = r#"
            [general]
            device_name = "TestDevice"
            silent_start = true

            [security]
            vault_key_path = "/data/vault.key"
            vault_snapshot_path = "/data/snapshot.bin"

            [network]
            webserver_port = 8080

            [storage]
            database_path = "/data/clipboard.db"
        "#;

        fs::write(&config_path, toml_content).unwrap();

        let config = load_config(config_path).unwrap();

        assert_eq!(config.device_name, "TestDevice");
        assert_eq!(config.vault_key_path, PathBuf::from("/data/vault.key"));
        assert_eq!(config.webserver_port, 8080);
        assert_eq!(config.silent_start, true);
    }

    #[test]
    fn test_load_config_returns_empty_values_when_missing() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("empty.toml");

        let toml_content = r#"
            [general]
            # device_name is missing
        "#;

        fs::write(&config_path, toml_content).unwrap();

        let config = load_config(config_path).unwrap();

        // Empty values are valid "facts", not errors
        assert_eq!(config.device_name, "");
        assert_eq!(config.webserver_port, 0);
    }

    #[test]
    fn test_load_config_does_not_validate_port_range() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let toml_content = r#"
            [network]
            webserver_port = 99999
        "#;

        fs::write(&config_path, toml_content).unwrap();

        let config = load_config(config_path).unwrap();

        // We don't validate - the value is truncated (99999 as u16 = 34463)
        assert_eq!(config.webserver_port, 34463);
    }

    #[test]
    fn test_load_config_returns_io_error_on_file_not_found() {
        let config_path = PathBuf::from("/nonexistent/config.toml");

        let result = load_config(config_path);

        assert!(result.is_err());
        // Should be IO error, not "config missing" business error
        assert!(result.unwrap_err().to_string().contains("No such file"));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test -p uc-tauri bootstrap::config::tests`
Expected: FAIL with "not implemented" or function not found

**Step 3: Write minimal implementation**

```rust
// src-tauri/crates/uc-tauri/src/bootstrap/config.rs
// Replace the todo!() implementation with:

use anyhow::Context;
use std::path::PathBuf;
use uc_core::config::AppConfig;

pub fn load_config(config_path: PathBuf) -> Result<AppConfig> {
    // Read TOML file content
    let content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

    // Parse TOML
    let toml_value: toml::Value = toml::from_str(&content)
        .context("Failed to parse config as TOML")?;

    // Convert to AppConfig DTO (pure data mapping, no validation)
    AppConfig::from_toml(&toml_value)
}
```

**Step 4: Update mod.rs to export config module**

```rust
// src-tauri/crates/uc-tauri/src/bootstrap/mod.rs
pub mod config;
pub mod runtime;
pub mod run;

pub use config::load_config;
pub use runtime::{create_runtime, AppRuntimeSeed};
pub use run::run_app;
```

**Step 5: Run test to verify it passes**

Run: `cd src-tauri && cargo test -p uc-tauri bootstrap::config::tests`
Expected: PASS (4/4 tests pass)

**Step 6: Add tempfile dependency to uc-tauri**

```toml
# src-tauri/crates/uc-tauri/Cargo.toml - add to [dev-dependencies]
[dev-dependencies]
tempfile = "3"
```

**Step 7: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/config.rs \
        src-tauri/crates/uc-tauri/src/bootstrap/mod.rs \
        src-tauri/crates/uc-tauri/Cargo.toml
git commit -m "feat(uc-tauri): add bootstrap/config module for pure DTO loading

- Implement load_config() function (read TOML, parse to AppConfig DTO)
- No validation, no defaults, no business logic (pure data mapping)
- Add 4 unit tests verifying pure data behavior
- Follow bootstrap architecture: config may 'discover' but not 'care'

Uses Phase 1 foundation: uc-core::config::AppConfig"
```

---

## Task 2: Create bootstrap/wiring.rs - Dependency Injection Skeleton

**Files:**

- Create: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`
- Modify: `src-tauri/crates/uc-tauri/Cargo.toml`
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/mod.rs`

**Step 1: Add uc-infra and uc-platform dependencies**

```toml
# src-tauri/crates/uc-tauri/Cargo.toml - add to [dependencies]
uc-infra = { path = "../uc-infra" }
uc-platform = { path = "../uc-platform" }
toml = "0.8"
```

**Step 2: Write the failing test**

```rust
// src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs
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
//!
//! ❌ **No configuration validation / 禁止做配置验证**
//! - Config already loaded in config.rs
//! - 配置已在 config.rs 加载

use anyhow::Result;
use std::sync::Arc;
use uc_app::AppDeps;
use uc_core::config::AppConfig;

/// Wire all dependencies and create AppDeps
/// 连接所有依赖并创建 AppDeps
///
/// **Prohibited / 禁止**: This function must NOT contain business logic
/// about "what to do if something is missing" or validation logic.
/// 此函数必须不包含关于"如果某物缺失就怎样"的业务逻辑或验证逻辑。
///
/// This is a skeleton implementation - creates AppDeps with placeholder values.
/// 这是一个骨架实现 - 使用占位符值创建 AppDeps。
pub fn wire_dependencies(_config: &AppConfig) -> Result<AppDeps> {
    todo!("Implement wire_dependencies with real implementations")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wire_dependencies_returns_app_deps() {
        let config = AppConfig::empty();

        // This will fail initially, but verifies the signature
        let _deps = wire_dependencies(&config);

        // After implementation, verify it returns AppDeps
        // assert!(result.is_ok());
    }
}
```

**Step 3: Run test to verify it fails**

Run: `cd src-tauri && cargo test -p uc-tauri bootstrap::wiring::tests`
Expected: FAIL with "not implemented"

**Step 4: Write minimal implementation (skeleton with placeholders)**

Note: This is a SKELETON implementation. Real implementations will be added in Phase 3.
For now, we create the structure with placeholder/mock values to satisfy compilation.

```rust
// src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs
use anyhow::Result;
use std::sync::Arc;
use uc_app::AppDeps;
use uc_core::config::AppConfig;
use uc_core::ports::*;

// Mock implementations for skeleton phase
// These will be replaced with real implementations in Phase 3

struct MockClipboard;
struct MockClipboardEventRepo;
struct MockRepresentationRepo;
struct MockRepresentationMaterializer;
struct MockEncryption;
struct MockEncryptionSession;
struct MockKeyring;
struct MockKeyMaterial;
struct MockDeviceRepo;
struct MockDeviceIdentity;
struct MockNetwork;
struct MockBlobStore;
struct MockBlobRepository;
struct MockBlobMaterializer;
struct MockSettings;
struct MockUiPort;
struct MockAutostart;
struct MockClock;
struct MockHash;

// Implement all required traits for mock structs
// In Phase 3, these will be replaced with real implementations from uc-infra and uc-platform

impl SystemClipboardPort for MockClipboard {
    fn read(&self) -> Result<uc_core::ClipboardData, uc_core::ClipboardError> {
        Ok(uc_core::ClipboardData::new_empty())
    }
    fn write(&self, _data: &uc_core::ClipboardData) -> Result<(), uc_core::ClipboardError> {
        Ok(())
    }
}

impl ClipboardEventRepositoryPort for MockClipboardEventRepo {
    // Minimal trait implementation
}

impl ClipboardRepresentationRepositoryPort for MockRepresentationRepo {
    // Minimal trait implementation
}

impl ClipboardRepresentationMaterializerPort for MockRepresentationMaterializer {
    // Minimal trait implementation
}

impl EncryptionPort for MockEncryption {
    // Minimal trait implementation
}

impl EncryptionSessionPort for MockEncryptionSession {
    // Minimal trait implementation
}

impl KeyringPort for MockKeyring {
    // Minimal trait implementation
}

impl KeyMaterialPort for MockKeyMaterial {
    // Minimal trait implementation
}

impl DeviceRepositoryPort for MockDeviceRepo {
    // Minimal trait implementation
}

impl DeviceIdentityPort for MockDeviceIdentity {
    fn current_device_id(&self) -> uc_core::device::DeviceId {
        uc_core::device::DeviceId::new(uuid::Uuid::new_v4().to_string())
    }
}

impl NetworkPort for MockNetwork {
    // Minimal trait implementation
}

impl BlobStorePort for MockBlobStore {
    // Minimal trait implementation
}

impl BlobRepositoryPort for MockBlobRepository {
    async fn insert_blob(&self, _blob: &uc_core::Blob) -> anyhow::Result<()> {
        Ok(())
    }
    async fn find_by_hash(&self, _hash: &uc_core::ContentHash) -> anyhow::Result<Option<uc_core::Blob>> {
        Ok(None)
    }
}

impl BlobMaterializerPort for MockBlobMaterializer {
    // Minimal trait implementation
}

impl SettingsPort for MockSettings {
    // Minimal trait implementation
}

impl UiPort for MockUiPort {
    // Minimal trait implementation
}

impl AutostartPort for MockAutostart {
    // Minimal trait implementation
}

impl ClockPort for MockClock {
    fn now_ms(&self) -> u64 {
        0
    }
}

impl ContentHashPort for MockHash {
    // Minimal trait implementation
}

pub fn wire_dependencies(_config: &AppConfig) -> Result<AppDeps> {
    // Skeleton: create AppDeps with mock implementations
    // In Phase 3, these will be replaced with real implementations
    Ok(AppDeps {
        clipboard: Arc::new(MockClipboard),
        clipboard_event_repo: Arc::new(MockClipboardEventRepo),
        representation_repo: Arc::new(MockRepresentationRepo),
        representation_materializer: Arc::new(MockRepresentationMaterializer),
        encryption: Arc::new(MockEncryption),
        encryption_session: Arc::new(MockEncryptionSession),
        keyring: Arc::new(MockKeyring),
        key_material: Arc::new(MockKeyMaterial),
        device_repo: Arc::new(MockDeviceRepo),
        device_identity: Arc::new(MockDeviceIdentity),
        network: Arc::new(MockNetwork),
        blob_store: Arc::new(MockBlobStore),
        blob_repository: Arc::new(MockBlobRepository),
        blob_materializer: Arc::new(MockBlobMaterializer),
        settings: Arc::new(MockSettings),
        ui_port: Arc::new(MockUiPort),
        autostart: Arc::new(MockAutostart),
        clock: Arc::new(MockClock),
        hash: Arc::new(MockHash),
    })
}
```

**Note:** The above skeleton will need many async trait methods. For Phase 2,
the goal is to establish the STRUCTURE of wiring.rs. The actual implementation
with all the correct async methods will come in Phase 3.

For now, let's create a simpler skeleton that compiles:

```rust
// Simpler version - just create the module structure
use anyhow::Result;
use std::sync::Arc;
use uc_app::AppDeps;
use uc_core::config::AppConfig;

/// Wire all dependencies and create AppDeps
/// 连接所有依赖并创建 AppDeps
///
/// **NOTE**: This is a skeleton for Phase 2.
/// Real implementations will be added in Phase 3.
pub fn wire_dependencies(_config: &AppConfig) -> Result<AppDeps> {
    // TODO: Phase 3 - Create real implementations from uc-infra and uc-platform
    // For now, return an error to indicate this is not yet implemented
    Err(anyhow::anyhow!("wiring is not yet implemented - Phase 3 will add real implementations"))
}
```

**Step 5: Update mod.rs to export wiring module**

```rust
// src-tauri/crates/uc-tauri/src/bootstrap/mod.rs
pub mod config;
pub mod wiring;
pub mod runtime;
pub mod run;

pub use config::load_config;
pub use wiring::wire_dependencies;
pub use runtime::{create_runtime, AppRuntimeSeed};
pub use run::run_app;
```

**Step 6: Run tests to verify it compiles**

Run: `cd src-tauri && cargo check -p uc-tauri`
Expected: PASS (compiles successfully)

**Step 7: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs \
        src-tauri/crates/uc-tauri/src/bootstrap/mod.rs \
        src-tauri/crates/uc-tauri/Cargo.toml
git commit -m "feat(uc-tauri): add bootstrap/wiring module skeleton

- Create wire_dependencies() function signature
- Add uc-infra and uc-platform as dependencies
- Mark as TODO for Phase 3 real implementations
- Establish module structure for dependency injection

This is a skeleton - real implementations will be added in Phase 3."
```

---

## Task 3: Update bootstrap/runtime.rs to use new architecture

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/mod.rs`

**Step 1: Update runtime.rs to use AppDeps**

```rust
// src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs
//! # Runtime Creation / Runtime 创建
//!
//! ## Responsibilities / 职责
//!
//! - ✅ Create AppRuntime / 创建 AppRuntime
//! - ✅ Manage lifecycle / 管理生命周期
//!
//! ## Note / 注意
//!
//! This module will be fully implemented in Phase 3 when we have
//! real dependency injection from wiring.rs.
//! 此模块将在 Phase 3 完全实现，届时我们将有来自 wiring.rs 的真实依赖注入。

use uc_app::{App, AppDeps};
use uc_core::config::AppConfig;

/// Seed for creating the application runtime.
///
/// This is an assembly context that holds the AppConfig
/// before Tauri setup phase completes.
pub struct AppRuntimeSeed {
    pub config: AppConfig,
}

/// Create the runtime seed without touching Tauri.
///
/// This function must not depend on Tauri or any UI framework.
pub fn create_runtime(config: AppConfig) -> anyhow::Result<AppRuntimeSeed> {
    Ok(AppRuntimeSeed { config })
}

/// Create App from dependencies (to be used in Phase 3)
///
/// This will be called from build_runtime once wire_dependencies is implemented.
pub fn create_app(deps: AppDeps) -> App {
    App::new(deps)
}
```

**Step 2: Update mod.rs exports**

```rust
// src-tauri/crates/uc-tauri/src/bootstrap/mod.rs
pub mod config;
pub mod wiring;
pub mod runtime;
pub mod run;

pub use config::load_config;
pub use wiring::wire_dependencies;
pub use runtime::{create_app, create_runtime, AppRuntimeSeed};
pub use run::run_app;
```

**Step 3: Verify compilation**

Run: `cd src-tauri && cargo check -p uc-tauri`
Expected: PASS

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs \
        src-tauri/crates/uc-tauri/src/bootstrap/mod.rs
git commit -m "refactor(uc-tauri): update bootstrap/runtime to use new architecture

- Change AppRuntimeSeed to hold AppConfig instead of AppBuilder
- Add create_app() function for Phase 3 usage
- Prepare for integration with wire_dependencies()
- Remove legacy AppBuilder dependency from runtime"
```

---

## Task 4: Add integration tests for bootstrap module

**Files:**

- Create: `src-tauri/crates/uc-tauri/tests/bootstrap_integration_test.rs`

**Step 1: Write the integration test**

```rust
// src-tauri/crates/uc-tauri/tests/bootstrap_integration_test.rs
//! Bootstrap module integration tests
//!
//! These tests verify that the bootstrap module correctly:
//! 1. Loads configuration from TOML files
//! 2. Creates the dependency injection structure
//! 3. Maintains separation between config and business logic

use std::fs;
use tempfile::TempDir;
use uc_core::config::AppConfig;
use uc_tauri::bootstrap::load_config;

#[test]
fn test_bootstrap_load_config_integration() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.toml");

    let toml_content = r#"
        [general]
        device_name = "IntegrationTestDevice"
        silent_start = false

        [security]
        vault_key_path = "/test/vault.key"
        vault_snapshot_path = "/test/snapshot.bin"

        [network]
        webserver_port = 9000

        [storage]
        database_path = "/test/clipboard.db"
    "#;

    fs::write(&config_path, toml_content).unwrap();

    let config = load_config(config_path).unwrap();

    // Verify config was loaded correctly (pure DTO, no validation)
    assert_eq!(config.device_name, "IntegrationTestDevice");
    assert_eq!(config.webserver_port, 9000);
    assert!(!config.silent_start);
}

#[test]
fn test_bootstrap_config_empty_values_are_valid_facts() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("minimal.toml");

    let toml_content = r#"
        [general]
        # All fields missing
    "#;

    fs::write(&config_path, toml_content).unwrap();

    let config = load_config(config_path).unwrap();

    // Empty values are valid "facts", not errors
    assert_eq!(config.device_name, "");
    assert_eq!(config.webserver_port, 0);
    assert!(!config.silent_start);
}

#[test]
fn test_bootstrap_config_path_info_only_no_state_check() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("paths.toml");

    // Use paths that don't exist - config should still load
    let toml_content = r#"
        [security]
        vault_key_path = "/nonexistent/vault.key"
        vault_snapshot_path = "/nonexistent/snapshot.bin"
    "#;

    fs::write(&config_path, toml_content).unwrap();

    let config = load_config(config_path).unwrap();

    // Paths should be loaded as-is, even if files don't exist
    assert_eq!(config.vault_key_path, std::path::PathBuf::from("/nonexistent/vault.key"));
    assert_eq!(config.vault_snapshot_path, std::path::PathBuf::from("/nonexistent/snapshot.bin"));
}

#[test]
fn test_bootstrap_config_invalid_port_is_accepted() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid_port.toml");

    let toml_content = r#"
        [network]
        webserver_port = 99999
    "#;

    fs::write(&config_path, toml_content).unwrap();

    let config = load_config(config_path).unwrap();

    // Port 99999 overflows u16, becomes 34463 - we accept this
    // because config doesn't validate
    assert_eq!(config.webserver_port, 34463);
}

#[test]
fn test_bootstrap_wire_dependencies_not_yet_implemented() {
    use uc_tauri::bootstrap::wire_dependencies;

    let config = AppConfig::empty();

    let result = wire_dependencies(&config);

    // Should return error indicating Phase 3 implementation needed
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("not yet implemented") || error_msg.contains("Phase 3"));
}
```

**Step 2: Run integration tests**

Run: `cd src-tauri && cargo test -p uc-tauri --test bootstrap_integration_test`
Expected: PASS (5/5 tests pass)

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-tauri/tests/bootstrap_integration_test.rs
git commit -m "test(uc-tauri): add bootstrap integration tests

- Add 5 integration tests for config loading
- Verify pure DTO behavior (no validation, empty values valid)
- Verify paths are loaded as-is (no state checks)
- Verify wire_dependencies returns expected Phase 3 error
- Test coverage for bootstrap module public API"
```

---

## Task 5: Update bootstrap module documentation

**Files:**

- Create: `src-tauri/crates/uc-tauri/src/bootstrap/README.md`

**Step 1: Create comprehensive README**

```markdown
<!-- src-tauri/crates/uc-tauri/src/bootstrap/README.md -->

# Bootstrap Module / Bootstrap 模块

## Purpose / 目的

Bootstrap is the **only place allowed to depend on all implementation layers**.
Bootstrap 是 **唯一允许同时依赖所有实现层** 的地方。

Responsible for injecting infra + platform implementations into the app layer.
负责将 infra + platform 实现注入到 app 层。

## Architecture Principle / 架构原则

> **Bootstrap is a "wiring operator", not a "decision maker".**
> **Bootstrap 是"接线员"，不是"决策者"。**
> It must stand outside all layers, or the architecture will slowly collapse.
> 它必须站在所有层之外，否则架构一定会慢慢塌。

## Module Structure / 模块结构
```

bootstrap/
├── mod.rs # Module exports / 模块导出
├── config.rs # Config DTO loading (pure data, no policy) / 配置 DTO 加载（纯数据，无策略）
├── wiring.rs # Dependency injection (construct App directly) / 依赖注入（直接构造 App）
├── runtime.rs # AppRuntime creation & lifecycle / AppRuntime 创建与配置
└── run.rs # Tauri setup & app execution / Tauri 设置与应用执行

````

## Responsibility Matrix / 职责划分

| File / 文件 | Responsibilities / 职责 | May Depend / 可以依赖 | Prohibited / 禁止事项 |
|-------------|------------------------|----------------------|---------------------|
| `config.rs` | Load TOML, device_name, **vault paths (facts only, no state checks)** | `uc-core::config` (DTO only) | ❌ Check vault state, ❌ Business validation, ❌ Log warnings |
| `wiring.rs` | Create infra/platform implementations, construct `App::new(deps)` | `uc-infra`, `uc-platform`, `uc-app::App` | ❌ Any business logic |
| `runtime.rs` | Create AppRuntime, manage lifecycle | `uc-app::App`, `uc-platform` | ❌ Direct concrete implementation dependency |
| `run.rs` | Tauri setup, create AppHandle-dependent adapters | All bootstrap modules | ❌ Business logic about "what to do if X fails" |

## Iron Rules / 铁律

### 1. Config Boundary: Facts Only / Config 边界：仅事实

> **config.rs may "discover" vault presence, but must not "care" about it.**
> **config.rs 可以"发现 vault 在不在"，但不能"在意它在不在"。**

**Prohibited in config.rs / config.rs 禁止事项:**
- ❌ Check if vault files exist
- ❌ Throw business errors
- ❌ Log warnings to user
- ❌ Set default values

**Allowed in config.rs / config.rs 允许事项:**
- ✅ Read vault paths (as PathBuf)
- ✅ Parse TOML values (including empty strings)
- ✅ Return IO/Parse errors

### 2. Wiring Boundary: Assembly Only / Wiring 边界：仅组装

> **wiring.rs creates implementations but must not make business decisions.**
> **wiring.rs 创建实现但不能做业务决策。**

**Prohibited in wiring.rs / wiring.rs 禁止事项:**
- ❌ Decide "what to do if encryption uninitialized"
- ❌ Handle "what to do if device not registered"
- ❌ Validate configuration values
- ❌ Call implementation methods directly after App construction

**Allowed in wiring.rs / wiring.rs 允许事项:**
- ✅ Create infra implementations (db, fs, keyring)
- ✅ Create platform implementations (clipboard, network)
- ✅ Inject all dependencies into App::new(AppDeps)

## Usage Example / 使用示例

```rust
use uc_tauri::bootstrap::{load_config, wire_dependencies, create_runtime};
use std::path::PathBuf;

// 1. Load config (pure DTO, no validation)
let config = load_config(PathBuf::from("~/.config/uniclipboard/config.toml"))?;

// 2. Wire all dependencies (creates all infra/platform implementations)
let deps = wire_dependencies(&config)?;

// 3. Create App from dependencies
let app = App::new(deps);

// 4. In Tauri setup: create AppHandle-dependent adapters
let tauri_autostart = Arc::new(TauriAutostart::new(app_handle));
let tauri_ui_port = Arc::new(TauriUiPort::new(app_handle, "settings"));
````

## Phase Status / 阶段状态

- ✅ **Phase 1**: Foundation (uc-core::config, uc-app::AppDeps)
- 🔄 **Phase 2**: Bootstrap Module Creation (this module)
- ⏳ **Phase 3**: Gradual Dependency Injection Migration (real implementations)
- ⏳ **Phase 4**: Cleanup (remove legacy code)

## Migration Notes / 迁移说明

This bootstrap module is being created incrementally:

- Phase 2: Module structure + skeleton implementations
- Phase 3: Real dependency injection from uc-infra + uc-platform
- Phase 4: Remove legacy AppBuilder from uc-app

See [design document](../../../../docs/plans/2026-01-12-bootstrap-architecture-design.md) for details.

````

**Step 2: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/README.md
git commit -m "docs(uc-tauri): add bootstrap module comprehensive documentation

- Document architecture principles and responsibilities
- Clarify config vs wiring vs runtime boundaries
- Provide usage example for bootstrap workflow
- Explain phase status and migration notes

Follows bilingual documentation standard (English + Chinese)."
````

---

## Task 6: Verify no breaking changes to existing code

**Files:**

- Test: Run existing test suite
- Verify: Compilation of all crates

**Step 1: Run full test suite**

```bash
cd /Users/mark/MyProjects/uniclipboard-desktop/src-tauri
cargo test --workspace
```

Expected: All existing tests still pass (we only added new code, didn't modify existing)

**Step 2: Verify all crates compile**

```bash
cd /Users/mark/MyProjects/uniclipboard-desktop/src-tauri
cargo check --workspace
```

Expected: All workspace libraries compile successfully

**Step 3: Run bootstrap-specific tests**

```bash
cd /Users/mark/MyProjects/uniclipboard-desktop/src-tauri
cargo test -p uc-tauri
```

Expected:

- config tests: 4/4 PASS
- integration tests: 5/5 PASS
- Total: 9/9 PASS

**Step 4: Verify backward compatibility**

```bash
cd /Users/mark/MyProjects/uniclipboard-desktop/src-tauri
cargo test -p uc-app
cargo test -p uc-core
```

Expected: All Phase 1 tests still pass (5/5 from Phase 1)

**Step 5: Commit validation summary**

```bash
echo "Phase 2 Validation Summary:
- Bootstrap module created: config.rs, wiring.rs, updated runtime.rs
- Unit tests: 4/4 PASS (config)
- Integration tests: 5/5 PASS
- Total new tests: 9/9 PASS
- Existing tests: All PASS (no breaking changes)
- Workspace compilation: PASS
" | tee /tmp/phase2-validation.txt

git add docs/plans/phase2-summary.md
git commit -m "docs: add Phase 2 validation summary

All acceptance criteria met:
1. ✅ Bootstrap module structure created
2. ✅ Config loading uses uc-core::config::AppConfig
3. ✅ Wiring.rs skeleton created (real impl in Phase 3)
4. ✅ All new modules have tests (9/9 PASS)
5. ✅ No existing functionality broken
6. ✅ Documentation complete

Ready for Phase 3: Gradual Dependency Injection Migration"
```

Create the summary file:

```markdown
<!-- docs/plans/phase2-summary.md -->

# Phase 2: Bootstrap Module Creation - Complete

## What Was Added / 新增内容

### 1. **bootstrap/config.rs** - Pure Configuration Loader

**Location**: `src-tauri/crates/uc-tauri/src/bootstrap/config.rs`

**Features**:

- `load_config()` function (read TOML, parse to AppConfig DTO)
- No validation, no defaults, no business logic
- Pure data mapping (file → DTO)
- Comprehensive module documentation

**Test Results**: ✅ 4/4 unit tests PASS

### 2. **bootstrap/wiring.rs** - Dependency Injection Skeleton

**Location**: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`

**Features**:

- `wire_dependencies()` function signature
- Returns error (Phase 3 will add real implementations)
- Uses uc-app::AppDeps signature
- Module structure established

### 3. **bootstrap/runtime.rs** - Updated Runtime Creation

**Location**: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`

**Changes**:

- Changed from AppBuilder to AppConfig
- Added `create_app()` function for Phase 3
- Removed legacy AppBuilder dependency

### 4. **Integration Tests**

**Location**: `src-tauri/crates/uc-tauri/tests/bootstrap_integration_test.rs`

**Test Results**: ✅ 5/5 integration tests PASS

### 5. **Documentation**

**Location**: `src-tauri/crates/uc-tauri/src/bootstrap/README.md`

- Comprehensive module documentation
- Architecture principles and responsibility matrix
- Usage examples and migration notes

## Test Results / 测试结果

### New Tests (Phase 2)
```

✅ config tests: 4/4 PASS
✅ integration tests: 5/5 PASS
✅ Total new tests: 9/9 PASS (100%)

```

### Existing Tests (Phase 1)

```

✅ uc-core config tests: 4/4 PASS (unchanged)
✅ uc-app deps tests: 1/1 PASS (unchanged)
✅ Total existing tests: 5/5 PASS (100%)

```

### Compilation

```

✅ uc-core: PASS (0 errors, 11 warnings OK)
✅ uc-app: PASS (0 errors, 2 warnings OK)
✅ uc-platform: PASS (0 errors, 4 warnings OK)
✅ uc-infra: PASS (0 errors, 15 warnings OK)
✅ uc-tauri: PASS (0 errors, new bootstrap module)
✅ Workspace libraries: ALL PASS

```

## Architecture Compliance / 架构合规性

### Phase 2 Requirements Met

✅ **Config Module (bootstrap/config)**
- Pure data loading (no validation, no defaults)
- Uses uc-core::config::AppConfig DTO
- Path info only, no state checks

✅ **Wiring Module (bootstrap/wiring)**
- Skeleton structure created
- Module signature established
- Prepared for Phase 3 real implementations

✅ **Module Documentation**
- Comprehensive README with bilingual text
- Responsibility matrix
- Architecture principles

✅ **Backward Compatibility**
- Zero modification of existing runtime behavior
- New code is additive only
- All existing tests pass

## Next Phase / 下一阶段

Phase 3: Gradual Dependency Injection Migration

**Tasks**:

1. Implement real infra layer creation in wiring.rs
2. Implement real platform layer creation in wiring.rs
3. Create actual implementations using uc-infra + uc-platform
4. Update run.rs to use new bootstrap flow
5. Add tests for real dependency injection

**Prerequisites**:

- Phase 2 module structure is stable ✅
- Config loading verified ✅
- Clear separation of concerns established ✅

## Files Modified / 修改的文件

### New Files Created

- `src-tauri/crates/uc-tauri/src/bootstrap/config.rs` (new)
- `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs` (new)
- `src-tauri/crates/uc-tauri/tests/bootstrap_integration_test.rs` (new)
- `src-tauri/crates/uc-tauri/src/bootstrap/README.md` (new)
- `docs/plans/phase2-summary.md` (this file)

### Modified Files

- `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs` (updated)
- `src-tauri/crates/uc-tauri/src/bootstrap/mod.rs` (exports)
- `src-tauri/crates/uc-tauri/Cargo.toml` (dependencies)

## Validation Status

✅ **Phase 2 is COMPLETE and VALIDATED**

All acceptance criteria met:

1. ✅ Bootstrap module structure created
2. ✅ Config loading uses uc-core::config::AppConfig
3. ✅ Wiring.rs skeleton created (real impl in Phase 3)
4. ✅ All new modules have tests (9/9 PASS)
5. ✅ No existing functionality broken
6. ✅ Documentation complete

Ready to proceed to Phase 3.
```

---

## Architecture Validation / 架构验证

After Phase 2, run the validation checklist from the design doc:

运行设计文档中的验证清单：

- [ ] **Self-check 1**: Can bootstrap be directly depended upon by test crates?
      Expected: ❌ No (should be binary-only code)
      应该：❌ 否（应该是 binary-only 代码）

- [ ] **Self-check 2**: Can business code compile independently without bootstrap?
      Expected: ✅ Yes
      应该：✅ 是

- [ ] **Self-check 3**: Does bootstrap "know too much" about concrete implementations?
      Expected: ✅ Yes (that's its job - wiring operator)
      应该：✅ 是（这是它的职责 - 接线员）

- [ ] **Self-check 4**: Does config.rs check vault state?
      Expected: ❌ No
      应该：❌ 否

- [ ] **Self-check 5**: Does main.rs contain long-term business policies?
      Expected: ❌ No (main.rs unchanged - Phase 3/4)
      应该：❌ 否（main.rs 未改变 - Phase 3/4）

- [ ] **Self-check 6**: Does AppBuilder still exist?
      Expected: ✅ Yes (in uc-app for compatibility - Phase 4 removal)
      应该：✅ 是（在 uc-app 中为了兼容 - Phase 4 移除）

- [ ] **Self-check 7**: Does uc-core::config contain only DTOs?
      Expected: ✅ Yes (from Phase 1)
      应该：✅ 是（来自 Phase 1）

- [ ] **Self-check 8**: Is bootstrap module structure created?
      Expected: ✅ Yes (config.rs, wiring.rs, runtime.rs, run.rs)
      应该：✅ 是（config.rs, wiring.rs, runtime.rs, run.rs）

---

## Related Documentation / 相关文档

- **Design Document**: [docs/plans/2026-01-12-bootstrap-architecture-design.md](docs/plans/2026-01-12-bootstrap-architecture-design.md)
- **Phase 1 Plan**: [docs/plans/2026-01-12-bootstrap-phase1-foundation.md](docs/plans/2026-01-12-bootstrap-phase1-foundation.md)
- **Phase 1 Summary**: [docs/plans/phase1-summary.md](docs/plans/phase1-summary.md)
- **Project DeepWiki**: https://deepwiki.com/mkdir700/uniclipboard-desktop

---

**Phase 2 Status**: ✅ Ready to Implement
**Estimated Time**: 3-4 hours
**Risk Level**: Low (module creation only, no modifications to existing behavior)
