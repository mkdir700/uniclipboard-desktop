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

| Command                     | File                                                                                        | Registered | Uses UseCases | Status      |
| --------------------------- | ------------------------------------------------------------------------------------------- | ---------- | ------------- | ----------- |
| `get_clipboard_entries`     | [clipboard.rs:12-39](../../src-tauri/crates/uc-tauri/src/commands/clipboard.rs#L12-L39)     | ✅         | ✅            | Complete    |
| `delete_clipboard_entry`    | [clipboard.rs:59-74](../../src-tauri/crates/uc-tauri/src/commands/clipboard.rs#L59-L74)     | ✅         | ✅            | Complete    |
| `capture_clipboard`         | [clipboard.rs:76-96](../../src-tauri/crates/uc-tauri/src/commands/clipboard.rs#L76-L96)     | ✅         | ❌            | Complex     |
| `initialize_encryption`     | [encryption.rs:21-31](../../src-tauri/crates/uc-tauri/src/commands/encryption.rs#L21-L31)   | ✅         | ✅            | Complete    |
| `is_encryption_initialized` | [encryption.rs:51-60](../../src-tauri/crates/uc-tauri/src/commands/encryption.rs#L51-L60)   | ✅         | ✅            | Complete    |
| `get_settings`              | [settings.rs:10-21](../../src-tauri/crates/uc-tauri/src/commands/settings.rs#L10-L21)       | ✅         | ✅            | Complete    |
| `update_settings`           | [settings.rs:23-38](../../src-tauri/crates/uc-tauri/src/commands/settings.rs#L23-L38)       | ✅         | ✅            | Complete    |

## Plugin Commands (External Dependencies)

| Command                      | File                                                                                                   | Registered | Type   | Status   |
| ---------------------------- | ------------------------------------------------------------------------------------------------------ | ---------- | ------ | -------- |
| `enable_rounded_corners`     | [plugins/mac_rounded_corners.rs:36-83](../../src-tauri/src/plugins/mac_rounded_corners.rs#L36-L83)     | ✅         | Plugin | External |
| `enable_modern_window_style` | [plugins/mac_rounded_corners.rs:86-143](../../src-tauri/src/plugins/mac_rounded_corners.rs#L86-L143)   | ✅         | Plugin | External |
| `reposition_traffic_lights`  | [plugins/mac_rounded_corners.rs:146-177](../../src-tauri/src/plugins/mac_rounded_corners.rs#L146-L177) | ✅         | Plugin | External |

## Legacy Bridge Commands (Temporary)

| Command                   | File                                                             | Registered | Uses UseCases | Status |
| ------------------------- | ---------------------------------------------------------------- | ---------- | ------------- | ------ |
| `check_onboarding_status` | [onboarding.rs:22-32](../../src-tauri/src/onboarding.rs#L22-L32) | ✅         | ❌            | Stub   |

**Note**: `check_onboarding_status` is a temporary stub implementation that returns hardcoded values. It needs to be migrated to the hexagonal architecture following the `IsEncryptionInitialized` pattern.

## Use Case Status

| Use Case                  | Exists | Location                                            | Used By Commands            |
| ------------------------- | ------ | --------------------------------------------------- | --------------------------- |
| `ListClipboardEntries`    | ✅     | `uc-app/src/usecases/list_clipboard_entries.rs`     | `get_clipboard_entries`     |
| `DeleteClipboardEntry`    | ✅     | `uc-app/src/usecases/delete_clipboard_entry.rs`     | `delete_clipboard_entry`    |
| `CaptureClipboard`        | ⚠️     | `uc-app/src/usecases/internal/capture_clipboard.rs` | `capture_clipboard` (TODO)  |
| `InitializeEncryption`    | ✅     | `uc-app/src/usecases/initialize_encryption.rs`      | `initialize_encryption`     |
| `IsEncryptionInitialized` | ✅     | `uc-app/src/usecases/is_encryption_initialized.rs`  | `is_encryption_initialized` |
| `GetSettings`             | ✅     | `uc-app/src/usecases/get_settings.rs`               | `get_settings`              |
| `UpdateSettings`          | ✅     | `uc-app/src/usecases/update_settings.rs`            | `update_settings`           |

## Migration Progress

**Core Commands: 6/7 using UseCases accessor (86%)**

**Note:** `capture_clipboard` requires complex multi-port orchestration and is tracked separately.

**Total Registered: 11 commands (7 core + 3 plugin + 1 bridge)**

### Completed ✅

1. **get_clipboard_entries** - Uses `ListClipboardEntries` via accessor
2. **delete_clipboard_entry** - Uses `DeleteClipboardEntry` via accessor
3. **initialize_encryption** - Uses `InitializeEncryption` via accessor
4. **is_encryption_initialized** - Uses `IsEncryptionInitialized` via accessor
5. **get_settings** - Uses `GetSettings` via accessor
6. **update_settings** - Uses `UpdateSettings` via accessor

### In Progress ⚠️

1. **capture_clipboard** - Complex multi-port use case required
   - Blocker: Requires orchestration of multiple ports
   - See: `docs/plans/2025-01-13-clipboard-capture-integration.md`

## Next Steps

1. ✅ Register all defined commands in `main.rs` invoke_handler
2. ✅ Refactor `is_encryption_initialized` to use UseCases accessor
3. ✅ Fix missing plugin command registrations (2025-01-14)
4. ⏳ Implement `CheckOnboardingStatus` use case and migrate command
5. ✅ Implement `GetSettings` and `UpdateSettings` use cases
6. ⏳ Update `capture_clipboard` command to use existing use case
7. ✅ Remove all direct `runtime.deps.xxx` access from commands (except capture_clipboard)

## Recent Changes

**2025-01-15**: Commands Layer refactoring to 86% complete

- Fixed plan documentation import path for SettingsPort
- Removed duplicate doc comment in main.rs
- Refactored `get_clipboard_entries` to use UseCases accessor
- Added `GetSettings` and `UpdateSettings` use cases
- Implemented `get_settings` and `update_settings` commands
- Extracted macOS platform commands to plugins module
- All settings commands now use UseCases accessor pattern

**2025-01-14**: Fixed command-not-found errors on startup

- Added `check_onboarding_status` command (stub implementation)
- Added macOS rounded corners plugin commands
- See: [docs/fixes/2025-01-14-tauri-commands-not-found.md](../fixes/2025-01-14-tauri-commands-not-found.md)

## References

- [Commands Layer Specification](./commands-layer-specification.md)
- [Hexagonal Architecture Principles](./principles.md)
- [Clipboard Capture Integration Plan](../plans/2025-01-13-clipboard-capture-integration.md)
