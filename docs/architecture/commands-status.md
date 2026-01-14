# Tauri Commands Architecture Status

**Last Updated**: 2025-01-14

## Overview

This document tracks the migration status of Tauri commands from direct Port access to proper UseCases accessor pattern following Hexagonal Architecture principles.

## Architecture Pattern

**Target Pattern**: Commands Layer (Driving Adapter) → UseCases Accessor → Use Case → Ports

```
┌─────────────────────────────────────────────────────────────────┐
│                        Frontend (React)                         │
└────────────────────────────┬────────────────────────────────────┘
                             │ Tauri IPC
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Commands Layer (uc-tauri)                    │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  #[tauri::command] fn xxx(runtime: State<AppRuntime>)    │  │
│  │      let uc = runtime.usecases().xxx();                  │  │
│  │      uc.execute().await                                  │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                   UseCases Accessor Pattern                     │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  impl AppRuntime {                                       │  │
│  │      fn usecases(&self) -> UseCases<'_>                 │  │
│  │  }                                                       │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Application Layer (uc-app)                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  pub struct XxxUseCase {                                 │  │
│  │      port: Arc<dyn Port>,                                │  │
│  │  }                                                       │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Port Layer (uc-core)                       │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  pub trait XxxPort: Send + Sync {                        │  │
│  │      async fn operation(&self) -> Result<()>;            │  │
│  │  }                                                       │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## Commands Status Matrix

| Command                     | Category   | UseCase Accessor | Status       | Notes                          |
| --------------------------- | ---------- | ---------------- | ------------ | ------------------------------ |
| `get_clipboard_entries`     | Clipboard  | ✅ Yes           | ✅ Compliant | Uses `ListClipboardEntries`    |
| `delete_clipboard_entry`    | Clipboard  | ✅ Yes           | ✅ Compliant | Uses `DeleteClipboardEntry`    |
| `capture_clipboard`         | Clipboard  | ✅ Yes           | ✅ Compliant | Uses `CaptureClipboard`        |
| `initialize_encryption`     | Encryption | ✅ Yes           | ✅ Compliant | Uses `InitializeEncryption`    |
| `is_encryption_initialized` | Encryption | ✅ Yes           | ✅ Compliant | Uses `IsEncryptionInitialized` |
| `get_settings`              | Settings   | ❌ No            | ⚠️ Legacy    | Direct Port access             |
| `update_settings`           | Settings   | ❌ No            | ⚠️ Legacy    | Direct Port access             |

## Use Cases Status

| Use Case                  | Module      | Implementation | Dependencies                                                               |
| ------------------------- | ----------- | -------------- | -------------------------------------------------------------------------- |
| `ListClipboardEntries`    | ✅ Complete | ✅ Implemented | `ClipboardSelectionPort`, `BlobMaterializerPort`                           |
| `DeleteClipboardEntry`    | ✅ Complete | ✅ Implemented | `ClipboardSelectionPort`                                                   |
| `CaptureClipboard`        | ✅ Complete | ✅ Implemented | `ClipboardSelectionPort`, `BlobMaterializerPort`, `ClipboardChangeHandler` |
| `InitializeEncryption`    | ✅ Complete | ✅ Implemented | `EncryptionPort`, `KeyMaterialPort`, `KeyScopePort`, `EncryptionStatePort` |
| `IsEncryptionInitialized` | ✅ Complete | ✅ Implemented | `EncryptionStatePort`                                                      |
| `GetSettings`             | ❌ Missing  | ⚠️ Not Created | N/A                                                                        |
| `UpdateSettings`          | ❌ Missing  | ⚠️ Not Created | N/A                                                                        |

## Migration Checklist

- [x] Task 1: Register `get_settings` and `update_settings` in `main.rs` invoke_handler
- [x] Task 2: Create `IsEncryptionInitialized` use case
- [x] Task 3: Update `is_encryption_initialized` command to use UseCases accessor
- [x] Task 4: Verify all commands are properly registered
- [x] Task 5: Document current architecture status

## Next Steps

1. **Create `GetSettings` use case** (uc-app)
   - Port: `SettingsPort` (already exists in uc-core)
   - Method: `get_settings() -> Result<Settings>`

2. **Create `UpdateSettings` use case** (uc-app)
   - Port: `SettingsPort` (already exists in uc-core)
   - Method: `update_settings(settings: Settings) -> Result<()>`

3. **Add accessor methods to `UseCases`** (uc-tauri/bootstrap/runtime.rs)
   - `get_settings()` → `GetSettings`
   - `update_settings()` → `UpdateSettings`

4. **Refactor settings commands** (uc-tauri/commands/settings.rs)
   - Replace direct `runtime.deps.settings` access
   - Use `runtime.usecases().get_settings()`
   - Use `runtime.usecases().update_settings()`

## Architecture Principles

### DO ✅

- Commands MUST use `runtime.usecases().xxx()` pattern
- UseCases MUST be accessed through the `UseCases` accessor
- UseCases MUST use `Arc<dyn Port>` trait objects
- All Ports MUST be defined in `uc-core/ports`

### DON'T ❌

- Commands MUST NOT access `runtime.deps.xxx` directly
- Commands MUST NOT access `runtime.deps.settings` or other ports
- UseCases MUST NOT be generic (use trait objects instead)

## References

- Plan: `docs/plans/2026-01-14-tauri-commands-registration-fix.md`
- UseCases: `src-tauri/crates/uc-app/src/usecases/`
- Ports: `src-tauri/crates/uc-core/src/ports/`
- Commands: `src-tauri/crates/uc-tauri/src/commands/`
