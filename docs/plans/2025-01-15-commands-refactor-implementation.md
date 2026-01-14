# Commands Layer Refactor Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Complete the Commands Layer migration by fixing plan documentation, removing duplicate doc comments, and standardizing all commands to use the UseCases accessor pattern.

**Architecture:** Follow existing Hexagonal Architecture patterns - use cases depend on Ports (traits), infrastructure implements ports, commands access use cases through UseCases accessor.

**Tech Stack:** Rust, async/await with tokio, Arc<dyn Trait> ports, Tauri 2.

---

## Context for Implementation

### Current State

1. **GetSettings Use Case** (`uc-app/usecases/get_settings.rs`) - Already exists with correct import: `use uc_core::ports::settings::SettingsPort;`
2. **SettingsPort** (`uc-core/ports/settings.rs`) - Already defined with `load()` and `save()` methods
3. **FileSettingsRepository** (`uc-infra/settings/repository.rs`) - Already implements SettingsPort
4. **Commands** (`uc-tauri/commands/`) - Mixed patterns:
   - `delete_clipboard_entry` - Uses `runtime.usecases()` pattern (CORRECT)
   - `initialize_encryption` - Uses `runtime.usecases()` pattern (CORRECT)
   - `is_encryption_initialized` - Uses `runtime.usecases()` pattern (CORRECT)
   - `get_clipboard_entries` - Uses direct `runtime.deps` access (NEEDS FIX)
   - `capture_clipboard` - Has TODO, returns placeholder error (NEEDS FIX)
   - `get_settings` - Uses deprecated `State<'_, AppDeps>` (NEEDS FIX)
   - `update_settings` - Uses deprecated `State<'_, AppDeps>`, no use case exists (NEEDS FIX)
5. **Plan Documentation** (`docs/plans/2025-01-14-settings-usecases-implementation.md`) - Has incorrect import path in example code (NEEDS FIX)
6. **main.rs** - Has duplicate doc comment for `run_app` (NEEDS FIX)
7. **main.rs** - Has inline macOS platform-specific commands in `generate_invoke_handler!` (NEEDS FIX)

### Issues to Fix

1. **Plan documentation** - Line 67-68: Incorrect import `use uc_core::ports::SettingsPort;` should be `use uc_core::ports::settings::SettingsPort;`
2. **main.rs** - Lines 116-118: Duplicate doc comment `/// Run the Tauri application`
3. **Commands** - Need to standardize all commands to use `runtime.usecases()` pattern
4. **main.rs** - Lines 89-114: Need to extract macOS-specific commands to conditional module

---

## Task 1: Fix Plan Documentation Import Path

**Files:**
- Modify: `docs/plans/2025-01-14-settings-usecases-implementation.md`

**Step 1: Update import path in Task 1 code example**

Modify: `docs/plans/2025-01-14-settings-usecases-implementation.md:67`

Change:
```rust
use anyhow::Result;
use uc_core::ports::SettingsPort;
use uc_core::settings::model::Settings;
```

To:
```rust
use anyhow::Result;
use uc_core::ports::settings::SettingsPort;
use uc_core::settings::model::Settings;
```

**Step 2: Verify markdown syntax**

Check that the markdown is properly formatted with no syntax errors.

**Step 3: Commit**

```bash
git add docs/plans/2025-01-14-settings-usecases-implementation.md
git commit -m "docs: fix SettingsPort import path in plan

Update import from uc_core::ports::SettingsPort to
uc_core::ports::settings::SettingsPort to match actual
module structure used in get_settings.rs implementation.
"
```

---

## Task 2: Remove Duplicate Doc Comment in main.rs

**Files:**
- Modify: `src-tauri/src/main.rs`

**Step 1: Remove duplicate doc comment**

Modify: `src-tauri/src/main.rs:116-118`

Remove line 117 (the duplicate `/// Run the Tauri application` comment).

Before:
```rust
/// Run the Tauri application

/// Run the Tauri application
fn run_app(config: AppConfig) {
```

After:
```rust
/// Run the Tauri application
fn run_app(config: AppConfig) {
```

**Step 2: Verify compilation**

Run: `cd src-tauri && cargo check`

Expected: No errors

**Step 3: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "fix: remove duplicate doc comment in main.rs

Remove duplicate 'Run the Tauri application' doc comment
above run_app function definition.
"
```

---

## Task 3: Refactor get_clipboard_entries to Use UseCases Accessor

**Files:**
- Modify: `src-tauri/crates/uc-tauri/src/commands/clipboard.rs`

**Step 1: Update get_clipboard_entries to use runtime.usecases()**

Modify: `src-tauri/crates/uc-tauri/src/commands/clipboard.rs:12-39`

Replace the entire function with:

```rust
/// Get clipboard history entries
/// 获取剪贴板历史条目
#[tauri::command]
pub async fn get_clipboard_entries(
    runtime: State<'_, AppRuntime>,
    limit: Option<usize>,
) -> Result<Vec<ClipboardEntryProjection>, String> {
    // Use UseCases accessor pattern (consistent with other commands)
    let uc = runtime.usecases().list_clipboard_entries();
    let limit = limit.unwrap_or(50);

    // Query entries through use case
    let entries = uc.execute(limit, 0)
        .await
        .map_err(|e| e.to_string())?;

    // Convert domain models to DTOs
    let projections: Vec<ClipboardEntryProjection> = entries
        .into_iter()
        .map(|entry| ClipboardEntryProjection {
            id: entry.entry_id.to_string(),
            preview: entry.title.unwrap_or_else(|| format!("Entry ({} bytes)", entry.total_size)),
            captured_at: entry.created_at_ms,
            content_type: "clipboard".to_string(),
            is_encrypted: false, // TODO: Determine from actual entry state
        })
        .collect();

    Ok(projections)
}
```

**Step 2: Remove unused import**

The line `use crate::bootstrap::AppRuntime;` is already imported at the top, and `use uc_app::usecases::ListClipboardEntries;` is no longer directly needed.

**Step 3: Verify compilation**

Run: `cd src-tauri && cargo check --package uc-tauri`

Expected: No errors

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/clipboard.rs
git commit -m "refactor(uc-tauri): use UseCases accessor for get_clipboard_entries

Change from direct deps access to runtime.usecases() pattern
for consistency with other commands (delete_clipboard_entry,
initialize_encryption, is_encryption_initialized).
"
```

---

## Task 4: Add GetSettings and UpdateSettings to UseCases Accessor

**Files:**
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`

**Step 1: Add get_settings method**

Modify: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`

Add after line 217 (after `is_encryption_initialized()` method):

```rust
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
```

**Step 2: Verify compilation**

Run: `cd src-tauri && cargo check --package uc-tauri`

Expected: No errors (GetSettings use case already exists)

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs
git commit -m "feat(uc-tauri): add get_settings to UseCases accessor

Adds convenience method for accessing GetSettings use case
with pre-wired SettingsPort dependency.
"
```

---

## Task 5: Create UpdateSettings Use Case

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

Add after line 20:

```rust
pub mod update_settings;
```

Add in the pub use section (after line 29):

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
Validates schema version before persisting through SettingsPort.
"
```

---

## Task 6: Add update_settings to UseCases Accessor

**Files:**
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`

**Step 1: Add update_settings method**

Modify: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`

Add after the `get_settings()` method:

```rust
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
```

**Step 2: Verify compilation**

Run: `cd src-tauri && cargo check --package uc-tauri`

Expected: No errors

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs
git commit -m "feat(uc-tauri): add update_settings to UseCases accessor

Adds convenience method for accessing UpdateSettings use case
with pre-wired SettingsPort dependency.
"
```

---

## Task 7: Update get_settings Command

**Files:**
- Modify: `src-tauri/crates/uc-tauri/src/commands/settings.rs`

**Step 1: Update get_settings to use UseCases accessor**

Modify: `src-tauri/crates/uc-tauri/src/commands/settings.rs:37-49`

Replace the entire function with:

```rust
/// Get application settings
/// 获取应用设置
#[tauri::command]
pub async fn get_settings(
    runtime: State<'_, AppRuntime>,
) -> Result<Value, String> {
    let uc = runtime.usecases().get_settings();
    let settings = uc.execute().await.map_err(|e| e.to_string())?;

    // Convert Settings to JSON value
    serde_json::to_value(&settings).map_err(|e| format!("Failed to serialize settings: {}", e))
}
```

**Step 2: Update imports**

Modify: `src-tauri/crates/uc-tauri/src/commands/settings.rs:1-6`

Replace:
```rust
//! Settings-related Tauri commands
//! 设置相关的 Tauri 命令

use serde_json::Value;
use tauri::State;
use uc_app::AppDeps;
```

With:
```rust
//! Settings-related Tauri commands
//! 设置相关的 Tauri 命令

use serde_json::Value;
use tauri::State;
use crate::bootstrap::AppRuntime;
```

**Step 3: Verify compilation**

Run: `cd src-tauri && cargo check --package uc-tauri`

Expected: No errors

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/settings.rs
git commit -m "feat(uc-tauri): implement get_settings command

Command now uses UseCases accessor pattern:
- Change from State<'_, AppDeps> to State<'_, AppRuntime>
- Use runtime.usecases().get_settings()
- Return Settings as JSON

Removes TODO placeholder, completes get_settings implementation.
"
```

---

## Task 8: Update update_settings Command

**Files:**
- Modify: `src-tauri/crates/uc-tauri/src/commands/settings.rs`

**Step 1: Update update_settings to use UseCases accessor**

Modify: `src-tauri/crates/uc-tauri/src/commands/settings.rs:51-94`

Replace the entire function with:

```rust
/// Update application settings
/// 更新应用设置
#[tauri::command]
pub async fn update_settings(
    runtime: State<'_, AppRuntime>,
    settings: Value,
) -> Result<(), String> {
    use uc_core::settings::model::Settings;

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
git commit -m "feat(uc-tauri): implement update_settings command

Command now uses UseCases accessor pattern:
- Change from State<'_, AppDeps> to State<'_, AppRuntime>
- Use runtime.usecases().update_settings()
- Parse JSON to Settings domain model
- Validate schema version in use case

Removes TODO placeholder, completes update_settings implementation.
"
```

---

## Task 9: Create macOS Platform Commands Module

**Files:**
- Create: `src-tauri/src/plugins/mac_rounded_corners.rs`
- Create: `src-tauri/src/plugins/mod.rs`
- Modify: `src-tauri/src/main.rs`

**Step 1: Create platform commands module**

Create: `src-tauri/src/plugins/mod.rs`

```rust
//! Platform-specific Tauri command modules
//! 平台特定的 Tauri 命令模块

#[cfg(target_os = "macos")]
pub mod mac_rounded_corners;

// Re-export macOS commands for invoke_handler macro
#[cfg(target_os = "macos")]
pub use mac_rounded_corners::{
    enable_rounded_corners,
    enable_modern_window_style,
    reposition_traffic_lights,
};
```

**Step 2: Update main.rs imports**

Modify: `src-tauri/src/main.rs:22-26`

Replace:
```rust
// Plugins
mod plugins;

#[cfg(target_os = "macos")]
use plugins::mac_rounded_corners;
```

With:
```rust
// Platform-specific command modules
mod plugins;

#[cfg(target_os = "macos")]
use plugins::{enable_modern_window_style, enable_rounded_corners, reposition_traffic_lights};
```

**Step 3: Update generate_invoke_handler! macro**

Modify: `src-tauri/src/main.rs:89-114`

Replace the entire macro with:

```rust
/// Macro to generate invoke handler with platform-specific commands
macro_rules! generate_invoke_handler {
    () => {
        tauri::generate_handler![
            // Clipboard commands
            uc_tauri::commands::clipboard::get_clipboard_entries,
            uc_tauri::commands::clipboard::delete_clipboard_entry,
            uc_tauri::commands::clipboard::capture_clipboard,
            // Encryption commands
            uc_tauri::commands::encryption::initialize_encryption,
            uc_tauri::commands::encryption::is_encryption_initialized,
            // Settings commands
            uc_tauri::commands::settings::get_settings,
            uc_tauri::commands::settings::update_settings,
            // Onboarding commands
            check_onboarding_status,
            // macOS-specific commands (conditionally compiled)
            #[cfg(target_os = "macos")]
            enable_rounded_corners,
            #[cfg(target_os = "macos")]
            enable_modern_window_style,
            #[cfg(target_os = "macos")]
            reposition_traffic_lights,
        ]
    };
}
```

**Step 4: Verify compilation**

Run: `cd src-tauri && cargo check`

Expected: No errors

**Step 5: Commit**

```bash
git add src-tauri/src/plugins/mod.rs src-tauri/src/main.rs
git commit -m "refactor: extract macOS platform commands to plugins module

- Create plugins/mod.rs to re-export macOS-specific commands
- Update main.rs to use plugins module instead of direct path
- Clean up generate_invoke_handler! macro with consistent formatting

This isolates platform-specific commands and makes main.rs cleaner.
"
```

---

## Task 10: Update Documentation

**Files:**
- Modify: `docs/architecture/commands-status.md`

**Step 1: Update commands status**

Modify: `docs/architecture/commands-status.md`

Update the Command Status Matrix table:

Change:
```markdown
| `get_clipboard_entries` | [clipboard.rs:12-40](../../src-tauri/crates/uc-tauri/src/commands/clipboard.rs#L12-L40) | ❌ | ✅ | Direct deps access |
| `capture_clipboard` | [clipboard.rs:119-137](../../src-tauri/crates/uc-tauri/src/commands/clipboard.rs#L119-L137) | ❌ | ❌ | TODO placeholder |
| `get_settings` | [settings.rs:37-49](../../src-tauri/crates/uc-tauri/src/commands/settings.rs#L37-L49) | ✅ | ❌ | Placeholder |
| `update_settings` | [settings.rs:82-94](../../src-tauri/crates/uc-tauri/src/commands/settings.rs#L82-L94) | ✅ | ❌ | Placeholder |
```

To:
```markdown
| `get_clipboard_entries` | [clipboard.rs:12-39](../../src-tauri/crates/uc-tauri/src/commands/clipboard.rs#L12-L39) | ✅ | ✅ | Complete |
| `capture_clipboard` | [clipboard.rs:76-60](../../src-tauri/crates/uc-tauri/src/commands/clipboard.rs#L76-L60) | ❌ | ❌ | Complex multi-port use case |
| `get_settings` | [settings.rs:10-21](../../src-tauri/crates/uc-tauri/src/commands/settings.rs#L10-L21) | ✅ | ✅ | Complete |
| `update_settings` | [settings.rs:23-38](../../src-tauri/crates/uc-tauri/src/commands/settings.rs#L23-L38) | ✅ | ✅ | Complete |
```

Update the Use Case Status table:

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

Update the Migration Progress section:

Change:
```markdown
**Core Commands: 5/7 using UseCases accessor (71%)**
```

To:
```markdown
**Core Commands: 6/7 using UseCases accessor (86%)**

**Note:** `capture_clipboard` requires complex multi-port orchestration and is tracked separately.
```

Update the Next Steps section, remove:

```markdown
5. ⏳ Implement `GetSettings` and `UpdateSettings` use cases
```

**Step 2: Verify markdown formatting**

Check that markdown files are valid (no syntax errors).

**Step 3: Commit**

```bash
git add docs/architecture/commands-status.md
git commit -m "docs: update commands status to 86% complete

All settings commands now use UseCases accessor pattern.
get_clipboard_entries refactored to use runtime.usecases().

Remaining work: capture_clipboard (complex multi-port use case).
"
```

---

## Task 11: Integration Testing

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

Full coverage of settings use case behavior.
"
```

---

## Summary

This plan completes the Commands Layer refactoring:

1. ✅ Fix plan documentation import path
2. ✅ Remove duplicate doc comment in main.rs
3. ✅ Refactor get_clipboard_entries to UseCases accessor
4. ✅ Add GetSettings to UseCases accessor
5. ✅ Create UpdateSettings use case
6. ✅ Add UpdateSettings to UseCases accessor
7. ✅ Implement get_settings command
8. ✅ Implement update_settings command
9. ✅ Extract macOS platform commands to plugins module
10. ✅ Update documentation
11. ✅ Add integration tests

**Total estimated time:** 45-60 minutes for full implementation

**Testing strategy:**

- Unit tests are implicit in use case simplicity
- Integration tests cover the full flow
- Manual testing: Run app, call commands from frontend

**Next steps after this plan:**

- Implement `CaptureClipboard` use case (complex multi-port orchestration)
- Consider adding settings change events
- Add frontend integration if needed
