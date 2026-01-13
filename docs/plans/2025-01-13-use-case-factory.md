# Use Case Factory Implementation Plan (Revised)

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create a UseCases accessor on AppRuntime that manages instantiation of all use cases, ensuring commands don't implement business logic directly while maintaining clean hexagonal architecture boundaries.

**Architecture:**
- **uc-app/usecases**: Pure use cases with `new()`/`from_arc()` constructors, no dependency on AppDeps
- **uc-tauri/bootstrap**: UseCases accessor that wires `Arc<dyn Port>` from AppDeps into use case instances
- **Commands**: Call `runtime.usecases().xxx()`, handle only parameter parsing and error mapping

**Tech Stack:** Rust, Tauri 2, Hexagonal Architecture (Ports & Adapters)

---

## Background & Context

**Current Problem:**
1. `initialize_encryption` command directly implements 8-step business logic instead of using the `InitializeEncryption` use case
2. Commands hold `Arc<dyn Port>` via AppDeps, but use cases use generic types `UseCase<P1, P2, ...>`
3. Previous solution of adding `from_deps(&AppDeps)` to use cases breaks architectural boundaries

**Why UseCases on AppRuntime (not in uc-app)?**
- ✅ Use cases stay pure - only depend on ports via constructors
- ✅ AppDeps stays in composition root (uc-tauri), doesn't leak into uc-app
- ✅ Wiring logic is centralized but separated from use case definitions
- ✅ Commands get clean syntax: `runtime.usecases().xxx()`
- ✅ Testing use cases doesn't require constructing huge AppDeps

**Architecture Boundary Rule:**
> If you see `from_deps(&AppDeps)` or `UseCaseFactory` directly referencing AppDeps in `uc-app/usecases/*`, wiring has leaked into the use case layer.

---

## Task 1: Create UseCases Module in uc-tauri

**Files:**
- Create: `src-tauri/crates/uc-tauri/src/bootstrap/usecases.rs`

**Step 1: Create the UseCases struct**

```rust
//! Use cases accessor for AppRuntime
//! AppRuntime 的用例访问器

use crate::runtime::{AppDeps};
use std::sync::Arc;
use uc_app::usecases::*;
use uc_core::ports::*;

/// Use cases accessor attached to AppRuntime
/// 附加到 AppRuntime 的用例访问器
pub struct UseCases<'a> {
    deps: &'a AppDeps,
}

impl<'a> UseCases<'a> {
    /// Security use cases / 安全用例
    pub fn initialize_encryption(&self) -> InitializeEncryption<
        Arc<dyn EncryptionPort>,
        Arc<dyn KeyMaterialPort>,
        Arc<dyn KeyScopePort>,
        Arc<dyn EncryptionStatePort>,
    > {
        InitializeEncryption::new(
            self.deps.encryption.clone(),
            self.deps.key_material.clone(),
            self.deps.key_scope.clone(),
            self.deps.encryption_state.clone(),
        )
    }

    /// Clipboard use cases / 剪贴板用例
    pub fn list_clipboard_entries(&self) -> ListClipboardEntries {
        ListClipboardEntries::from_arc(self.deps.clipboard_entry_repo.clone())
    }

    /// Settings use cases / 设置用例
    pub fn apply_autostart(&self) -> ApplyAutostartSetting<
        Arc<dyn SettingsPort>,
        Arc<dyn AutostartPort>,
    > {
        ApplyAutostartSetting::new(
            self.deps.settings.clone(),
            self.deps.autostart.clone(),
        )
    }

    pub fn apply_theme(&self) -> ApplyThemeSetting<Arc<dyn SettingsPort>> {
        ApplyThemeSetting::new(self.deps.settings.clone())
    }
}
```

**Step 2: Add to bootstrap module**

Edit `src-tauri/crates/uc-tauri/src/bootstrap/mod.rs`:

```rust
pub mod config;
pub mod runtime;
pub mod usecases;  // Add this line
pub mod wiring;
```

**Step 3: Run cargo check**

```bash
cd src-tauri
cargo check -p uc-tauri
```

Expected: May have errors about missing `new()` methods on use cases - we'll fix those in next tasks

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/usecases.rs
git add src-tauri/crates/uc-tauri/src/bootstrap/mod.rs
git commit -m "feat(uc-tauri): add UseCases accessor struct"
```

---

## Task 2: Add usecases() Method to AppRuntime

**Files:**
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/usecases.rs`

**Step 1: Update UseCases to work with AppRuntime**

Edit `src-tauri/crates/uc-tauri/src/bootstrap/usecases.rs`:

```rust
use crate::runtime::AppRuntime;

impl AppRuntime {
    /// Get use cases accessor
    /// 获取用例访问器
    pub fn usecases(&self) -> UseCases<'_> {
        UseCases { deps: &self.deps }
    }
}
```

**Step 2: Ensure AppRuntime exposes deps**

Check `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`:

```rust
use uc_app::AppDeps;

pub struct AppRuntime {
    pub deps: AppDeps,
}

impl AppRuntime {
    pub fn new(deps: AppDeps) -> Self {
        Self { deps }
    }
}
```

**Step 3: Run cargo check**

```bash
cd src-tauri
cargo check -p uc-tauri
```

Expected: PASS

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/
git commit -m "feat(uc-tauri): add usecases() method to AppRuntime"
```

---

## Task 3: Ensure Use Cases Have Proper Constructors

**Files:**
- Verify: `src-tauri/crates/uc-app/src/usecases/initialize_encryption.rs`
- Verify: `src-tauri/crates/uc-app/src/usecases/settings/apply_autostart.rs`
- Verify: `src-tauri/crates/uc-app/src/usecases/settings/apply_theme.rs`

**Step 1: Verify InitializeEncryption has new()**

Check that `uc-app/src/usecases/initialize_encryption.rs` has:

```rust
impl<E, K, KS, ES> InitializeEncryption<E, K, KS, ES>
where
    E: EncryptionPort,
    K: KeyMaterialPort,
    KS: KeyScopePort,
    ES: EncryptionStatePort,
{
    pub fn new(
        encryption: Arc<E>,
        key_material: Arc<K>,
        key_scope: Arc<KS>,
        encryption_state_repo: Arc<ES>,
    ) -> Self {
        Self {
            encryption,
            key_material,
            key_scope,
            encryption_state_repo,
        }
    }
}
```

If missing, add it.

**Step 2: Verify ApplyAutostartSetting has new()**

Check `uc-app/src/usecases/settings/apply_autostart.rs`:

```rust
impl<S, A> ApplyAutostartSetting<S, A>
where
    S: SettingsPort,
    A: AutostartPort,
{
    pub fn new(settings: Arc<S>, autostart: Arc<A>) -> Self {
        Self { settings, autostart }
    }
}
```

If missing, add it.

**Step 3: Verify ApplyThemeSetting has new()**

Check `uc-app/src/usecases/settings/apply_theme.rs`:

```rust
impl<S> ApplyThemeSetting<S>
where
    S: SettingsPort,
{
    pub fn new(settings: Arc<S>) -> Self {
        Self { settings }
    }
}
```

If missing, add it.

**Step 4: Run tests**

```bash
cd src-tauri
cargo test -p uc-app
```

Expected: PASS

**Step 5: Commit if any changes were made**

```bash
git add src-tauri/crates/uc-app/src/usecases/
git commit -m "chore(uc-app): ensure use cases have new() constructors"
```

---

## Task 4: Add Missing Use Cases to UseCases Accessor

**Files:**
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/usecases.rs`

**Step 1: Add restore_clipboard_selection()**

```rust
use uc_app::usecases::clipboard::RestoreClipboardSelectionUseCase;

impl<'a> UseCases<'a> {
    // ... existing methods ...

    pub fn restore_clipboard_selection(&self) -> RestoreClipboardSelectionUseCase<
        Arc<dyn ClipboardEntryRepositoryPort>,
        Arc<dyn SystemClipboardPort>,
        Arc<dyn ClipboardSelectionRepositoryPort>,
        Arc<dyn ClipboardRepresentationRepositoryPort>,
        Arc<dyn BlobStorePort>,
    > {
        RestoreClipboardSelectionUseCase::new(
            self.deps.clipboard_entry_repo.clone(),
            self.deps.clipboard.clone(),
            self.deps.selection_repo.clone(),
            self.deps.representation_repo.clone(),
            self.deps.blob_store.clone(),
        )
    }
}
```

**Step 2: Add capture_clipboard()**

```rust
use uc_app::usecases::internal::CaptureClipboardUseCase;

impl<'a> UseCases<'a> {
    // ... existing methods ...

    pub fn capture_clipboard(&self) -> CaptureClipboardUseCase<
        Arc<dyn PlatformClipboardPort>,
        Arc<dyn ClipboardEntryRepositoryPort>,
        Arc<dyn ClipboardEventWriterPort>,
        Arc<dyn SelectRepresentationPolicyPort>,
        Arc<dyn ClipboardRepresentationMaterializerPort>,
        Arc<dyn DeviceIdentityPort>,
    > {
        CaptureClipboardUseCase::new(
            self.deps.clipboard.clone(),
            self.deps.clipboard_entry_repo.clone(),
            self.deps.clipboard_event_repo.clone(),
            self.deps.representation_policy.clone(),
            self.deps.representation_materializer.clone(),
            self.deps.device_identity.clone(),
        )
    }
}
```

**Step 3: Run cargo check**

```bash
cd src-tauri
cargo check -p uc-tauri
```

Expected: May need to fix import paths or type parameter names

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/usecases.rs
git commit -m "feat(uc-tauri): add missing use cases to UseCases accessor"
```

---

## Task 5: Refactor initialize_encryption Command

**Files:**
- Modify: `src-tauri/crates/uc-tauri/src/commands/encryption.rs`

**Step 1: Replace direct implementation with use case**

Edit `src-tauri/crates/uc-tauri/src/commands/encryption.rs`:

```rust
use crate::bootstrap::runtime::AppRuntime;
use uc_core::security::model::Passphrase;
use tauri::State;

#[tauri::command]
pub async fn initialize_encryption(
    runtime: State<'_, AppRuntime>,
    passphrase: String,
) -> Result<(), String> {
    // 1. Parse parameter
    let passphrase = Passphrase(passphrase);

    // 2. Get use case and execute
    let uc = runtime.usecases().initialize_encryption();
    uc.execute(passphrase)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
```

**Step 2: Delete the old implementation (lines 26-79)**

Remove all the manual 8-step logic that was previously in this function.

**Step 3: Update is_encryption_initialized to use pattern**

```rust
#[tauri::command]
pub async fn is_encryption_initialized(
    runtime: State<'_, AppRuntime>,
) -> Result<bool, String> {
    let state = runtime.deps.encryption_state
        .load_state()
        .await
        .map_err(|e| e.to_string())?;

    Ok(state == uc_core::security::state::EncryptionState::Initialized)
}
```

**Step 4: Run tests**

```bash
cd src-tauri
cargo test -p uc-tauri
cargo check -p uc-tauri
```

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/encryption.rs
git commit -m "refactor(uc-tauri): use InitializeEncryption use case in command"
```

---

## Task 6: Update clipboard Commands

**Files:**
- Modify: `src-tauri/crates/uc-tauri/src/commands/clipboard.rs`

**Step 1: Update get_clipboard_entries**

```rust
use crate::bootstrap::runtime::AppRuntime;
use crate::models::ClipboardEntryProjection;
use tauri::State;

#[tauri::command]
pub async fn get_clipboard_entries(
    runtime: State<'_, AppRuntime>,
    limit: Option<usize>,
) -> Result<Vec<ClipboardEntryProjection>, String> {
    let factory = runtime.usecases();
    let use_case = factory.list_clipboard_entries();
    let limit = limit.unwrap_or(50);

    let entries = use_case
        .execute(limit, 0)
        .await
        .map_err(|e| e.to_string())?;

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

**Step 2: Update delete_clipboard_entry (placeholder)**

```rust
#[tauri::command]
pub async fn delete_clipboard_entry(
    _runtime: State<'_, AppRuntime>,
    _entry_id: String,
) -> Result<(), String> {
    // TODO: Implement DeleteClipboardEntry use case
    // Required ports: ClipboardEntryRepositoryPort
    // Once implemented, use: runtime.usecases().delete_clipboard_entry()
    Err("Not yet implemented".to_string())
}
```

**Step 3: Update capture_clipboard (placeholder)**

```rust
#[tauri::command]
pub async fn capture_clipboard(
    _runtime: State<'_, AppRuntime>,
) -> Result<(), String> {
    // TODO: Use CaptureClipboardUseCase via runtime.usecases().capture_clipboard()
    Err("Not yet implemented".to_string())
}
```

**Step 4: Run tests**

```bash
cd src-tauri
cargo test -p uc-tauri
```

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/clipboard.rs
git commit -m "refactor(uc-tauri): update clipboard commands to use UseCases"
```

---

## Task 7: Update Tauri Builder to Register AppRuntime

**Files:**
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/run.rs` (or main lib.rs)

**Step 1: Ensure AppRuntime is registered with Tauri**

Check the Tauri builder setup:

```rust
use tauri::Builder;

fn build_app() -> anyhow::Result<()> {
    // ... config loading ...

    // Wire dependencies
    let deps = wire_dependencies(&config)?;

    // Create runtime
    let runtime = AppRuntime::new(deps);

    Builder::default()
        .manage(runtime)  // Register AppRuntime, not AppDeps
        .invoke_handler(tauri::generate_handler![
            initialize_encryption,
            is_encryption_initialized,
            get_clipboard_entries,
            // ... other commands ...
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
```

**Step 2: Update command signatures**

Ensure all commands use `State<'_, AppRuntime>` instead of `State<'_, AppDeps>`.

**Step 3: Run full build**

```bash
cd src-tauri
cargo build
```

Expected: SUCCESS

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/run.rs
git commit -m "refactor(uc-tauri): register AppRuntime with Tauri"
```

---

## Task 8: Add Documentation

**Files:**
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/usecases.rs`

**Step 1: Add module documentation**

```rust
//! # Use Cases Accessor
//!
//! This module provides the `UseCases` accessor which is attached to `AppRuntime`
//! to provide convenient access to all use cases with their dependencies pre-wired.
//!
//! ## Architecture
//!
//! - **uc-app/usecases**: Pure use cases with `new()` constructors taking ports
//! - **uc-tauri/bootstrap**: This module wires `Arc<dyn Port>` from AppDeps into use cases
//! - **Commands**: Call `runtime.usecases().xxx()` to get use case instances
//!
//! ## Usage
//!
//! ```rust,no_run
//! use uc_tauri::bootstrap::AppRuntime;
//! use tauri::State;
//!
//! #[tauri::command]
//! async fn my_command(runtime: State<'_, AppRuntime>) -> Result<(), String> {
//!     let uc = runtime.usecases().initialize_encryption();
//!     uc.execute(passphrase).await.map_err(|e| e.to_string())?;
//!     Ok(())
//! }
//! ```
//!
//! ## Adding New Use Cases
//!
//! 1. Ensure use case has a `new()` constructor taking its required ports
//! 2. Add a method to `UseCases` that calls `new()` with deps
//! 3. Commands can now call `runtime.usecases().your_use_case()`
```

**Step 2: Add ARCHITECTURE.md note**

Create `docs/architecture/usecase-accessor-pattern.md`:

```markdown
# Use Case Accessor Pattern

## Pattern Overview

Instead of having use cases depend on `AppDeps` or having commands wire dependencies directly, we use an accessor pattern:

```
Commands → AppRuntime.usecases() → UseCases → new(ports) → UseCase
```

## Benefits

1. **Clean boundaries**: Use cases only depend on ports via constructors
2. **Centralized wiring**: All port→usecase wiring in one place (UseCases)
3. **Testable**: Use cases can be tested independently without AppDeps
4. **Clean commands**: Commands get clean syntax: `runtime.usecases().xxx()`

## Adding a New Use Case

1. **Define use case** in `uc-app/usecases/`:

```rust
pub struct MyUseCase<P1: Port1, P2: Port2> {
    port1: Arc<P1>,
    port2: Arc<P2>,
}

impl<P1: Port1, P2: Port2> MyUseCase<P1, P2> {
    pub fn new(port1: Arc<P1>, port2: Arc<P2>) -> Self {
        Self { port1, port2 }
    }
}
```

2. **Add accessor method** in `uc-tauri/bootstrap/usecases.rs`:

```rust
impl<'a> UseCases<'a> {
    pub fn my_use_case(&self) -> MyUseCase<Arc<dyn Port1>, Arc<dyn Port2>> {
        MyUseCase::new(
            self.deps.port1.clone(),
            self.deps.port2.clone(),
        )
    }
}
```

3. **Use in command**:

```rust
#[tauri::command]
async fn my_command(runtime: State<'_, AppRuntime>) -> Result<(), String> {
    let uc = runtime.usecases().my_use_case();
    uc.execute(...).await.map_err(|e| e.to_string())?;
    Ok(())
}
```

## Anti-Patterns to Avoid

❌ **Don't** add `from_deps(&AppDeps)` to use cases in uc-app
❌ **Don't** make use cases depend on AppDeps directly
❌ **Don't** wire ports in commands
```

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/usecases.rs
git add docs/architecture/usecase-accessor-pattern.md
git commit -m "docs: add Use Cases accessor documentation"
```

---

## Task 9: Add Tests

**Files:**
- Create: `src-tauri/crates/uc-tauri/tests/usecases_accessor_test.rs`

**Step 1: Write integration test**

```rust
use uc_tauri::bootstrap::{AppRuntime, AppDeps, UseCases};
use uc_core::ports::*;

// This test verifies UseCases methods are callable
// Actual behavior testing is in uc-app use case tests

#[test]
fn test_use_cases_has_all_methods() {
    // Compile-time verification that all methods exist
    let _ = |use_cases: &UseCases| {
        let _ = use_cases.initialize_encryption;
        let _ = use_cases.list_clipboard_entries;
        let _ = use_cases.restore_clipboard_selection;
        let _ = use_cases.capture_clipboard;
        let _ = use_cases.apply_autostart;
        let _ = use_cases.apply_theme;
    };
}

#[test]
fn test_app_runtime_has_usecases_method() {
    // Compile-time verification
    let _ = |runtime: &AppRuntime| {
        let _ = runtime.usecases();
    };
}
```

**Step 2: Run tests**

```bash
cd src-tauri
cargo test -p uc-tauri usecases_accessor
```

Expected: PASS

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-tauri/tests/
git commit -m "test(uc-tauri): add UseCases accessor tests"
```

---

## Task 10: Final Verification

**Step 1: Run full test suite**

```bash
cd src-tauri
cargo test
cargo clippy -p uc-app -p uc-tauri
```

Expected: All tests pass, no warnings

**Step 2: Build release**

```bash
cd src-tauri
cargo build --release
```

Expected: SUCCESS

**Step 3: Manual testing (if dev environment available)**

```bash
cd src-tauri
bun tauri dev
```

Test the initialize_encryption command from the frontend

**Step 4: Final commit**

```bash
git add .
git commit -m "feat(uc-tauri): complete UseCases accessor implementation

- Add UseCases accessor on AppRuntime
- Refactor commands to use runtime.usecases() pattern
- Remove business logic from command layer
- Add comprehensive documentation

Architecture improvements:
- Use cases no longer depend on AppDeps
- Wiring centralized in composition root (uc-tauri)
- Commands handle only parameter parsing and error mapping
- Clean hexagonal architecture boundaries maintained"
```

---

## Success Criteria

1. ✅ Use cases have `new()` constructors, no `from_deps()`
2. ✅ `UseCases` accessor in uc-tauri (not in uc-app)
3. ✅ Commands use `runtime.usecases().xxx()` pattern
4. ✅ No business logic in commands
5. ✅ `initialize_encryption` command refactored
6. ✅ All tests pass
7. ✅ Documentation complete

## Architecture Verification

**Check these don't exist:**
- ❌ `from_deps(&AppDeps)` in uc-app/usecases/*
- ❌ `UseCaseFactory` directly in uc-app/usecases/*
- ❌ Use cases importing AppDeps

**Check these do exist:**
- ✅ `UseCases` in uc-tauri/bootstrap/usecases.rs
- ✅ `runtime.usecases()` method on AppRuntime
- ✅ Commands using `State<'_, AppRuntime>`

## Rollback Plan

If issues arise:
1. Commands can still call use case constructors directly
2. UseCases accessor is an additional convenience, not a hard dependency
3. Revert to previous commit if integration issues block progress
