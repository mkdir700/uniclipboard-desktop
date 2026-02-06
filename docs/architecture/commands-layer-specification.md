# Commands Layer Architecture Specification

This document defines the mandatory architecture rules for the **Commands Layer** (`uc-tauri/src/commands/`) in UniClipboard's hexagonal architecture.

## Overview

The Commands Layer is a **Driving Adapter** in hexagonal architecture. It translates external requests (Tauri IPC from the frontend) into use case executions.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         Frontend (React)                        │
│                    Tauri IPC Commands                          │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Commands Layer (适配器)                       │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  #[tauri::command] fn xxx() {                             │  │
│  │      let uc = runtime.usecases().xxx();                   │  │
│  │      uc.execute(...).await                                │  │
│  │  }                                                        │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                   UseCases Layer (应用层)                        │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  pub struct InitializeEncryption<E, K, KS, ES> {          │  │
│  │      encryption: Arc<E>,                                  │  │
│  │      key_material: Arc<K>,                                │  │
│  │      ...                                                  │  │
│  │  }                                                        │  │
│  │                                                          │  │
│  │  impl<E, K, KS, ES> InitializeEncryption<E, K, KS, ES> { │  │
│  │      pub fn execute(...) -> Result<(), Error> { ... }    │  │
│  │  }                                                        │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Ports (端口接口层)                           │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  trait EncryptionPort { fn derive_kek(...) }             │  │
│  │  trait KeyMaterialPort { fn store_keyslot(...) }         │  │
│  │  trait KeyScopePort { fn current_scope(...) }            │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Adapters (基础设施适配器)                       │
│  ┌───────────────────┐  ┌───────────────────┐  ┌─────────────┐ │
│  │  RustCrypto       │  │  Keyring Adapter  │  │  Database   │ │
│  │  Adapter          │  │                   │  │  Adapter    │ │
│  └───────────────────┘  └───────────────────┘  └─────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Commands Layer Responsibilities

The Commands Layer is responsible for:

1. **Receiving external requests** - Accept Tauri IPC calls from the frontend
2. **Parameter conversion** - Convert JSON/frontend types to domain models
3. **Use case invocation** - Call use cases through the UseCases accessor
4. **Result conversion** - Convert domain models to frontend DTOs
5. **Error handling** - Translate domain errors to frontend-friendly messages

## What Commands Layer Must NOT Do

- ❌ **Directly access Ports** - Never call `runtime.deps.encryption.xxx()` directly
- ❌ **Contain business logic** - All business logic belongs in Use Cases
- ❌ **Operate databases or external systems** - That's what Adapters are for
- ❌ **Know domain rules** - Commands shouldn't understand business constraints

## Mandatory Rules

### Rule 1: Use UseCases Accessor

All commands MUST use the `runtime.usecases()` accessor to get use case instances.

```rust
// ✅ CORRECT - Through UseCases accessor
#[tauri::command]
pub async fn initialize_encryption(
    runtime: State<'_, AppRuntime>,
    passphrase: String,
) -> Result<(), String> {
    let uc = runtime.usecases().initialize_encryption();
    uc.execute(Passphrase(passphrase))
        .await
        .map_err(map_err)?;
    Ok(())
}

// ❌ FORBIDDEN - Direct Port access
#[tauri::command]
pub async fn initialize_encryption(
    runtime: State<'_, AppRuntime>,
    passphrase: String,
) -> Result<(), String> {
    let kek = runtime.deps.encryption
        .derive_kek(&Passphrase(passphrase), &salt, &kdf)
        .await
        .map_err(|e| e.to_string())?;
    // ... more direct port calls
    Ok(())
}
```

### Rule 2: Use map_err for Error Conversion

All commands MUST use the `map_err` utility function for error conversion.

```rust
use crate::commands::map_err;

#[tauri::command]
pub async fn my_command(
    runtime: State<'_, AppRuntime>,
) -> Result<(), String> {
    let uc = runtime.usecases().my_use_case();
    uc.execute()
        .await
        .map_err(map_err)?;  // ← Unified error conversion
    Ok(())
}
```

### Rule 3: Convert Parameters to Domain Models

Frontend parameters MUST be converted to domain models before passing to use cases.

```rust
use uc_core::security::model::Passphrase;

#[tauri::command]
pub async fn initialize_encryption(
    runtime: State<'_, AppRuntime>,
    passphrase: String,  // ← Frontend type
) -> Result<(), String> {
    let uc = runtime.usecases().initialize_encryption();
    uc.execute(Passphrase(passphrase))  // ← Convert to domain model
        .await
        .map_err(map_err)?;
    Ok(())
}
```

### Rule 4: Convert Results to DTOs

Domain models MUST be converted to DTOs before returning to the frontend.

```rust
use crate::models::ClipboardEntryProjection;

#[tauri::command]
pub async fn get_clipboard_entries(
    runtime: State<'_, AppRuntime>,
    limit: Option<usize>,
) -> Result<Vec<ClipboardEntryProjection>, String> {
    let uc = runtime.usecases().list_clipboard_entries();
    let limit = limit.unwrap_or(50);

    // Execute use case, get domain models
    let entries = uc
        .execute(limit, 0)
        .await
        .map_err(map_err)?;

    // Convert to DTOs
    let projections: Vec<ClipboardEntryProjection> = entries
        .into_iter()
        .map(|entry| ClipboardEntryProjection {
            id: entry.entry_id.to_string(),
            preview: entry.title.unwrap_or_else(|| {
                format!("Entry ({} bytes)", entry.total_size)
            }),
            captured_at: entry.created_at_ms,
            content_type: "clipboard".to_string(),
            is_encrypted: false,
        })
        .collect();

    Ok(projections)
}
```

## UseCases Accessor Pattern

The `UseCases` accessor in `uc-tauri/src/bootstrap/runtime.rs` is the factory bridge between Commands and Use Cases layers.

### Design Purpose

1. **Centralized dependency wiring** - All port-to-use-case connections in one place
2. **Decouple Commands** - Commands don't need to know which ports a use case needs
3. **Keep use cases pure** - Use cases don't depend on `AppDeps` or `AppRuntime`

### Implementation Pattern

```rust
// uc-tauri/src/bootstrap/runtime.rs

pub struct UseCases<'a> {
    runtime: &'a AppRuntime,
}

impl<'a> UseCases<'a> {
    pub fn new(runtime: &'a AppRuntime) -> Self {
        Self { runtime }
    }

    // Security use cases
    pub fn initialize_encryption(&self) -> InitializeEncryptionUseCase {
        InitializeEncryptionUseCase::new(
            self.runtime.deps.encryption.clone(),
            self.runtime.deps.key_material.clone(),
            self.runtime.deps.key_scope.clone(),
            self.runtime.deps.encryption_state.clone(),
        )
    }

    // Clipboard use cases
    pub fn list_clipboard_entries(&self) -> ListClipboardEntries {
        ListClipboardEntries::from_arc(
            self.runtime.deps.clipboard_entry_repo.clone()
        )
    }

    // Settings use cases
    pub fn get_settings(&self) -> GetSettingsUseCase {
        GetSettingsUseCase::new(
            self.runtime.deps.settings.clone(),
        )
    }

    pub fn update_settings(&self) -> UpdateSettingsUseCase {
        UpdateSettingsUseCase::new(
            self.runtime.deps.settings.clone(),
        )
    }
}
```

### Two Constructor Patterns

Use cases may have different constructor patterns:

**Pattern 1: Generic with full type parameters**

```rust
// Use case definition
pub struct InitializeEncryption<E, K, KS, ES>
where
    E: EncryptionPort,
    K: KeyMaterialPort,
    KS: KeyScopePort,
    ES: EncryptionStatePort,
{
    encryption: Arc<E>,
    key_material: Arc<K>,
    key_scope: Arc<KS>,
    encryption_state_repo: Arc<ES>,
}

// UseCases accessor method
pub fn initialize_encryption(&self) -> InitializeEncryption<...> {
    InitializeEncryption::new(
        self.runtime.deps.encryption.clone(),
        self.runtime.deps.key_material.clone(),
        self.runtime.deps.key_scope.clone(),
        self.runtime.deps.encryption_state.clone(),
    )
}
```

**Pattern 2: Simplified from_arc constructor**

```rust
// Use case definition
impl ListClipboardEntries {
    pub fn from_arc(repo: Arc<dyn ClipboardEntryRepositoryPort>) -> Self {
        Self { repo }
    }
}

// UseCases accessor method
pub fn list_clipboard_entries(&self) -> ListClipboardEntries {
    ListClipboardEntries::from_arc(
        self.runtime.deps.clipboard_entry_repo.clone()
    )
}
```

### Type Aliases for Complex Use Cases

For use cases with many generic parameters, define type aliases in the use case module:

```rust
// uc-app/src/usecases/mod.rs
pub type InitializeEncryptionUseCase = InitializeEncryption<
    Arc<dyn EncryptionPort>,
    Arc<dyn KeyMaterialPort>,
    Arc<dyn KeyScopePort>,
    Arc<dyn EncryptionStatePort>,
>;

// uc-tauri/src/bootstrap/runtime.rs
pub fn initialize_encryption(&self) -> InitializeEncryptionUseCase {
    InitializeEncryptionUseCase::new(
        self.runtime.deps.encryption.clone(),
        self.runtime.deps.key_material.clone(),
        self.runtime.deps.key_scope.clone(),
        self.runtime.deps.encryption_state.clone(),
    )
}
```

## Complete Command Example

Here's a complete example showing all rules:

```rust
//! Encryption-related Tauri commands
//! 加密相关的 Tauri 命令

use tauri::State;
use uc_core::security::model::Passphrase;
use crate::bootstrap::AppRuntime;
use crate::commands::map_err;

/// Initialize encryption with passphrase
/// 使用密码短语初始化加密
#[tauri::command]
pub async fn initialize_encryption(
    runtime: State<'_, AppRuntime>,
    passphrase: String,
) -> Result<(), String> {
    let uc = runtime.usecases().initialize_encryption();
    uc.execute(Passphrase(passphrase))
        .await
        .map_err(map_err)?;
    Ok(())
}

/// Change passphrase
/// 更改密码短语
#[tauri::command]
pub async fn change_passphrase(
    runtime: State<'_, AppRuntime>,
    old_passphrase: String,
    new_passphrase: String,
) -> Result<(), String> {
    let uc = runtime.usecases().change_passphrase();
    uc.execute(
        Passphrase(old_passphrase),
        Passphrase(new_passphrase),
    )
    .await
    .map_err(map_err)?;
    Ok(())
}
```

## Architecture Compliance Checklist

Every new command MUST satisfy:

- [ ] Uses `runtime.usecases().xxx()` to get use case instance
- [ ] Does NOT directly access `runtime.deps.xxx` ports
- [ ] Uses `map_err` for error conversion
- [ ] Converts frontend parameters to domain models
- [ ] Converts domain results to DTOs (if needed)

## Code Review Guidelines

When reviewing Commands code, check for:

**❌ Forbidden Patterns:**

```rust
// Direct port access
runtime.deps.encryption.derive_kek(...)

// Inline business logic
if state == EncryptionState::Initialized {
    // Multi-line business logic...
}

// Direct error conversion without map_err
.map_err(|e| e.to_string())?
```

**✅ Correct Patterns:**

```rust
// Through use case
runtime.usecases().initialize_encryption()

// Using map_err
.map_err(map_err)?
```

## Adding New Commands Workflow

```
1. Create use case in uc-app/src/usecases/
2. Add accessor method in uc-tauri/src/bootstrap/runtime.rs
3. Create command function in commands/
4. Code review for architecture compliance
5. Run tests to verify functionality
```

## Current Status

| Command                  | File                                                                                      | Status             | Use Case Exists | Needs Refactor |
| ------------------------ | ----------------------------------------------------------------------------------------- | ------------------ | --------------- | -------------- |
| `get_clipboard_entries`  | [clipboard.rs:12-40](../../src-tauri/crates/uc-tauri/src/commands/clipboard.rs#L12-L40)   | ✅ Refactored      | ✅              | No             |
| `delete_clipboard_entry` | [clipboard.rs:42-79](../../src-tauri/crates/uc-tauri/src/commands/clipboard.rs#L42-L79)   | 📝 Documented TODO | ❌              | **TODO**       |
| `capture_clipboard`      | [clipboard.rs:81-142](../../src-tauri/crates/uc-tauri/src/commands/clipboard.rs#L81-L142) | 📝 Documented TODO | ❌              | **TODO**       |
| `initialize_encryption`  | [encryption.rs:22-31](../../src-tauri/crates/uc-tauri/src/commands/encryption.rs#L22-L31) | ✅ Refactored      | ✅              | No             |
| `get_settings`           | [settings.rs:37-49](../../src-tauri/crates/uc-tauri/src/commands/settings.rs#L37-L49)     | 📝 Documented TODO | ❌              | **TODO**       |
| `update_settings`        | [settings.rs:81-94](../../src-tauri/crates/uc-tauri/src/commands/settings.rs#L81-L94)     | 📝 Documented TODO | ❌              | **TODO**       |

### Legend / 图例

- ✅ **Refactored** - Fully compliant with Commands Layer Specification
- 📝 **Documented TODO** - Has comprehensive TODO comments with implementation requirements
- ❌ **Direct ports** - Architecture violation (needs refactoring)

## Progress Summary / 进度总结

### Completed / 已完成

- ✅ `initialize_encryption` - Refactored to use UseCases accessor pattern
- ✅ `get_clipboard_entries` - Uses ListClipboardEntries use case
- ✅ All pending commands have comprehensive TODO documentation

### In Progress / 进行中

- 📝 `delete_clipboard_entry` - TODO documented, awaiting DeleteClipboardEntry use case
- 📝 `capture_clipboard` - TODO documented, awaiting CaptureClipboard use case (complex multi-port)
- 📝 `get_settings` - TODO documented, awaiting GetSettings use case
- 📝 `update_settings` - TODO documented, awaiting UpdateSettings use case

## TODO: Missing Use Cases

The following commands require use case implementation (separate task):

- ⏳ `DeleteClipboardEntry` - Delete clipboard entry
  - Port: ClipboardEntryRepositoryPort (already exists)
  - Complexity: Low (simple delete)

- ⏳ `CaptureClipboard` - Complete capture flow
  - Ports: ClipboardSnapshotPort, MaterializationPort, ClipboardEventWriterPort, ClipboardEntryRepositoryPort
  - Complexity: High (multi-port orchestration)

- ⏳ `GetSettings` / `UpdateSettings` - Settings management
  - Port: SettingsPort (needs definition)
  - Complexity: Medium (requires port definition)

## Further Reading

- [Architecture Principles](principles.md) - Hexagonal architecture fundamentals
- [Coding Standards](../standards/coding-standards.md) - General coding standards
- [Error Handling](../guides/error-handling.md) - Error handling strategy
