# Tauri Commands Registration Fix Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix "Tauri commands not found" error by registering missing commands in invoke_handler and ensuring proper UseCases accessor pattern compliance.

**Architecture:** Commands layer is a Driving Adapter that MUST call Use Cases through the UseCases accessor. Commands defined in `uc-tauri/src/commands/` must be registered in `main.rs` `invoke_handler`.

**Tech Stack:** Rust, Tauri 2, Hexagonal Architecture

---

## Context for Implementation

### Current State

**Problem:** The `get_settings` and `update_settings` commands are defined in `src-tauri/crates/uc-tauri/src/commands/settings.rs` but NOT registered in `src-tauri/src/main.rs` invoke_handler. This causes "command not found" errors when frontend tries to call them.

**Root Cause:** During hexagonal architecture refactoring, new commands were added but not registered in the Tauri builder.

**Architecture Status:**

- ✅ `get_clipboard_entries` - Registered, uses UseCases accessor
- ✅ `delete_clipboard_entry` - Registered, uses UseCases accessor
- ✅ `initialize_encryption` - Registered, uses UseCases accessor
- ⚠️ `capture_clipboard` - Registered but returns placeholder error
- ❌ `get_settings` - Defined but NOT registered
- ❌ `update_settings` - Defined but NOT registered

### Files Involved

- `src-tauri/src/main.rs` - Missing command registrations
- `src-tauri/crates/uc-tauri/src/commands/settings.rs` - Commands not registered
- `src-tauri/crates/uc-tauri/src/commands/encryption.rs` - Direct Port access
- `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs` - UseCases accessor

---

## Task 1: Register Missing Commands in main.rs

**Files:**

- Modify: `src-tauri/src/main.rs`

**Step 1: Read current invoke_handler**

The current invoke_handler (lines 195-202) only has 5 commands:

```rust
.invoke_handler(tauri::generate_handler![
    // Clipboard commands
    uc_tauri::commands::clipboard::get_clipboard_entries,
    uc_tauri::commands::clipboard::delete_clipboard_entry,
    uc_tauri::commands::clipboard::capture_clipboard,
    // Encryption commands
    uc_tauri::commands::encryption::initialize_encryption,
])
```

**Step 2: Add missing settings commands**

Add the settings commands section after the encryption commands (after line 202):

```rust
.invoke_handler(tauri::generate_handler![
    // Clipboard commands
    uc_tauri::commands::clipboard::get_clipboard_entries,
    uc_tauri::commands::clipboard::delete_clipboard_entry,
    uc_tauri::commands::clipboard::capture_clipboard,
    // Encryption commands
    uc_tauri::commands::encryption::initialize_encryption,
    // Settings commands
    uc_tauri::commands::settings::get_settings,
    uc_tauri::commands::settings::update_settings,
])
```

**Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`

Expected: SUCCESS, no errors

**Step 4: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "fix(main): register missing settings commands in invoke_handler

Add get_settings and update_settings to Tauri's invoke_handler.
These commands were defined but not registered, causing 'command not found'
errors when frontend tried to call them.
"
```

---

Add new use case to check encryption initialization status.
Update command to use UseCases accessor pattern instead of
direct Port access, following hexagonal architecture principles.
"

````

---

## Task 3: Verify All Commands Are Properly Registered

**Step 1: List all defined commands**

Run: `grep -r "#\[tauri::command\]" src-tauri/crates/uc-tauri/src/commands/`

Expected output should show all command functions:

- `get_clipboard_entries`
- `delete_clipboard_entry`
- `capture_clipboard`
- `initialize_encryption`
- `get_settings`
- `update_settings`

**Step 2: Verify invoke_handler has all commands**

Read: `src-tauri/src/main.rs`

Check that the `invoke_handler` section includes all 6 commands.

**Step 3: Run full compilation**

Run: `cd src-tauri && cargo check`

Expected: SUCCESS across all crates

**Step 4: Test command availability manually** (optional)

Run: `bun tauri dev`

Test: Check browser console for Tauri API availability

Expected: All commands should be accessible via `__TAURI__.core.invoke()`

**Step 5: Create verification commit**

```bash
git add -A
git commit -m "chore: verify all commands are registered and compiled

All 6 commands are now properly defined and registered:
- 3 clipboard commands
- 1 encryption command
- 2 settings commands

All use UseCases accessor pattern (except placeholder commands
pending use case implementation).
"
````

---

## Task 4: Document Current Architecture Status

**Files:**

- Create: `docs/architecture/commands-status.md`

**Step 1: Create status documentation**

Create: `docs/architecture/commands-status.md`

```markdown
# Tauri Commands Architecture Status

## Overview

This document tracks the current status of all Tauri commands in the uniclipboard-desktop
application, showing migration progress from direct Port access to UseCases accessor pattern.

## Architecture Principle

> **Commands Layer MUST use `runtime.usecases().xxx()` to access use cases, NEVER `runtime.deps.xxx` directly.**

Commands are **Driving Adapters** in Hexagonal Architecture:

- Input: Frontend calls via Tauri IPC
- Output: Use case invocation through accessor
- Rule: No direct Port access, no business logic

## Command Status Matrix

| Command                  | File                                                                                        | Registered | Uses UseCases | Status      |
| ------------------------ | ------------------------------------------------------------------------------------------- | ---------- | ------------- | ----------- |
| `get_clipboard_entries`  | [clipboard.rs:12-40](../../src-tauri/crates/uc-tauri/src/commands/clipboard.rs#L12-L40)     | ✅         | ✅            | Complete    |
| `delete_clipboard_entry` | [clipboard.rs:59-74](../../src-tauri/crates/uc-tauri/src/commands/clipboard.rs#L59-L74)     | ✅         | ✅            | Complete    |
| `capture_clipboard`      | [clipboard.rs:118-137](../../src-tauri/crates/uc-tauri/src/commands/clipboard.rs#L118-L137) | ✅         | ❌            | Placeholder |
| `initialize_encryption`  | [encryption.rs:21-31](../../src-tauri/crates/uc-tauri/src/commands/encryption.rs#L21-L31)   | ✅         | ✅            | Complete    |
| `get_settings`           | [settings.rs:37-49](../../src-tauri/crates/uc-tauri/src/commands/settings.rs#L37-L49)       | ✅         | ❌            | Placeholder |
| `update_settings`        | [settings.rs:81-94](../../src-tauri/crates/uc-tauri/src/commands/settings.rs#L81-L94)       | ✅         | ❌            | Placeholder |

## Use Case Status

| Use Case               | Exists | Location                                            | Used By Commands           |
| ---------------------- | ------ | --------------------------------------------------- | -------------------------- |
| `ListClipboardEntries` | ✅     | `uc-app/src/usecases/list_clipboard_entries.rs`     | `get_clipboard_entries`    |
| `DeleteClipboardEntry` | ✅     | `uc-app/src/usecases/delete_clipboard_entry.rs`     | `delete_clipboard_entry`   |
| `CaptureClipboard`     | ⚠️     | `uc-app/src/usecases/internal/capture_clipboard.rs` | `capture_clipboard` (TODO) |
| `InitializeEncryption` | ✅     | `uc-app/src/usecases/initialize_encryption.rs`      | `initialize_encryption`    |
| `GetSettings`          | ❌     | -                                                   | `get_settings` (TODO)      |
| `UpdateSettings`       | ❌     | -                                                   | `update_settings` (TODO)   |

## Migration Progress

**Complete: 4/6 commands (67%)**

### Completed ✅

1. **get_clipboard_entries** - Uses `ListClipboardEntries` via accessor
2. **delete_clipboard_entry** - Uses `DeleteClipboardEntry` via accessor
3. **initialize_encryption** - Uses `InitializeEncryption` via accessor

### In Progress ⚠️

1. **capture_clipboard** - Use case exists (`CaptureClipboardUseCase`) but command not updated
   - Blocker: Complex multi-port orchestration
   - See: `docs/plans/2025-01-13-clipboard-capture-integration.md`

### Pending ❌

1. **get_settings** - Needs `GetSettings` use case
2. **update_settings** - Needs `UpdateSettings` use case

## Next Steps

1. ✅ Register all defined commands in `main.rs` invoke_handler
2. ⏳ Implement `GetSettings` and `UpdateSettings` use cases
3. ⏳ Update `capture_clipboard` command to use existing use case
4. ⏳ Remove all direct `runtime.deps.xxx` access from commands

## References

- [Commands Layer Specification](./commands-layer-specification.md)
- [Hexagonal Architecture Principles](./principles.md)
- [Clipboard Capture Integration Plan](../plans/2025-01-13-clipboard-capture-integration.md)
```

**Step 2: Update CLAUDE.md with quick reference**

Modify: `CLAUDE.md`

Add section after "## Tauri Commands":

```markdown
## Commands Layer Status

**Current Migration Status:** 4/6 commands using UseCases accessor (67%)

When adding new commands:

1. Define command function in `src-tauri/crates/uc-tauri/src/commands/`
2. Create/refer to use case in `uc-app/src/usecases/`
3. Add accessor method to `UseCases` in `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`
4. Register in `invoke_handler![]` in `src-tauri/src/main.rs`
5. Use `runtime.usecases().xxx()` - NEVER `runtime.deps.xxx`

See `docs/architecture/commands-status.md` for detailed status.
```

**Step 3: Commit documentation**

```bash
git add docs/architecture/commands-status.md CLAUDE.md
git commit -m "docs: add commands layer architecture status documentation

Track migration progress from direct Port access to UseCases accessor pattern.
Document current status of all 7 commands and associated use cases.
"
```

---

## Summary

This plan fixes the "Tauri commands not found" issue and ensures proper hexagonal architecture compliance:

1. ✅ Registers missing `get_settings` and `update_settings` commands
2. ✅ Documents current architecture status

**Architecture Compliance Achieved:**

- All commands registered in `invoke_handler`
- All implemented commands use `runtime.usecases().xxx()` pattern
- Direct Port access eliminated from commands layer
- Clear documentation of migration status

**Testing Strategy:**

1. Compile check: `cargo check`
2. Manual test: `bun tauri dev` and call commands from frontend
3. Verify no direct Port access in commands

**Estimated Time:** 1-2 hours for full implementation

---

## Verification Checklist

After completing all tasks:

- ☐ `get_settings` and `update_settings` registered in main.rs
- ☐ All commands compile without errors
- ☐ No direct `runtime.deps.xxx` access in implemented commands
- ☐ Documentation updated with current status
- ☐ All commits follow conventional commit format

---

## References

- [Commands Layer Specification](../architecture/commands-layer-specification.md) - Architecture rules
- [Clipboard Capture Integration Plan](../plans/2025-01-13-clipboard-capture-integration.md) - Related plan
- [Commands Layer Refactor Plan](../plans/2025-01-13-commands-layer-refactor.md) - Original refactor plan
