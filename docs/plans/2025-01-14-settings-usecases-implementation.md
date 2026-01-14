# Settings Use Cases Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement `GetSettings` and `UpdateSettings` use cases to complete the Commands Layer migration (currently at 71%, targeting 100%).

**Architecture:** Follow existing Hexagonal Architecture patterns - use cases depend on Ports (traits), infrastructure implements ports, commands access use cases through UseCases accessor.

**Tech Stack:** Rust, async/await with tokio, Arc<dyn Trait> ports, serde for JSON serialization.

---

## Context for Implementation

### Current State

1. **SettingsPort** (`uc-core/ports/settings.rs`) - Already defined with `load()` and `save()` methods
2. **Settings domain model** (`uc-core/settings/model.rs`) - Complete with all settings structures
3. **FileSettingsRepository** (`uc-infra/settings/repository.rs`) - Already implements SettingsPort with file-based persistence and migration
4. **AppDeps** (`uc-app/deps.rs:55`) - Already has `pub settings: Arc<dyn SettingsPort>`
5. **Commands** (`uc-tauri/commands/settings.rs`) - Return placeholder errors, need updating

### Migration Pattern (Reference: IsEncryptionInitialized)

```text
Use Case (uc-app/usecases/)
    ↓ depends on
Port (uc-core/ports/)
    ↓ implemented by
Repository (uc-infra/settings/)
    ↓ wired in
AppDeps (uc-app/deps.rs)
    ↓ accessed via
UseCases accessor (uc-tauri/bootstrap/runtime.rs)
    ↓ called by
Command (uc-tauri/commands/)
```

### Existing Command Contract

```rust
// Current signature (uses deprecated AppDeps)
pub async fn get_settings(_deps: State<'_, AppDeps>) -> Result<Value, String>

// Target signature (uses AppRuntime)
pub async fn get_settings(runtime: State<'_, AppRuntime>) -> Result<Value, String>
```

---

## Task 1: Create GetSettings Use Case

**Files:**

- Create: `src-tauri/crates/uc-app/src/usecases/get_settings.rs`
- Modify: `src-tauri/crates/uc-app/src/usecases/mod.rs`

**Step 1: Write the use case implementation**

Create: `src-tauri/crates/uc-app/src/usecases/get_settings.rs`

```rust
//! Use case for getting application settings
//! 获取应用设置的用例

use anyhow::Result;
use uc_core::ports::settings::SettingsPort;
use uc_core::settings::model::Settings;

/// Use case for retrieving application settings.
///
/// ## Behavior / 行为
/// - Loads settings from the settings port
/// - Returns the complete settings structure
///
/// ## English
/// Loads the current application settings from the configured
/// settings repository and returns them to the caller.
pub struct GetSettings {
    settings: std::sync::Arc<dyn SettingsPort>,
}

impl GetSettings {
    /// Create a new GetSettings use case.
    pub fn new(settings: std::sync::Arc<dyn SettingsPort>) -> Self {
        Self { settings }
    }

    /// Execute the use case.
    ///
    /// # Returns / 返回值
    /// - `Ok(Settings)` - The current application settings
    /// - `Err(e)` if loading settings fails
    pub async fn execute(&self) -> Result<Settings> {
        self.settings.load().await
    }
}
```

**Step 2: Export the use case from mod.rs**

Modify: `src-tauri/crates/uc-app/src/usecases/mod.rs`

Add after line 29:

```rust
pub mod get_settings;
```

Add in the pub use section (after line 30):

```rust
pub use get_settings::GetSettings;
```

**Step 3: Verify compilation**

Run: `cd src-tauri && cargo check --package uc-app`

Expected: No errors

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/get_settings.rs src-tauri/crates/uc-app/src/usecases/mod.rs
git commit -m "feat(uc-app): add GetSettings use case

Implements use case for retrieving application settings.
Follows existing pattern from IsEncryptionInitialized.
"
```

---

## Task 2: Create UpdateSettings Use Case

**Files:**

- Create: `src-tauri/crates/uc-app/src/usecases/update_settings.rs`
- Modify: `src-tauri/crates/uc-app/src/usecases/mod.rs`

**Step 1: Write the use case implementation**

Create: `src-tauri/crates/uc-app/src/usecases/update_settings.rs`

```rust
//! Use case for updating application settings
//! 更新应用设置的用例

use anyhow::Result;
use uc_core::ports::settings::SettingsPort;
use uc_core::settings::model::Settings;

/// Use case for updating application settings.
///
/// ## Behavior / 行为
/// - Validates settings (basic validation)
/// - Persists settings through the settings port
///
/// ## English
/// Updates the application settings by validating and persisting
/// the provided settings through the configured settings repository.
pub struct UpdateSettings {
    settings: std::sync::Arc<dyn SettingsPort>,
}

impl UpdateSettings {
    /// Create a new UpdateSettings use case.
    pub fn new(settings: std::sync::Arc<dyn SettingsPort>) -> Self {
        Self { settings }
    }

    /// Execute the use case.
    ///
    /// # Parameters / 参数
    /// - `settings`: The settings to persist
    ///
    /// # Returns / 返回值
    /// - `Ok(())` if settings are saved successfully
    /// - `Err(e)` if validation or save fails
    pub async fn execute(&self, settings: Settings) -> Result<()> {
        // Basic validation: ensure schema version is current
        let current_version = uc_core::settings::model::CURRENT_SCHEMA_VERSION;
        if settings.schema_version != current_version {
            return Err(anyhow::anyhow!(
                "Invalid schema version: expected {}, got {}",
                current_version,
                settings.schema_version
            ));
        }

        // Persist settings
        self.settings.save(&settings).await
    }
}
```

**Step 2: Export the use case from mod.rs**

Modify: `src-tauri/crates/uc-app/src/usecases/mod.rs`

Add after the get_settings import:

```rust
pub mod update_settings;
```

Add in the pub use section:

```rust
pub use update_settings::UpdateSettings;
```

**Step 3: Verify compilation**

Run: `cd src-tauri && cargo check --package uc-app`

Expected: No errors

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/update_settings.rs src-tauri/crates/uc-app/src/usecases/mod.rs
git commit -m "feat(uc-app): add UpdateSettings use case

Implements use case for updating application settings.
Validates schema version before persisting.
"
```

---

## Task 3: Add Use Cases to UseCases Accessor

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`

**Step 1: Add get_settings method**

Modify: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`

Add method after line 217 (after `is_encryption_initialized()`):

````rust
    /// Get application settings
    ///
    /// ## Example / 示例
    ///
    /// ```rust,no_run
    /// # use uc_tauri::bootstrap::AppRuntime;
    /// # use tauri::State;
    /// # async fn example(runtime: State<'_, AppRuntime>) -> Result<uc_core::settings::model::Settings, String> {
    /// let uc = runtime.usecases().get_settings();
    /// let settings = uc.execute().await.map_err(|e| e.to_string())?;
    /// # Ok(settings)
    /// # }
    /// ```
    pub fn get_settings(&self) -> uc_app::usecases::GetSettings {
        uc_app::usecases::GetSettings::new(self.runtime.deps.settings.clone())
    }
````

**Step 2: Add update_settings method**

Add after the `get_settings()` method:

````rust
    /// Update application settings
    ///
    /// ## Example / 示例
    ///
    /// ```rust,no_run
    /// # use uc_tauri::bootstrap::AppRuntime;
    /// # use tauri::State;
    /// # use uc_core::settings::model::Settings;
    /// # async fn example(runtime: State<'_, AppRuntime>, settings: Settings) -> Result<(), String> {
    /// let uc = runtime.usecases().update_settings();
    /// uc.execute(settings).await.map_err(|e| e.to_string())?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn update_settings(&self) -> uc_app::usecases::UpdateSettings {
        uc_app::usecases::UpdateSettings::new(self.runtime.deps.settings.clone())
    }
````

**Step 3: Verify compilation**

Run: `cd src-tauri && cargo check --package uc-tauri`

Expected: No errors

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs
git commit -m "feat(uc-tauri): add settings use cases to UseCases accessor

Adds get_settings() and update_settings() methods for convenient
use case access with pre-wired dependencies.
"
```

---

## Task 4: Update get_settings Command

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/settings.rs`

**Step 1: Update imports and command signature**

Modify: `src-tauri/crates/uc-tauri/src/commands/settings.rs`

Replace the entire file content with:

```rust
//! Settings-related Tauri commands
//! 设置相关的 Tauri 命令

use serde_json::Value;
use tauri::State;
use uc_core::settings::model::Settings;
use uc_tauri::bootstrap::AppRuntime;

/// Get application settings
/// 获取应用设置
///
/// Returns the complete application settings as JSON.
///
/// ## Returns / 返回值
/// - JSON representation of current Settings
#[tauri::command]
pub async fn get_settings(
    runtime: State<'_, AppRuntime>,
) -> Result<Value, String> {
    let uc = runtime.usecases().get_settings();
    let settings = uc.execute().await.map_err(|e| e.to_string())?;

    // Convert Settings to JSON value
    serde_json::to_value(&settings).map_err(|e| format!("Failed to serialize settings: {}", e))
}

/// Update application settings
/// 更新应用设置
///
/// Updates application settings from JSON.
///
/// ## Parameters / 参数
/// - `settings`: JSON value containing settings to update
#[tauri::command]
pub async fn update_settings(
    runtime: State<'_, AppRuntime>,
    settings: Value,
) -> Result<(), String> {
    // Parse JSON into Settings domain model
    let settings: Settings = serde_json::from_value(settings)
        .map_err(|e| format!("Failed to parse settings: {}", e))?;

    let uc = runtime.usecases().update_settings();
    uc.execute(settings).await.map_err(|e| e.to_string())
}
```

**Step 2: Verify compilation**

Run: `cd src-tauri && cargo check --package uc-tauri`

Expected: No errors

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/settings.rs
git commit -m "feat(uc-tauri): implement get_settings and update_settings commands

Commands now use UseCases accessor pattern:
- get_settings: Returns Settings as JSON
- update_settings: Parses JSON and updates Settings

Completes Commands Layer migration for settings (100%).
"
```

---

## Task 5: Update Documentation

**Files:**

- Modify: `docs/architecture/commands-status.md`
- Modify: `CLAUDE.md` (optional, if you want to add quick reference)

**Step 1: Update commands status**

Modify: `docs/architecture/commands-status.md`

Update the Command Status Matrix table (around line 27-28):

Change:

```markdown
| `get_settings` | [settings.rs:37-49](../../src-tauri/crates/uc-tauri/src/commands/settings.rs#L37-L49) | ✅ | ❌ | Placeholder |
| `update_settings` | [settings.rs:81-94](../../src-tauri/crates/uc-tauri/src/commands/settings.rs#L81-L94) | ✅ | ❌ | Placeholder |
```

To:

```markdown
| `get_settings` | [settings.rs:18-33](../../src-tauri/crates/uc-tauri/src/commands/settings.rs#L18-L33) | ✅ | ✅ | Complete |
| `update_settings` | [settings.rs:45-60](../../src-tauri/crates/uc-tauri/src/commands/settings.rs#L45-L60) | ✅ | ✅ | Complete |
```

Update the Use Case Status table (around line 55-56):

Change:

```markdown
| `GetSettings` | ❌ | - | `get_settings` (TODO) |
| `UpdateSettings` | ❌ | - | `update_settings` (TODO) |
```

To:

```markdown
| `GetSettings` | ✅ | `uc-app/src/usecases/get_settings.rs` | `get_settings` |
| `UpdateSettings` | ✅ | `uc-app/src/usecases/update_settings.rs` | `update_settings` |
```

Update the Migration Progress section (around line 60):

Change:

```markdown
**Core Commands: 5/7 using UseCases accessor (71%)**
```

To:

```markdown
**Core Commands: 7/7 using UseCases accessor (100%)**
```

Update the Next Steps section (around line 86-88), remove:

```markdown
5. ⏳ Implement `GetSettings` and `UpdateSettings` use cases
```

**Step 2: Verify markdown formatting**

Run: Check that markdown files are valid (no syntax errors)

**Step 3: Commit**

```bash
git add docs/architecture/commands-status.md
git commit -m "docs: update commands status to 100% complete

All core commands now use UseCases accessor pattern.
Settings use cases implementation complete.
"
```

---

## Task 6: Integration Testing

**Files:**

- Create: `src-tauri/crates/uc-tauri/tests/integration_settings.rs`

**Step 1: Create integration test**

Create: `src-tauri/crates/uc-tauri/tests/integration_settings.rs`

```rust
//! Integration tests for settings use cases
//!
//! Tests the complete flow from command to persistence

use std::sync::Arc;
use tempfile::tempdir;
use uc_app::usecases::{GetSettings, UpdateSettings};
use uc_core::settings::model::{Settings, CURRENT_SCHEMA_VERSION};
use uc_infra::settings::repository::FileSettingsRepository;

#[tokio::test]
async fn test_get_settings_returns_defaults() {
    let temp_dir = tempdir().unwrap();
    let settings_path = temp_dir.path().join("test_settings.json");

    let repo = FileSettingsRepository::new(settings_path);
    let repo_arc: Arc<dyn uc_core::ports::settings::SettingsPort> = Arc::new(repo);

    let uc = GetSettings::new(repo_arc.clone());
    let settings = uc.execute().await.unwrap();

    // Should return defaults since file doesn't exist
    assert_eq!(settings.schema_version, CURRENT_SCHEMA_VERSION);
}

#[tokio::test]
async fn test_update_settings_persists() {
    let temp_dir = tempdir().unwrap();
    let settings_path = temp_dir.path().join("test_settings.json");

    let repo = FileSettingsRepository::new(settings_path.clone());
    let repo_arc: Arc<dyn uc_core::ports::settings::SettingsPort> = Arc::new(repo);

    // Update settings
    let mut settings = Settings::default();
    settings.general.device_name = Some("test_device".to_string());

    let update_uc = UpdateSettings::new(repo_arc.clone());
    update_uc.execute(settings.clone()).await.unwrap();

    // Verify persistence through GetSettings
    let get_uc = GetSettings::new(repo_arc);
    let loaded = get_uc.execute().await.unwrap();

    assert_eq!(loaded.general.device_name, Some("test_device".to_string()));
}

#[tokio::test]
async fn test_update_settings_validates_schema_version() {
    let temp_dir = tempdir().unwrap();
    let settings_path = temp_dir.path().join("test_settings.json");

    let repo = FileSettingsRepository::new(settings_path);
    let repo_arc: Arc<dyn uc_core::ports::settings::SettingsPort> = Arc::new(repo);

    let mut settings = Settings::default();
    settings.schema_version = 999; // Invalid version

    let update_uc = UpdateSettings::new(repo_arc);
    let result = update_uc.execute(settings).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid schema version"));
}
```

**Step 2: Run integration tests**

Run: `cd src-tauri && cargo test --package uc-tauri --test integration_settings`

Expected: All tests pass

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-tauri/tests/integration_settings.rs
git commit -m "test(uc-tauri): add integration tests for settings use cases

Tests:
- GetSettings returns defaults for missing file
- UpdateSettings persists and can be loaded
- UpdateSettings validates schema version
"
```

---

## Summary

This plan implements the complete settings use cases migration:

1. ✅ GetSettings use case - Loads settings from port
2. ✅ UpdateSettings use case - Validates and persists settings
3. ✅ UseCases accessor - Adds convenience methods
4. ✅ Commands - Updated to use AppRuntime + UseCases accessor
5. ✅ Documentation - Commands Layer now 100% complete
6. ✅ Integration tests - Full coverage of use case behavior

**Total estimated time:** 30-45 minutes for full implementation

**Testing strategy:**

- Unit tests are implicit in use case simplicity
- Integration tests cover the full flow
- Manual testing: Run app, call commands from frontend

**Next steps after this plan:**

- Consider adding settings change events (emitted when UpdateSettings succeeds)
- Add frontend integration if needed
- Review other placeholder commands for similar migration
