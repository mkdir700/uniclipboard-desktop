# Tracing Migration Phase 1: Command Layer Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add root spans to all Tauri command functions in the Command layer, establishing the entry point for distributed tracing across the application.

**Architecture:** Each Tauri command creates a root `info_span` at function entry with consistent naming (`command.<module>.<action>`) and required fields (e.g., `device_id`). The span encompasses the entire command execution, automatically capturing child spans from UseCase layer calls.

**Tech Stack:** `tracing` 0.1 (info_span!, info_span macros), existing Tauri commands in `src-tauri/crates/uc-tauri/src/commands/`.

---

## Prerequisites

**Required**: Phase 0 (Infrastructure Setup) must be completed before starting this phase.

**Verification**: Run `RUST_LOG=trace bun tauri dev` and confirm the application starts with tracing initialized.

---

## Task 1: Add tracing imports to commands/clipboard.rs

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/clipboard.rs:1-10`

**Step 1: Read current clipboard.rs imports**

Run: `head -10 src-tauri/crates/uc-tauri/src/commands/clipboard.rs`
Expected: See current imports (tauri::State, crate::bootstrap::AppRuntime, etc.).

**Step 2: Add tracing import**

Add to the imports section (after line 5):

```rust
//! Clipboard-related Tauri commands
//! 剪贴板相关的 Tauri 命令

use tauri::State;
use crate::bootstrap::AppRuntime;
use crate::models::ClipboardEntryProjection;
use tracing::info_span;  // NEW: Import for span creation
```

**Step 3: Run cargo check to verify**

Run: `cargo check -p uc-tauri`
Expected: No errors, tracing import resolved.

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/clipboard.rs
git commit -m "refactor(commands): add tracing import to clipboard commands

Import info_span macro for root span creation in command functions.

Part of Phase 1: Command layer migration"
```

---

## Task 2: Add root span to get_clipboard_entries command

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/clipboard.rs:10-37`

**Step 1: Modify get_clipboard_entries function**

Replace the existing function (lines 10-37) with:

```rust
/// Get clipboard history entries
/// 获取剪贴板历史条目
#[tauri::command]
pub async fn get_clipboard_entries(
    runtime: State<'_, AppRuntime>,
    limit: Option<usize>,
) -> Result<Vec<ClipboardEntryProjection>, String> {
    // Create root span for this command
    let span = info_span!(
        "command.clipboard.get_entries",
        device_id = %runtime.deps.device_identity.current_device_id(),
        limit = limit.unwrap_or(50),
    );
    let _enter = span.enter();

    // Use UseCases accessor pattern (consistent with other commands)
    let uc = runtime.usecases().list_clipboard_entries();
    let limit = limit.unwrap_or(50);

    // Query entries through use case
    let entries = uc.execute(limit, 0)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to get clipboard entries");
            e.to_string()
        })?;

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

    tracing::info!(count = projections.len(), "Retrieved clipboard entries");
    Ok(projections)
}
```

**Step 2: Run cargo check to verify**

Run: `cargo check -p uc-tauri`
Expected: No errors.

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/clipboard.rs
git commit -m "feat(commands): add root span to get_clipboard_entries

Create info_span with device_id and limit fields.
Log error on failure and entry count on success.

Span: command.clipboard.get_entries
Fields: device_id, limit

Part of Phase 1: Command layer migration"
```

---

## Task 3: Add root span to delete_clipboard_entry command

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/clipboard.rs:39-71`

**Step 1: Modify delete_clipboard_entry function**

Replace the existing function (lines 39-71) with:

````rust
/// Deletes a clipboard entry identified by `entry_id`.
///
/// This command converts the provided `entry_id` to the domain `EntryId` type and invokes the runtime's
/// delete clipboard-entry use case; on success it returns without value, otherwise it returns a stringified error.
///
/// # Examples
///
/// ```no_run
/// # async fn example(runtime: tauri::State<'_, uc_tauri::bootstrap::AppRuntime>) {
/// // Tauri provides `State<AppRuntime>` when invoking commands from the frontend.
/// let result = uc_tauri::commands::clipboard::delete_clipboard_entry(runtime, "entry-id-123".to_string()).await;
/// match result {
///     Ok(()) => println!("Deleted"),
///     Err(e) => eprintln!("Delete failed: {}", e),
/// }
/// # }
/// ```
#[tauri::command]
pub async fn delete_clipboard_entry(
    runtime: State<'_, AppRuntime>,
    entry_id: String,
) -> Result<(), String> {
    // Create root span for this command
    let span = info_span!(
        "command.clipboard.delete_entry",
        device_id = %runtime.deps.device_identity.current_device_id(),
        entry_id = %entry_id,
    );
    let _enter = span.enter();

    // Parse entry_id
    let parsed_id = match uc_core::ids::EntryId::from(entry_id.clone()) {
        Ok(id) => id,
        Err(e) => {
            tracing::error!(error = %e, entry_id = %entry_id, "Failed to parse entry_id");
            return Err(e.to_string());
        }
    };

    // Execute use case
    let use_case = runtime.usecases().delete_clipboard_entry();
    use_case.execute(&parsed_id)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, entry_id = %entry_id, "Failed to delete entry");
            e.to_string()
        })?;

    tracing::info!(entry_id = %entry_id, "Deleted clipboard entry");
    Ok(())
}
````

**Step 2: Run cargo check to verify**

Run: `cargo check -p uc-tauri`
Expected: No errors.

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/clipboard.rs
git commit -m "feat(commands): add root span to delete_clipboard_entry

Create info_span with device_id and entry_id fields.
Log parse errors and deletion errors.
Log success confirmation.

Span: command.clipboard.delete_entry
Fields: device_id, entry_id

Part of Phase 1: Command layer migration"
```

---

## Task 4: Add root span to capture_clipboard command (stub implementation)

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/clipboard.rs:73-134`

**Step 1: Modify capture_clipboard function**

Replace the existing function (lines 73-134) with:

````rust
/// Capture current clipboard content
/// 捕获当前剪贴板内容
///
/// **TODO**: Implement CaptureClipboard use case
/// **TODO**: This command currently returns placeholder error
/// **Tracking**: Complex use case requiring multiple ports
///
/// ## Required Changes / 所需更改
///
/// 1. Create `CaptureClipboard` use case in `uc-app/src/usecases/`
/// 2. Add `capture_clipboard()` method to `UseCases` accessor
/// 3. Update this command to use `runtime.usecases().capture_clipboard()`
///
/// ## Use Case Requirements / 用例需求
///
/// This is a complex use case that orchestrates multiple ports:
///
/// 1. **ClipboardSnapshotPort** - Read current clipboard content
/// 2. **MaterializationPort** - Convert raw data to representations
/// 3. **ClipboardEventWriterPort** - Create and persist clipboard event
/// 4. **ClipboardEntryRepositoryPort** - Store entry in database
///
/// ## Architecture Flow / 架构流程
///
/// ```text
/// Frontend → Command → CaptureClipboard Use Case → Multiple Ports
///                                      ↓
///                    1. Snapshot (ClipboardSnapshotPort)
///                    2. Materialize (MaterializationPort)
///                    3. Create Event (ClipboardEventWriterPort)
///                    4. Persist (ClipboardEntryRepositoryPort)
/// ```
///
/// ## Issue Tracking / 问题跟踪
///
/// - [ ] Create use case: `uc-app/src/usecases/capture_clipboard.rs`
/// - [ ] Add ClipboardSnapshotPort to uc-core/ports/
/// - [ ] Add MaterializationPort to uc-core/ports/
/// - [ ] Add ClipboardEventWriterPort to uc-core/ports/
/// - [ ] Implement ports in uc-platform/ (clipboard adapters)
/// - [ ] Add to UseCases accessor: `uc-tauri/src/bootstrap/runtime.rs`
/// - [ ] Update command implementation
#[tauri::command]
pub async fn capture_clipboard(
    runtime: State<'_, AppRuntime>,
) -> Result<String, String> {
    // Create root span for this command (even in stub state)
    let span = info_span!(
        "command.clipboard.capture",
        device_id = %runtime.deps.device_identity.current_device_id(),
    );
    let _enter = span.enter();

    // TODO: Implement CaptureClipboard use case
    // This is a complex use case requiring:
    //
    // 1. Create use case in uc-app/src/usecases/
    // 2. Define required ports in uc-core/ports/:
    //    - ClipboardSnapshotPort (read clipboard)
    //    - MaterializationPort (convert to representations)
    //    - ClipboardEventWriterPort (create event)
    // 3. Implement ports in uc-platform/adapters/
    // 4. Add to UseCases accessor in uc-tauri/src/bootstrap/runtime.rs
    // 5. Wire all dependencies
    // 6. Update this command to use runtime.usecases().capture_clipboard()
    //
    // Tracking: Complex multi-port orchestration

    tracing::warn!("capture_clipboard command not yet implemented");
    Err("Not yet implemented - requires CaptureClipboard use case with multiple ports".to_string())
}
````

**Step 2: Run cargo check to verify**

Run: `cargo check -p uc-tauri`
Expected: No errors.

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/clipboard.rs
git commit -m "feat(commands): add root span to capture_clipboard

Create info_span with device_id field.
Log warning for stub implementation.

Span: command.clipboard.capture
Fields: device_id

Note: Full implementation requires multi-port use case.
Part of Phase 1: Command layer migration"
```

---

## Task 5: Add tracing imports to commands/encryption.rs

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/encryption.rs:1-6`

**Step 1: Add tracing import**

Add to the imports section:

```rust
//! Encryption-related Tauri commands
//! 加密相关的 Tauri 命令

use tauri::State;
use crate::bootstrap::AppRuntime;
use tracing::info_span;  // NEW
```

**Step 2: Run cargo check to verify**

Run: `cargo check -p uc-tauri`
Expected: No errors.

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/encryption.rs
git commit -m "refactor(commands): add tracing import to encryption commands

Import info_span macro for root span creation.

Part of Phase 1: Command layer migration"
```

---

## Task 6: Add root span to initialize_encryption command

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/encryption.rs:8-30`

**Step 1: Modify initialize_encryption function**

Replace the existing function with:

```rust
/// Initialize encryption with passphrase
/// 使用密码短语初始化加密
///
/// This command uses the InitializeEncryption use case through the UseCases accessor.
/// 此命令通过 UseCases 访问器使用 InitializeEncryption 用例。
///
/// ## Architecture / 架构
///
/// - Commands layer (Driving Adapter) → UseCases accessor → Use Case → Ports
/// - No direct Port access from commands
/// - 命令层（驱动适配器）→ UseCases 访问器 → 用例 → 端口
/// - 命令不直接访问端口
#[tauri::command]
pub async fn initialize_encryption(
    runtime: State<'_, AppRuntime>,
    passphrase: String,
) -> Result<(), String> {
    let span = info_span!(
        "command.encryption.initialize",
        device_id = %runtime.deps.device_identity.current_device_id(),
    );
    let _enter = span.enter();

    let uc = runtime.usecases().initialize_encryption();
    uc.execute(uc_core::security::model::Passphrase(passphrase))
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to initialize encryption");
            e.to_string()
        })?;

    tracing::info!("Encryption initialized successfully");
    Ok(())
}
```

**Step 2: Run cargo check to verify**

Run: `cargo check -p uc-tauri`
Expected: No errors.

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/encryption.rs
git commit -m "feat(commands): add root span to initialize_encryption

Create info_span with device_id field.
Log error on failure, success on completion.

Span: command.encryption.initialize
Fields: device_id

Part of Phase 1: Command layer migration"
```

---

## Task 7: Add root span to is_encryption_initialized command

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/encryption.rs:32-44`

**Step 1: Modify is_encryption_initialized function**

Replace the existing function with:

```rust
/// Check if encryption is initialized
/// 检查加密是否已初始化
///
/// This command uses the IsEncryptionInitialized use case.
/// 此命令使用 IsEncryptionInitialized 用例。
#[tauri::command]
pub async fn is_encryption_initialized(
    runtime: State<'_, AppRuntime>,
) -> Result<bool, String> {
    let span = info_span!(
        "command.encryption.is_initialized",
        device_id = %runtime.deps.device_identity.current_device_id(),
    );
    let _enter = span.enter();

    let uc = runtime.usecases().is_encryption_initialized();
    let result = uc.execute().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to check encryption status");
        e.to_string()
    })?;

    tracing::info!(is_initialized = result, "Encryption status checked");
    Ok(result)
}
```

**Step 2: Run cargo check to verify**

Run: `cargo check -p uc-tauri`
Expected: No errors.

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/encryption.rs
git commit -m "feat(commands): add root span to is_encryption_initialized

Create info_span with device_id field.
Log result (is_initialized) on success.

Span: command.encryption.is_initialized
Fields: device_id

Part of Phase 1: Command layer migration"
```

---

## Task 8: Add tracing imports to commands/settings.rs

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/settings.rs:1-8`

**Step 1: Add tracing import**

Add to the imports section:

```rust
//! Settings-related Tauri commands
//! 设置相关的 Tauri 命令

use serde_json::Value;
use tauri::State;
use uc_core::settings::model::Settings;
use crate::bootstrap::AppRuntime;
use tracing::info_span;  // NEW
```

**Step 2: Run cargo check to verify**

Run: `cargo check -p uc-tauri`
Expected: No errors.

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/settings.rs
git commit -m "refactor(commands): add tracing import to settings commands

Import info_span macro for root span creation.

Part of Phase 1: Command layer migration"
```

---

## Task 9: Add root span to get_settings command

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/settings.rs:9-25`

**Step 1: Modify get_settings function**

Replace the existing function with:

```rust
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
    let span = info_span!(
        "command.settings.get",
        device_id = %runtime.deps.device_identity.current_device_id(),
    );
    let _enter = span.enter();

    let uc = runtime.usecases().get_settings();
    let settings = uc.execute().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get settings");
        e.to_string()
    })?;

    // Convert Settings to JSON value
    let json_value = serde_json::to_value(&settings)
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to serialize settings");
            format!("Failed to serialize settings: {}", e)
        })?;

    tracing::info!("Retrieved settings successfully");
    Ok(json_value)
}
```

**Step 2: Run cargo check to verify**

Run: `cargo check -p uc-tauri`
Expected: No errors.

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/settings.rs
git commit -m "feat(commands): add root span to get_settings

Create info_span with device_id field.
Log error on failure, success on completion.

Span: command.settings.get
Fields: device_id

Part of Phase 1: Command layer migration"
```

---

## Task 10: Add root span to update_settings command

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/settings.rs:27-46`

**Step 1: Modify update_settings function**

Replace the existing function with:

```rust
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
    let span = info_span!(
        "command.settings.update",
        device_id = %runtime.deps.device_identity.current_device_id(),
    );
    let _enter = span.enter();

    // Parse JSON into Settings domain model
    let parsed_settings: Settings = serde_json::from_value(settings.clone())
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to parse settings JSON");
            format!("Failed to parse settings: {}", e)
        })?;

    let uc = runtime.usecases().update_settings();
    uc.execute(parsed_settings).await.map_err(|e| {
        tracing::error!(error = %e, "Failed to update settings");
        e.to_string()
    })?;

    tracing::info!("Settings updated successfully");
    Ok(())
}
```

**Step 2: Run cargo check to verify**

Run: `cargo check -p uc-tauri`
Expected: No errors.

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/settings.rs
git commit -m "feat(commands): add root span to update_settings

Create info_span with device_id field.
Log parse errors and update errors.
Log success on completion.

Span: command.settings.update
Fields: device_id

Part of Phase 1: Command layer migration"
```

---

## Task 11: Verify root spans in development environment

**Files:**

- None (verification step)

**Step 1: Start development server with verbose logging**

Run: `RUST_LOG=info bun tauri dev`
Expected:

- Application starts
- Logs show command execution with span prefixes

**Step 2: Trigger a command from frontend**

Open the application and trigger any action (e.g., get clipboard entries).

**Step 3: Observe span output in terminal**

Look for log lines like:

```
2025-01-15 10:30:45.123 INFO [clipboard.rs:42] [command.clipboard.get_entries{device_id=abc123,limit=50}] Retrieved clipboard entries
```

**Expected output pattern:**

- Span name: `command.clipboard.get_entries`
- Fields in braces: `device_id=xxx`, `limit=50`
- File location shows the command file
- Message after closing brace

**Step 4: Verify error handling**

Try an operation that fails (e.g., invalid entry_id deletion) and confirm error logs appear:

```
2025-01-15 10:31:00.456 ERROR [clipboard.rs:72] [command.clipboard.delete_entry{device_id=abc123,entry_id=invalid}] Failed to parse entry_id
```

**Step 5: Create verification note**

```bash
cat > /tmp/phase1_verification.md << 'EOF'
# Phase 1 Verification Results

## Date
$(date +%Y-%m-%d)

## Commands Migrated
- [x] command.clipboard.get_entries
- [x] command.clipboard.delete_entry
- [x] command.clipboard.capture
- [x] command.encryption.initialize
- [x] command.encryption.is_initialized
- [x] command.settings.get
- [x] command.settings.update

## Span Format Verified
- [x] All spans use <layer>.<module>.<action> naming
- [x] All spans include device_id field
- [x] Span-specific fields present (limit, entry_id)
- [x] Error logging with span context
- [x] Success logging with span context

## Log Output Sample
(Paste sample output from Step 3)

## Ready for Phase 2
Yes - Command layer root spans are in place.
Next: Migrate UseCase layer to create child spans.
EOF
cat /tmp/phase1_verification.md
```

**Step 6: Commit verification milestone**

```bash
git add docs/plans/2025-01-15-tracing-phase1-implementation.md
git commit --allow-empty -m "test(verification): Phase 1 Command layer complete

All Tauri commands now create root spans:
- 7 commands migrated across 3 modules
- Consistent naming: command.<module>.<action>
- Required fields: device_id
- Error and success logging with span context

Verification:
- Spans visible in RUST_LOG=info output
- Span fields correctly formatted
- Error handling preserves span context

Ready to proceed to Phase 2: UseCase layer migration."
```

---

## Summary

After completing all tasks, the Command layer will have complete root span coverage:

**Commands Migrated**: 7 commands across 3 modules

- `commands/clipboard.rs`: 3 commands (get_entries, delete_entry, capture)
- `commands/encryption.rs`: 2 commands (initialize, is_initialized)
- `commands/settings.rs`: 2 commands (get, update)

**Span Naming Convention**:

- Format: `command.<module>.<action>`
- Examples: `command.clipboard.get_entries`, `command.encryption.initialize`

**Required Fields**:

- All spans: `device_id` (from `runtime.deps.device_identity.current_device_id()`)
- Span-specific: `limit`, `entry_id`, etc.

**Error Handling**:

- All errors logged with `tracing::error!` inside span context
- Errors include `error = %e` field for structured capture

**Next Phase**:

- Phase 2: Migrate UseCase layer to create child spans
- UseCase spans will automatically become children of Command spans

**Verification Command**:

```bash
RUST_LOG=info bun tauri dev
```

Should see span-braced log entries like:

```
INFO [clipboard.rs:42] [command.clipboard.get_entries{device_id=abc,limit=50}] Retrieved clipboard entries
```

---

## References

- Design document: `docs/plans/2025-01-15-tracing-migration-design.md`
- Phase 0 plan: `docs/plans/2025-01-15-tracing-phase0-implementation.md`
- Span naming: Section 2.1 of design document
- Command layer spec: Section 3 of design document
