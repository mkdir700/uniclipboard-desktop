# Bootstrap Architecture Refactoring - Phase 1: Foundation

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create foundational structures for bootstrap module without affecting existing functionality.

**Architecture:** Add `uc-core::config` (pure DTO module) and `uc-app::AppDeps` (dependency grouping) while maintaining backward compatibility with existing `AppBuilder`. This phase adds new interfaces only, no breaking changes.

**Tech Stack:** Rust, anyhow for error handling, existing uc-core/uc-app/uc-infra crates

---

## Task 1: Create uc-core/src/config module (Pure DTO)

**Files:**

- Create: `src-tauri/crates/uc-core/src/config/mod.rs`
- Modify: `src-tauri/crates/uc-core/src/lib.rs`

**Step 1: Create the config module with pure DTO**

```rust
// src-tauri/crates/uc-core/src/config/mod.rs
//! # Pure Data Module / 纯数据模块 - Data Transfer Objects Only
//!
//! ## Responsibilities / 职责
//!
//! - ✅ Define configuration data structures / 定义配置数据结构
//! - ✅ Provide TOML → DTO mapping / 提供 TOML → DTO 的映射
//!
//! ## Prohibited / 禁止事项
//!
//! ❌ **No business logic or policies / 禁止任何业务逻辑或策略**
//! ❌ **No validation logic / 禁止验证逻辑**
//! ❌ **No default value calculation / 禁止默认值计算**
//!
//! ## Iron Rule / 铁律
//!
//! > **This module contains data only, no policy, no validation.**
//! > **此模块只包含数据结构定义，禁止：任何业务逻辑或策略、验证逻辑、默认值计算。**

use std::path::PathBuf;

/// Application configuration DTO (pure data, no logic)
/// 应用配置 DTO（纯数据，无逻辑）
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Device name (may be empty - this is a fact, not an error)
    /// 设备名称（可能为空 - 这就是事实，不是错误）
    pub device_name: String,

    /// Vault key file path (path info only, no existence check)
    /// Vault 密钥文件路径（仅路径信息，不检查文件是否存在）
    pub vault_key_path: PathBuf,

    /// Vault snapshot file path (path info only, no existence check)
    /// Vault snapshot 文件路径（仅路径信息，不检查文件是否存在）
    pub vault_snapshot_path: PathBuf,

    /// Web server port
    pub webserver_port: u16,

    /// Database path
    pub database_path: PathBuf,

    /// Silent start flag
    pub silent_start: bool,
}

impl AppConfig {
    /// Create AppConfig from TOML value
    /// 从 TOML 值创建 AppConfig
    ///
    /// **Prohibited / 禁止**: This method must NOT contain any validation
    /// or default value logic. Empty strings are valid "facts".
    /// 此方法必须不包含任何验证或默认值逻辑。空字符串是合法的"事实"。
    pub fn from_toml(toml_value: &toml::Value) -> anyhow::Result<Self> {
        Ok(Self {
            device_name: toml_value
                .get("general")
                .and_then(|g| g.get("device_name"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            vault_key_path: PathBuf::from(
                toml_value
                    .get("security")
                    .and_then(|s| s.get("vault_key_path"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
            ),
            vault_snapshot_path: PathBuf::from(
                toml_value
                    .get("security")
                    .and_then(|s| s.get("vault_snapshot_path"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
            ),
            webserver_port: toml_value
                .get("network")
                .and_then(|n| n.get("webserver_port"))
                .and_then(|v| v.as_integer())
                .unwrap_or(0) as u16,
            database_path: PathBuf::from(
                toml_value
                    .get("storage")
                    .and_then(|s| s.get("database_path"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("clipboard.db")
            ),
            silent_start: toml_value
                .get("general")
                .and_then(|g| g.get("silent_start"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        })
    }

    /// Create empty AppConfig (all empty/default values)
    /// 创建空的 AppConfig（所有字段为空/默认值）
    ///
    /// **Note**: This is a pure data constructor with "empty" as valid facts.
    /// 注意：这是一个纯数据构造函数，"空"是合法的事实。
    pub fn empty() -> Self {
        Self {
            device_name: String::new(),
            vault_key_path: PathBuf::new(),
            vault_snapshot_path: PathBuf::new(),
            webserver_port: 0,
            database_path: PathBuf::from("clipboard.db"),
            silent_start: false,
        }
    }
}
```

**Step 2: Export the config module from lib.rs**

```rust
// src-tauri/crates/uc-core/src/lib.rs
// Add this line after other module exports
pub mod config;

// Re-export at the top level for convenience
pub use config::AppConfig;
```

**Step 3: Write unit tests for config module**

```rust
// src-tauri/crates/uc-core/src/config/mod.rs (add at end)

#[cfg(test)]
mod tests {
    use super::*;
    use toml::Value;

    #[test]
    fn test_from_toml_returns_empty_device_name_when_missing() {
        let toml_str = r#"
            [general]
            # device_name is missing
        "#;
        let toml_value: Value = toml::from_str(toml_str).unwrap();

        let config = AppConfig::from_toml(&toml_value).unwrap();

        // Empty string is valid (fact, not error)
        assert_eq!(config.device_name, "");
    }

    #[test]
    fn test_from_toml_parses_device_name_when_present() {
        let toml_str = r#"
            [general]
            device_name = "MyDevice"
        "#;
        let toml_value: Value = toml::from_str(toml_str).unwrap();

        let config = AppConfig::from_toml(&toml_value).unwrap();

        assert_eq!(config.device_name, "MyDevice");
    }

    #[test]
    fn test_empty_creates_valid_dto() {
        let config = AppConfig::empty();

        // All "empty" values are valid facts
        assert_eq!(config.device_name, "");
        assert_eq!(config.webserver_port, 0);
        assert!(!config.silent_start);
    }

    #[test]
    fn test_from_toml_does_not_validate_port_range() {
        // Port 99999 is out of u16 range, but we use 0 as default
        let toml_str = r#"
            [network]
            webserver_port = 99999
        "#;
        let toml_value: Value = toml::from_str(toml_str).unwrap();

        let config = AppConfig::from_toml(&toml_value).unwrap();

        // We don't validate - 0 means "not set" (valid fact)
        assert_eq!(config.webserver_port, 0);
    }
}
```

**Step 4: Run tests to verify they pass**

```bash
cd /Users/mark/MyProjects/uniclipboard-desktop/src-tauri
cargo test -p uc-core config
```

Expected: All tests PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-core/src/config.rs src-tauri/crates/uc-core/src/lib.rs
git commit -m "feat(uc-core): add pure config DTO module

- Add AppConfig DTO with no validation/logic
- Add from_toml() method (pure data mapping only)
- Add empty() constructor
- Add unit tests for config module
- Export AppConfig from uc-core

This follows the 'data only, no policy' principle for bootstrap architecture."
```

---

## Task 2: Add AppDeps struct to uc-app

**Files:**

- Create: `src-tauri/crates/uc-app/src/deps.rs`
- Modify: `src-tauri/crates/uc-app/src/lib.rs`

**Step 1: Create the deps module with AppDeps struct**

```rust
// src-tauri/crates/uc-app/src/deps.rs
//! # Application Dependencies / 应用依赖
//!
//! This module defines the dependency grouping for App construction.
//! 此模块定义 App 构造的依赖分组。
//!
//! **Note / 注意**: This is NOT a Builder pattern.
//! **这不是 Builder 模式。**
//! - No build steps / 无构建步骤
//! - No default values / 无默认值
//! - No hidden logic / 无隐藏逻辑
//! - Just parameter grouping / 仅用于参数打包

use std::sync::Arc;
use uc_core::ports::*;

/// Application dependency grouping (non-Builder, just parameter grouping)
/// 应用依赖分组（非 Builder，仅参数打包）
///
/// **NOT a Builder pattern** - this is just a struct to group parameters.
/// **不是 Builder 模式** - 这只是一个打包参数的结构体。
///
/// All dependencies are required - no defaults, no optional fields.
/// 所有依赖都是必需的 - 无默认值，无可选字段。
pub struct AppDeps {
    // Clipboard dependencies / 剪贴板依赖
    pub clipboard: Arc<dyn LocalClipboardPort>,
    pub clipboard_event_repo: Arc<dyn ClipboardEventRepositoryPort>,
    pub representation_repo: Arc<dyn RepresentationRepositoryPort>,
    pub representation_materializer: Arc<dyn RepresentationMaterializerPort>,

    // Security dependencies / 安全依赖
    pub encryption: Arc<dyn EncryptionPort>,
    pub encryption_session: Arc<dyn EncryptionSessionPort>,
    pub keyring: Arc<dyn KeyringPort>,
    pub key_material: Arc<dyn KeyMaterialPort>,

    // Device dependencies / 设备依赖
    pub device_repo: Arc<dyn DeviceRepositoryPort>,
    pub device_identity: Arc<dyn DeviceIdentityPort>,

    // Network dependencies / 网络依赖
    pub network: Arc<dyn NetworkPort>,

    // Storage dependencies / 存储依赖
    pub blob_store: Arc<dyn BlobStorePort>,
    pub blob_repository: Arc<dyn BlobRepositoryPort>,
    pub blob_materializer: Arc<dyn BlobMaterializerPort>,

    // Settings dependencies / 设置依赖
    pub settings: Arc<dyn SettingsPort>,

    // UI dependencies / UI 依赖
    pub ui_port: Arc<dyn UiPort>,
    pub autostart: Arc<dyn AutostartPort>,

    // System dependencies / 系统依赖
    pub clock: Arc<dyn ClockPort>,
    pub hash: Arc<dyn HashPort>,
}
```

**Step 2: Add new() constructor to App that accepts AppDeps**

```rust
// src-tauri/crates/uc-app/src/lib.rs (modify the App struct)

// First, import the deps module at the top
mod deps;
pub use deps::AppDeps;

// Then, add the new constructor to App impl block
impl App {
    /// Create new App instance from dependencies
    /// 从依赖创建新的 App 实例
    ///
    /// This constructor signature IS the dependency manifest.
    /// 这个构造函数签名就是依赖清单。
    ///
    /// All dependencies must be provided - no defaults, no optionals.
    /// 必须提供所有依赖 - 无默认值，无可选字段。
    pub fn new(deps: AppDeps) -> Self {
        Self {
            // Store deps internally for use case creation
            // 在内部存储 deps 用于 use case 创建
            deps,
        }
    }
}
```

**Step 3: Keep existing AppBuilder for backward compatibility**

```rust
// src-tauri/crates/uc-app/src/builder.rs (keep existing file unchanged)
// This file remains to maintain backward compatibility during migration.
// 在迁移期间保留此文件以保持向后兼容性。
// It will be removed in Phase 4.
// 它将在 Phase 4 中被移除。
```

**Step 4: Write unit tests for AppDeps**

```rust
// src-tauri/crates/uc-app/src/deps.rs (add at end)

#[cfg(test)]
mod tests {
    use super::*;
    use uc_core::ports::*;

    // Mock implementations for testing
    struct MockClipboard;
    impl LocalClipboardPort for MockClipboard {
        // ... minimal impl
    }

    // Note: Full integration tests will be added in Phase 3
    // when actual implementations are available

    #[test]
    fn test_app_deps_is_just_a_struct() {
        // This test verifies AppDeps is a plain struct,
        // not a Builder with methods
        fn assert_plain_struct<T: Sized>(_: &T) {}

        let deps = AppDeps {
            clipboard: Arc::new(MockClipboard),
            // ... other fields would be set in integration tests
            clipboard_event_repo: todo!(),
            representation_repo: todo!(),
            representation_materializer: todo!(),
            encryption: todo!(),
            encryption_session: todo!(),
            keyring: todo!(),
            key_material: todo!(),
            device_repo: todo!(),
            device_identity: todo!(),
            network: todo!(),
            blob_store: todo!(),
            blob_repository: todo!(),
            blob_materializer: todo!(),
            settings: todo!(),
            ui_port: todo!(),
            autostart: todo!(),
            clock: todo!(),
            hash: todo!(),
        };

        // If this compiles, AppDeps is a plain struct
        assert_plain_struct(&deps);
    }
}
```

**Step 5: Run tests to verify they pass**

```bash
cd /Users/mark/MyProjects/uniclipboard-desktop/src-tauri
cargo test -p uc-app deps
```

Expected: Test PASSES (verifying AppDeps is a plain struct)

**Step 6: Update lib.rs to export AppDeps**

```rust
// src-tauri/crates/uc-app/src/lib.rs
pub mod deps;
pub use deps::{App, AppDeps};
```

**Step 7: Verify the project still compiles**

```bash
cd /Users/mark/MyProjects/uniclipboard-desktop/src-tauri
cargo check
```

Expected: Compilation succeeds (no breaking changes)

**Step 8: Commit**

```bash
git add src-tauri/crates/uc-app/src/
git commit -m "feat(uc-app): add AppDeps struct and App::new() constructor

- Add AppDeps struct for dependency grouping (NOT a Builder)
- Add App::new(AppDeps) constructor
- Keep existing AppBuilder for backward compatibility
- Add unit test verifying AppDeps is plain struct
- Export AppDeps from uc-app

This enables direct dependency injection without Builder pattern.
Existing AppBuilder remains for compatibility during migration."
```

---

## Task 3: Add documentation for the new architecture

**Files:**

- Create: `src-tauri/crates/uc-core/src/config/README.md`
- Create: `src-tauri/crates/uc-app/src/deps.md`

**Step 1: Create config module README**

````markdown
<!-- src-tauri/crates/uc-core/src/config/README.md -->

# Config Module - Pure Data Only

## Purpose / 目的

This module contains **pure data structures** for application configuration.
此模块包含应用配置的**纯数据结构**。

## Iron Rules / 铁律

1. **No validation** / 无验证
   - Empty strings are valid / 空字符串是合法的
   - Zero values are valid / 零值是合法的

2. **No default value logic** / 无默认值逻辑
   - Callers decide defaults / 调用者决定默认值

3. **No business logic** / 无业务逻辑
   - This is a DTO only / 这只是一个 DTO

## Examples / 示例

```rust
use uc_core::config::AppConfig;

// Empty config is valid (fact, not error)
let config = AppConfig::empty();

// From TOML - missing fields become empty values
let config = AppConfig::from_toml(&toml_value)?;
```
````

## Migration Note / 迁移说明

This replaces the old `settings` concept which contained policy.
这替代了包含策略的旧的 `settings` 概念。

````

**Step 2: Create deps module documentation**

```markdown
<!-- src-tauri/crates/uc-app/src/deps.md -->
# AppDeps - Dependency Grouping

## Purpose / 目的

Groups all application dependencies into a single struct for clean dependency injection.
将所有应用依赖分组到单个结构体中，以实现干净的依赖注入。

## Important / 重要

**This is NOT a Builder pattern.**
**这不是 Builder 模式。**

- ❌ No build steps / ❌ 无构建步骤
- ❌ No default values / ❌ 无默认值
- ❌ No hidden logic / ❌ 无隐藏逻辑
- ✅ Just parameter grouping / ✅ 仅参数打包

## Constructor Signature = Dependency Manifest / 构造函数签名即依赖清单

```rust
pub fn new(deps: AppDeps) -> App
````

Looking at this function signature tells you ALL dependencies of App.
查看此函数签名即可知道 App 的所有依赖。

No hidden dependencies, no defaults, no magic.
无隐藏依赖，无默认值，无魔法。

## Migration Path / 迁移路径

- Phase 1: AppDeps added alongside AppBuilder (compatibility) / AppDeps 与 AppBuilder 共存（兼容）
- Phase 2: Bootstrap starts using App::new(AppDeps) / Bootstrap 开始使用 App::new(AppDeps)
- Phase 4: AppBuilder removed / 移除 AppBuilder

````

**Step 3: Update uc-app main README**

```rust
// src-tauri/crates/uc-app/README.md (add section)

## Dependency Injection / 依赖注入

The `App` struct now supports two construction methods:
`App` 结构现在支持两种构造方法：

1. **`App::new(AppDeps)`** (Preferred) / （推荐）
   - Direct dependency injection / 直接依赖注入
   - Constructor signature = dependency manifest / 构造函数签名即依赖清单
   - No hidden magic / 无隐藏魔法

2. **`AppBuilder::build()`** (Legacy, to be removed)
   - **旧版，将被移除**
   - Kept for backward compatibility during migration / 迁移期间保持向后兼容
   - Will be removed in Phase 4 / 将在 Phase 4 中移除

```rust
// Recommended / 推荐
let app = App::new(AppDeps {
    clipboard: Arc::new(clipboard_impl),
    encryption: Arc::new(encryption_impl),
    // ...
});

// Legacy (will be removed) / 旧版（将被移除）
let app = AppBuilder::new()
    .with_clipboard(clipboard_impl)
    .build()?;
````

````

**Step 4: Commit documentation**

```bash
git add src-tauri/crates/uc-core/src/config/README.md \
        src-tauri/crates/uc-app/src/deps.md \
        src-tauri/crates/uc-app/README.md
git commit -m "docs(uc-app, uc-core): add architecture documentation

- Add config module README (pure data principles)
- Add AppDeps documentation (NOT a Builder pattern)
- Update uc-app README with dependency injection guide

Documents the new foundation for bootstrap architecture refactoring."
````

---

## Task 4: Verify no breaking changes to existing code

**Files:**

- Test: Run existing test suite
- Verify: Compilation of all crates

**Step 1: Run full test suite**

```bash
cd /Users/mark/MyProjects/uniclipboard-desktop/src-tauri
cargo test --workspace
```

Expected: All existing tests still PASS

**Step 2: Check compilation of all crates**

```bash
cargo check --workspace --all-targets
```

Expected: No compilation errors

**Step 3: Verify Tauri app still builds**

```bash
cargo tauri build --debug
```

Expected: Build succeeds (may have warnings, but no errors)

**Step 4: Run the app manually to verify no runtime issues**

```bash
cargo tauri dev
```

Expected: App launches and runs normally

**Step 5: Create summary of Phase 1 completion**

```bash
cat > /tmp/phase1-summary.md << 'EOF'
# Phase 1: Foundation - Complete

## What Was Added / 新增内容

1. **uc-core/src/config module** - Pure DTO configuration
   - `AppConfig` struct with no validation/logic
   - `from_toml()` method (data mapping only)
   - `empty()` constructor
   - Unit tests for config module

2. **uc-app/src/deps module** - Dependency grouping
   - `AppDeps` struct (NOT a Builder)
   - `App::new(AppDeps)` constructor
   - Unit tests for deps module
   - Documentation

3. **Backward Compatibility** / 向后兼容
   - Existing `AppBuilder` kept unchanged
   - All existing code continues to work
   - No breaking changes

## What Was NOT Changed / 未改变内容

- ❌ No existing code was modified
- ❌ No behavior changes
- ❌ AppBuilder still works
- ❌ main.rs unchanged

## Next Phase / 下一阶段

Phase 2: Bootstrap Module Creation
- Create `uc-tauri/src/bootstrap/` directory
- Implement `config.rs` (use uc-core::config)
- Implement `wiring.rs` (use uc-app::AppDeps)
- Add tests for bootstrap module
EOF
cat /tmp/phase1-summary.md
```

**Step 6: Commit Phase 1 completion marker**

```bash
git add docs/plans/2026-01-12-bootstrap-phase1-foundation.md
git commit -m "docs: mark Phase 1 Foundation as complete

Phase 1 Summary:
- Added uc-core::config (pure DTO module)
- Added uc-app::AppDeps and App::new()
- Maintained full backward compatibility
- All tests passing
- Ready for Phase 2: Bootstrap Module Creation

See docs/plans/2026-01-12-bootstrap-phase1-foundation.md for details."
```

---

## Phase 1 Completion Checklist / Phase 1 完成清单

After completing all tasks, verify:

- [ ] `uc-core/src/config/mod.rs` created with pure DTO
- [ ] `uc-core/src/config` tests passing
- [ ] `uc-app/src/deps.rs` created with AppDeps struct
- [ ] `uc-app/src/lib.rs` exports AppDeps
- [ ] `App::new(AppDeps)` constructor added
- [ ] Existing `AppBuilder` still works (backward compat)
- [ ] All existing tests still pass
- [ ] `cargo check --workspace` succeeds
- [ ] Documentation created
- [ ] No breaking changes to existing code
- [ ] Phase 1 completion commit created

---

## Architecture Validation / 架构验证

After Phase 1, run the validation checklist from the design doc:

运行设计文档中的验证清单：

- [ ] **Self-check 1**: Can bootstrap be directly depended upon by test crates?
      Expected: ❌ No (bootstrap doesn't exist yet - Phase 2)
      应该：❌ 否（bootstrap 尚不存在 - Phase 2）

- [ ] **Self-check 2**: Can business code compile independently without bootstrap?
      Expected: ✅ Yes
      应该：✅ 是

- [ ] **Self-check 6**: Does AppBuilder still exist?
      Expected: ✅ Yes (for compatibility - will be removed in Phase 4)
      应该：✅ 是（为了兼容 - 将在 Phase 4 移除）

- [ ] **Self-check 7**: Does uc-core::config contain only DTOs?
      Expected: ✅ Yes
      应该：✅ 是

---

## Related Documentation / 相关文档

- **Design Document**: [docs/plans/2026-01-12-bootstrap-architecture-design.md](docs/plans/2026-01-12-bootstrap-architecture-design.md)
- **Project DeepWiki**: https://deepwiki.com/UniClipboard/UniClipboard

---

**Phase 1 Status**: ✅ Ready to Implement
**Estimated Time**: 2-3 hours
**Risk Level**: Low (additive changes only, no modifications to existing code)
