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

| Command                     | File                                                                                      | Registered | Uses UseCases | Status   |
| --------------------------- | ----------------------------------------------------------------------------------------- | ---------- | ------------- | -------- |
| `get_clipboard_entries`     | [clipboard.rs:12-39](../../src-tauri/crates/uc-tauri/src/commands/clipboard.rs#L12-L39)   | ✅         | ✅            | Complete |
| `delete_clipboard_entry`    | [clipboard.rs:59-74](../../src-tauri/crates/uc-tauri/src/commands/clipboard.rs#L59-L74)   | ✅         | ✅            | Complete |
| `capture_clipboard`         | [clipboard.rs:76-96](../../src-tauri/crates/uc-tauri/src/commands/clipboard.rs#L76-L96)   | ✅         | ❌            | Complex  |
| `initialize_encryption`     | [encryption.rs:21-31](../../src-tauri/crates/uc-tauri/src/commands/encryption.rs#L21-L31) | ✅         | ✅            | Complete |
| `is_encryption_initialized` | [encryption.rs:51-60](../../src-tauri/crates/uc-tauri/src/commands/encryption.rs#L51-L60) | ✅         | ✅            | Complete |
| `get_settings`              | [settings.rs:17-25](../../src-tauri/crates/uc-tauri/src/commands/settings.rs#L17-L25)     | ✅         | ✅            | Complete |
| `update_settings`           | [settings.rs:35-45](../../src-tauri/crates/uc-tauri/src/commands/settings.rs#L35-L45)     | ✅         | ✅            | Complete |

## Plugin Commands (External Dependencies)

| Command                      | File                                                                                                   | Registered | Type   | Status   |
| ---------------------------- | ------------------------------------------------------------------------------------------------------ | ---------- | ------ | -------- |
| `enable_rounded_corners`     | [plugins/mac_rounded_corners.rs:36-83](../../src-tauri/src/plugins/mac_rounded_corners.rs#L36-L83)     | ✅         | Plugin | External |
| `enable_modern_window_style` | [plugins/mac_rounded_corners.rs:86-143](../../src-tauri/src/plugins/mac_rounded_corners.rs#L86-L143)   | ✅         | Plugin | External |
| `reposition_traffic_lights`  | [plugins/mac_rounded_corners.rs:146-177](../../src-tauri/src/plugins/mac_rounded_corners.rs#L146-L177) | ✅         | Plugin | External |

## Legacy Bridge Commands (Temporary)

None. Legacy onboarding status checks were superseded by `get_onboarding_state`.

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

**Core Commands: 7/7 using UseCases accessor (100%)**
**Total Registered: 10 commands (7 core + 3 plugin)**

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

### Pending ❌

None

## Next Steps

1. ✅ Register all defined commands in `main.rs` invoke_handler
2. ✅ Refactor `is_encryption_initialized` to use UseCases accessor
3. ✅ Fix missing plugin command registrations (2025-01-14)
4. ✅ Implement `GetSettings` and `UpdateSettings` use cases
5. ⏳ Update `capture_clipboard` command to use existing use case

## Recent Changes

**2025-01-14**: Settings use cases implementation complete

- Implemented `GetSettings` and `UpdateSettings` use cases
- Migrated `get_settings` and `update_settings` commands to UseCases accessor pattern
- **Commands Layer Migration: 100% Complete** (7/7 core commands using accessor)
- See: [docs/plans/2025-01-14-settings-use-cases.md](../plans/2025-01-14-settings-use-cases.md)

**2025-01-14**: Fixed command-not-found errors on startup

- Added `check_onboarding_status` command (stub implementation)
- Added macOS rounded corners plugin commands
- See: [docs/fixes/2025-01-14-tauri-commands-not-found.md](../fixes/2025-01-14-tauri-commands-not-found.md)

## References

- [Commands Layer Specification](./commands-layer-specification.md)
- [Hexagonal Architecture Principles](./principles.md)
- [Clipboard Capture Integration Plan](../plans/2025-01-13-clipboard-capture-integration.md)
