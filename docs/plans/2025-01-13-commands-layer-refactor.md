# Commands Layer Refactor Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Refactor Commands layer to follow hexagonal architecture by enforcing UseCases accessor pattern and eliminating direct Port access.

**Architecture:** Commands layer is a Driving Adapter that MUST call Use Cases through the UseCases accessor, never directly accessing Ports. All business logic remains in Use Cases layer.

**Tech Stack:** Rust, Tauri 2, Hexagonal Architecture

---

## Context

**Problem:** Commands in `src-tauri/crates/uc-tauri/src/commands/` currently directly access Ports (`runtime.deps.xxx`), violating hexagonal architecture.

**Solution:** Enforce UseCases accessor pattern where all commands go through `runtime.usecases().xxx()` to get use case instances.

**Files to Modify:**

- `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs` - Add UseCases accessor methods
- `src-tauri/crates/uc-tauri/src/commands/encryption.rs` - Refactor to use UseCases
- `src-tauri/crates/uc-app/src/usecases/mod.rs` - Export use case type alias

**Scope:** ONLY refactor commands that have existing use cases:

- ✅ `initialize_encryption` - Use case exists, command needs refactor
- ⏳ Others marked TODO (separate task for missing use cases)

---

## Task 1: Add Type Alias for InitializeEncryption Use Case

**Files:**

- Modify: `src-tauri/crates/uc-app/src/usecases/mod.rs`

**Step 1: Add type alias to mod.rs**

Add this at the end of the file:

```rust
//! Business logic use cases
//! 是否是独立 Use Case，
//! 取决于"是否需要用户 / 系统再次做出决策"
//!
//! [ClipboardWatcher]
//        ↓
// CaptureClipboardUseCase
//         ↓
// ---------------------------------
//         ↓
// ListClipboardEntryPreviewsUseCase  → UI 列表
// GetClipboardEntryPreviewUseCase    → UI hover / detail
// ---------------------------------
//         ↓
// MaterializeClipboardSelectionUseCase → 粘贴 / 恢复 / 同步

pub mod change_passphrase;
pub mod clipboard;
pub mod initialize_encryption;
pub mod internal;
pub mod list_clipboard_entries;
pub mod list_clipboard_entry_previews;
pub mod settings;

pub use list_clipboard_entries::ListClipboardEntries;
pub use initialize_encryption::InitializeEncryption;

// Type alias for UseCases accessor
pub type InitializeEncryptionUseCase = InitializeEncryption<
    std::sync::Arc<dyn uc_core::ports::EncryptionPort>,
    std::sync::Arc<dyn uc_core::ports::KeyMaterialPort>,
    std::sync::Arc<dyn uc_core::ports::security::key_scope::KeyScopePort>,
    std::sync::Arc<dyn uc_core::ports::security::encryption_state::EncryptionStatePort>,
>;
```

**Step 2: Verify compilation**

Run: `cargo check -p uc-app`
Expected: SUCCESS, no errors

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/mod.rs
git commit -m "feat(uc-app): add InitializeEncryptionUseCase type alias for UseCases accessor"
```

---

## Task 2: Add InitializeEncryption to UseCases Accessor

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`

**Step 1: Read current runtime.rs to find UseCases impl block**

The file is at: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`

Find the `impl<'a> UseCases<'a>` block around line 124.

**Step 2: Add initialize_encryption method to UseCases accessor**

Add this method inside the `impl<'a> UseCases<'a>` block:

````rust
    /// Security use cases / 安全用例
    ///
    /// Get the InitializeEncryption use case for setting up encryption.
    ///
    /// 获取 InitializeEncryption 用例以设置加密。
    ///
    /// ## Example / 示例
    ///
    /// ```rust,no_run
    /// # use uc_tauri::bootstrap::AppRuntime;
    /// # use tauri::State;
    /// # async fn example(runtime: State<'_, AppRuntime>) -> Result<(), String> {
    /// let uc = runtime.usecases().initialize_encryption();
    /// uc.execute(uc_core::security::model::Passphrase("my_pass".to_string()))
    ///     .await
    ///     .map_err(|e| e.to_string())?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn initialize_encryption(&self) -> uc_app::usecases::InitializeEncryptionUseCase {
        uc_app::usecases::InitializeEncryptionUseCase::new(
            self.runtime.deps.encryption.clone(),
            self.runtime.deps.key_material.clone(),
            self.runtime.deps.key_scope.clone(),
            self.runtime.deps.encryption_state.clone(),
        )
    }
````

Place it after the `list_clipboard_entries` method (around line 150).

**Step 3: Verify compilation**

Run: `cargo check -p uc-tauri`
Expected: SUCCESS, no errors

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs
git commit -m "feat(uc-tauri): add initialize_encryption to UseCases accessor"
```

---

## Task 3: Refactor initialize_encryption Command

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/encryption.rs`

**Step 1: Read current encryption.rs command**

The file is at: `src-tauri/crates/uc-tauri/src/commands/encryption.rs`

Current implementation (lines 19-83) directly accesses Ports:

```rust
#[tauri::command]
pub async fn initialize_encryption(
    runtime: State<'_, AppRuntime>,
    passphrase: String,
) -> Result<(), String> {
    let deps = &runtime.deps;
    use uc_core::security::model::{KeySlot, WrappedMasterKey};
    use uc_core::security::model::{EncryptionAlgo, MasterKey};

    // 1. Check if already initialized
    let state = deps.encryption_state
        .load_state()
        .await
        .map_err(|e| format!("Failed to load encryption state: {}", e))?;

    if state == EncryptionState::Initialized {
        return Err("Encryption is already initialized".to_string());
    }

    // ... more direct port calls
}
```

**Step 2: Replace with UseCases accessor pattern**

Replace the entire `initialize_encryption` function (lines 19-83) with:

```rust
/// Initialize encryption with passphrase
/// 使用密码短语初始化加密
#[tauri::command]
pub async fn initialize_encryption(
    runtime: State<'_, AppRuntime>,
    passphrase: String,
) -> Result<(), String> {
    let uc = runtime.usecases().initialize_encryption();
    uc.execute(uc_core::security::model::Passphrase(passphrase))
        .await
        .map_err(crate::commands::map_err)?;
    Ok(())
}
```

**Step 3: Add missing import at top of file**

Ensure this import is present:

```rust
use crate::commands::map_err;
```

**Step 4: Remove unused imports**

Remove these imports that are no longer needed:

```rust
use uc_core::security::model::{KeySlot, WrappedMasterKey};
use uc_core::security::model::{EncryptionAlgo, MasterKey};
```

**Step 5: Verify compilation**

Run: `cargo check -p uc-tauri`
Expected: SUCCESS, no errors

**Step 6: Test the command manually** (if development environment is set up)

Run: `bun tauri dev`
Test: Call `initialize_encryption` from frontend
Expected: Command executes successfully through use case

**Step 7: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/encryption.rs
git commit -m "refactor(uc-tauri): initialize_encryption command to use UseCases accessor"
```

---

## Task 4: Add TODO Comments for Other Commands

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/encryption.rs`
- Modify: `src-tauri/crates/uc-tauri/src/commands/clipboard.rs`
- Modify: `src-tauri/crates/uc-tauri/src/commands/settings.rs`

**Step 1: Update is_encryption_initialized command**

In `src-tauri/crates/uc-tauri/src/commands/encryption.rs`, update the `is_encryption_initialized` function:

Replace current implementation (lines 88-98) with:

```rust
/// Check if encryption is initialized
/// 检查加密是否已初始化
///
/// TODO: Implement IsEncryptionInitialized use case first.
/// This command should use: runtime.usecases().is_encryption_initialized()
///
/// Tracking: https://github.com/your-org/uniclipboard-desktop/issues/XXX
#[tauri::command]
pub async fn is_encryption_initialized(
    runtime: State<'_, AppRuntime>,
) -> Result<bool, String> {
    // TODO: Refactor to use UseCases accessor after implementing use case
    let _ = runtime; // Suppress unused warning until implemented
    Err("Not yet implemented - requires IsEncryptionInitialized use case".to_string())
}
```

**Step 2: Update delete_clipboard_entry command**

In `src-tauri/crates/uc-tauri/src/commands/clipboard.rs`, update the function:

```rust
/// Delete a clipboard entry
/// 删除剪贴板条目
///
/// TODO: Implement DeleteClipboardEntry use case first.
/// This command should use: runtime.usecases().delete_clipboard_entry()
///
/// Tracking: https://github.com/your-org/uniclipboard-desktop/issues/XXX
#[tauri::command]
pub async fn delete_clipboard_entry(
    runtime: State<'_, AppRuntime>,
    entry_id: String,
) -> Result<(), String> {
    // TODO: Refactor to use UseCases accessor after implementing use case
    let _ = runtime; // Suppress unused warning until implemented
    let _ = entry_id;
    Err("Not yet implemented - requires DeleteClipboardEntry use case".to_string())
}
```

**Step 3: Update capture_clipboard command**

In `src-tauri/crates/uc-tauri/src/commands/clipboard.rs`, update the function:

```rust
/// Capture current clipboard content
/// 捕获当前剪贴板内容
///
/// TODO: Implement CaptureClipboard use case first.
/// This command should use: runtime.usecases().capture_clipboard()
///
/// Tracking: https://github.com/your-org/uniclipboard-desktop/issues/XXX
#[tauri::command]
pub async fn capture_clipboard(
    runtime: State<'_, AppRuntime>,
) -> Result<String, String> {
    // TODO: Refactor to use UseCases accessor after implementing use case
    let _ = runtime; // Suppress unused warning until implemented
    Err("Not yet implemented - requires CaptureClipboard use case".to_string())
}
```

**Step 4: Update settings commands**

In `src-tauri/crates/uc-tauri/src/commands/settings.rs`, update both functions:

```rust
/// Get application settings
/// 获取应用设置
///
/// TODO: Implement GetSettings use case first.
/// This command should use: runtime.usecases().get_settings()
///
/// Tracking: https://github.com/your-org/uniclipboard-desktop/issues/XXX
#[tauri::command]
pub async fn get_settings(
    deps: State<'_, AppDeps>,
) -> Result<Value, String> {
    // TODO: Refactor to use UseCases accessor after implementing use case
    let _ = deps; // Suppress unused warning until implemented
    Err("Not yet implemented - requires GetSettings use case".to_string())
}

/// Update application settings
/// 更新应用设置
///
/// TODO: Implement UpdateSettings use case first.
/// This command should use: runtime.usecases().update_settings()
///
/// Tracking: https://github.com/your-org/uniclipboard-desktop/issues/XXX
#[tauri::command]
pub async fn update_settings(
    deps: State<'_, AppDeps>,
    settings: Value,
) -> Result<(), String> {
    // TODO: Refactor to use UseCases accessor after implementing use case
    let _ = deps; // Suppress unused warning until implemented
    let _ = settings;
    Err("Not yet implemented - requires UpdateSettings use case".to_string())
}
```

**Step 5: Verify compilation**

Run: `cargo check -p uc-tauri`
Expected: SUCCESS, no errors (warnings about unused code are OK)

**Step 6: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/
git commit -m "refactor(uc-tauri): add TODO comments for commands requiring use case implementation"
```

---

## Task 5: Update Documentation References

**Files:**

- Modify: `docs/architecture/commands-layer-specification.md`

**Step 1: Update the status table in the spec document**

In `docs/architecture/commands-layer-specification.md`, find the "Current Status" table and update `initialize_encryption` status:

```markdown
| Command                     | File                                                                                      | Status | Use Case Exists | Needs Refactor      |
| --------------------------- | ----------------------------------------------------------------------------------------- | ------ | --------------- | ------------------- |
| `get_clipboard_entries`     | [clipboard.rs:12-40](../../src-tauri/crates/uc-tauri/src/commands/clipboard.rs#L12-L40)   | ✅     | ✅              | No                  |
| `delete_clipboard_entry`    | [clipboard.rs:45-51](../../src-tauri/crates/uc-tauri/src/commands/clipboard.rs#L45-L51)   | TODO   | ❌              | **TODO**            |
| `capture_clipboard`         | [clipboard.rs:62-74](../../src-tauri/crates/uc-tauri/src/commands/clipboard.rs#L62-L74)   | TODO   | ⚠️              | **TODO**            |
| `initialize_encryption`     | [encryption.rs:19-30](../../src-tauri/crates/uc-tauri/src/commands/encryption.rs#L19-L30) | ✅     | ✅              | **No (refactored)** |
| `is_encryption_initialized` | [encryption.rs:40-50](../../src-tauri/crates/uc-tauri/src/commands/encryption.rs#L40-L50) | TODO   | ❌              | **TODO**            |
| `get_settings`              | [settings.rs:11-27](../../src-tauri/crates/uc-tauri/src/commands/settings.rs#L11-L27)     | TODO   | ❌              | **TODO**            |
| `update_settings`           | [settings.rs:29-43](../../src-tauri/crates/uc-tauri/src/commands/settings.rs#L29-L43)     | TODO   | ❌              | **TODO**            |
```

**Step 2: Commit**

```bash
git add docs/architecture/commands-layer-specification.md
git commit -m "docs: update commands layer spec status after refactor"
```

---

## Task 6: Final Verification

**Step 1: Run full project compilation**

Run: `cargo check`
Expected: SUCCESS across all crates

**Step 2: Run tests (if any exist)**

Run: `cargo test --workspace`
Expected: All existing tests pass

**Step 3: Manual testing (optional)**

If development environment is available:
Run: `bun tauri dev`
Test: Initialize encryption from frontend UI
Expected: Works as before, now through UseCases accessor

**Step 4: Create summary commit**

```bash
git add -A
git commit -m "docs: add implementation plan for Commands layer refactor"
```

---

## Verification Checklist

After completing all tasks:

- ☐ `initialize_encryption` uses `runtime.usecases().initialize_encryption()`
- ☐ No direct Port access in `initialize_encryption` command
- ☐ Type alias `InitializeEncryptionUseCase` exported from `uc-app`
- ☐ UseCases accessor has `initialize_encryption()` method
- ☐ Other commands have TODO comments with issue tracking
- ☐ All code compiles without errors
- ☐ Documentation updated to reflect new status
- ☐ All commits follow conventional commit format

---

## Architecture Compliance

**What Changed:**

- `initialize_encryption` command now goes through UseCases accessor
- Direct Port access removed from Commands layer
- Type alias added for cleaner UseCases accessor API

**What Stayed the Same:**

- Business logic remains in `InitializeEncryption` use case
- Frontend API unchanged (same command signature)
- All Ports still wired in bootstrap

**Architecture Rule Enforced:**

> Commands Layer MUST use `runtime.usecases().xxx()` to access use cases, NEVER `runtime.deps.xxx` directly.

---

## References

- [Commands Layer Specification](../architecture/commands-layer-specification.md) - Architecture rules
- [Architecture Principles](../architecture/principles.md) - Hexagonal architecture
- [Coding Standards](../standards/coding-standards.md) - Development standards
