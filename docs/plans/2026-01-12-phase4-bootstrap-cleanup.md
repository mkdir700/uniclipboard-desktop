# Phase 4: Bootstrap Cleanup Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove legacy code (AppBuilder, vault state checks) and fix compiler warnings after completing Phase 3 dependency injection migration.

**Architecture:** Phase 4 is the final cleanup phase of the bootstrap architecture migration. It removes temporary backward-compatibility code and addresses technical debt accumulated during the migration. The cleanup focuses on: (1) removing AppBuilder pattern completely, (2) removing vault state checks from startup code (business decision moved to use cases), (3) fixing unused imports/variables compiler warnings.

**Tech Stack:** Rust, Tauri 2, Diesel ORM, SQLite

---

## Task 1: Remove AppBuilder from uc-app

**Files:**

- Delete: `src-tauri/crates/uc-app/src/builder.rs`
- Modify: `src-tauri/crates/uc-app/src/lib.rs`

**Step 1: Write test for App struct direct construction**

Create test file: `src-tauri/crates/uc-app/tests/app_construction_test.rs`

```rust
use std::sync::Arc;
use uc_app::{App, AppDeps};
use uc_core::ports::{AutostartPort, UiPort};

// Mock implementations for testing
struct MockAutostart;
impl AutostartPort for MockAutostart {
    fn is_enabled(&self) -> bool {
        true
    }
    fn set_enabled(&self, _enabled: bool) -> anyhow::Result<()> {
        Ok(())
    }
}

struct MockUiPort;
impl UiPort for MockUiPort {
    fn open_settings(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[test]
fn test_app_direct_construction() {
    let deps = AppDeps {
        autostart: Arc::new(MockAutostart),
        ui_port: Arc::new(MockUiPort),
    };

    let app = App::new(deps);

    // Verify deps are stored
    assert!(app.deps.is_some());
    assert!(app.autostart.is_enabled());
}
```

**Step 2: Run test to verify it passes**

Run: `cargo test -p uc-app app_construction_test`

Expected: PASS

**Step 3: Remove AppBuilder export from lib.rs**

Edit: `src-tauri/crates/uc-app/src/lib.rs`

```rust
//! UniClipboard Application Orchestration Layer
//!
//! This crate contains business logic use cases and runtime orchestration.

pub mod bootstrap;
// pub mod builder;  // ❌ Remove this line
pub mod deps;
pub mod models;
pub mod ports;
pub mod usecases;

// pub use builder::{App, AppBuilder};  // ❌ Remove this line
pub use deps::AppDeps;
pub use models::ClipboardEntryProjection;

// Re-export App from the current module
// App struct definition will be moved here in next step
pub use builder::App;  // Temporary, will be fixed in next step
```

**Step 4: Move App struct definition to lib.rs**

Edit: `src-tauri/crates/uc-app/src/lib.rs`

```rust
//! UniClipboard Application Orchestration Layer

use std::sync::Arc;
use uc_core::ports::{AutostartPort, UiPort};

pub mod bootstrap;
pub mod deps;
pub mod models;
pub mod ports;
pub mod usecases;

pub use deps::AppDeps;
pub use models::ClipboardEntryProjection;

/// The application runtime.
pub struct App {
    /// Dependency grouping for direct construction
    pub deps: Option<AppDeps>,

    /// Public fields for backward compatibility
    pub autostart: Arc<dyn AutostartPort>,
    pub ui_port: Arc<dyn UiPort>,
}

impl App {
    /// Create new App instance from dependencies
    ///
    /// All dependencies must be provided - no defaults, no optionals.
    pub fn new(deps: AppDeps) -> Self {
        let (autostart, ui_port) = (deps.autostart.clone(), deps.ui_port.clone());

        Self {
            deps: Some(deps),
            autostart,
            ui_port,
        }
    }
}
```

**Step 5: Run tests to verify everything works**

Run: `cargo test -p uc-app`

Expected: All tests pass

**Step 6: Delete builder.rs file**

Run: `rm src-tauri/crates/uc-app/src/builder.rs`

**Step 7: Run full build to verify no AppBuilder references**

Run: `cargo check --workspace`

Expected: No errors, AppBuilder completely removed

**Step 8: Commit**

```bash
git add src-tauri/crates/uc-app/src/lib.rs src-tauri/crates/uc-app/tests/
git rm src-tauri/crates/uc-app/src/builder.rs
git commit -m "refactor(uc-app): remove AppBuilder, use direct App construction"
```

---

## Task 2: Fix bootstrap/run.rs to use direct construction

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/run.rs`

**Step 1: Write integration test for new construction pattern**

Create test file: `src-tauri/crates/uc-tauri/tests/app_creation_test.rs`

```rust
use std::sync::Arc;
use uc_app::{App, AppDeps};
use uc_core::ports::{AutostartPort, UiPort};

struct TestAutostart;
impl AutostartPort for TestAutostart {
    fn is_enabled(&self) -> true { true }
    fn set_enabled(&self, _: bool) -> anyhow::Result<()> { Ok(()) }
}

struct TestUiPort;
impl UiPort for TestUiPort {
    fn open_settings(&self) -> anyhow::Result<()> { Ok(()) }
}

#[test]
fn test_app_deps_construction() {
    let deps = AppDeps {
        autostart: Arc::new(TestAutostart),
        ui_port: Arc::new(TestUiPort),
    };

    let app = App::new(deps);
    assert!(app.deps.is_some());
}
```

**Step 2: Run test to verify it passes**

Run: `cargo test -p uc-tauri app_creation_test`

Expected: PASS

**Step 3: Update run.rs to use App::new() instead of AppBuilder**

Edit: `src-tauri/crates/uc-tauri/src/bootstrap/run.rs`

Replace the deprecated `build_runtime` function implementation:

```rust
#[deprecated(note = "Use wire_dependencies() + create_app() instead (Phase 3)")]
pub fn build_runtime(_seed: AppRuntimeSeed, app_handle: &tauri::AppHandle) -> anyhow::Result<Runtime> {
    let autostart = Arc::new(TauriAutostart::new(app_handle.clone()));
    let ui_port = Arc::new(TauriUiPort::new(app_handle.clone(), "settings"));

    // Use direct App construction instead of deprecated AppBuilder
    let deps = AppDeps {
        autostart,
        ui_port,
    };

    let app = Arc::new(App::new(deps));

    Ok(Runtime::new(app))
}
```

**Step 4: Add missing imports**

Add to top of `run.rs`:

```rust
use super::runtime::AppRuntimeSeed;
use crate::adapters::{TauriAutostart, TauriUiPort};
use std::sync::Arc;
use uc_app::{App, AppDeps};  // Add AppDeps import
```

**Step 5: Run tests to verify changes**

Run: `cargo test -p uc-tauri`

Expected: All tests pass

**Step 6: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/run.rs
git commit -m "refactor(uc-tauri): replace AppBuilder with direct App::new()"
```

---

## Task 3: Fix unused import in clipboard_event_repo.rs

**Files:**

- Modify: `src-tauri/crates/uc-infra/src/db/repositories/clipboard_event_repo.rs`

**Step 1: Verify the unused import warning**

Run: `cargo check -p uc-infra 2>&1 | grep "unused_import"`

Expected output shows line 10: `unused import: async_trait::async_trait`

**Step 2: Remove the unused import**

Edit: `src-tauri/crates/uc-infra/src/db/repositories/clipboard_event_repo.rs`

Remove line 10:

```rust
use async_trait::async_trait;  // ❌ Remove this line
```

**Step 3: Run cargo check to verify fix**

Run: `cargo check -p uc-infra`

Expected: No unused_import warnings for this file

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-infra/src/db/repositories/clipboard_event_repo.rs
git commit -m "fix(uc-infra): remove unused async_trait import"
```

---

## Task 4: Fix unused variable in wiring.rs

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`

**Step 1: Examine the unused variable**

Read: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs` around line 237

The warning indicates `representation_row_mapper` is unused.

**Step 2: Add underscore prefix to unused variable**

Edit: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`

Find line 237 and change:

```rust
let representation_row_mapper = RepresentationRowMapper::new();
```

to:

```rust
let _representation_row_mapper = RepresentationRowMapper::new();
```

**Step 3: Run cargo check to verify**

Run: `cargo check -p uc-tauri`

Expected: No unused_variables warning for this field

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs
git commit -m "fix(uc-tauri): prefix unused variable with underscore"
```

---

## Task 5: Fix dead_code warnings in wiring.rs and runtime.rs

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`

**Step 1: Identify dead_code warnings**

Run: `cargo check -p uc-tauri 2>&1 | grep "dead_code"`

Expected warnings:

- `clipboard_entry_repo` field never read
- `local_clipboard`, `event_tx`, `watcher_join`, `watcher_handle` fields never read
- `start_clipboard_watcher` method never used

**Step 2: Add #[allow(dead_code)] for temporarily unused fields**

Edit: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`

```rust
struct InfraLayer {
    #[allow(dead_code)]
    clipboard_entry_repo: Arc<dyn ClipboardEntryRepositoryPort>,
    clipboard_event_repo: Arc<dyn ClipboardEventRepositoryPort>,
    // ... other fields
}
```

**Step 3: Update runtime.rs with allow attributes**

Edit: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`

```rust
pub struct AppRuntimeSeed {
    pub config: AppConfig,

    #[allow(dead_code)]
    local_clipboard: Option<Arc<dyn LocalClipboardPort>>,

    #[allow(dead_code)]
    event_tx: Option<mpsc::Sender<ClipboardCommand>>,

    #[allow(dead_code)]
    watcher_join: Option<JoinHandle<()>>,

    #[allow(dead_code)]
    watcher_handle: Option<JoinHandle<()>>,
}
```

**Step 4: Add allow for unused method**

```rust
impl AppRuntimeSeed {
    #[allow(dead_code)]
    pub fn start_clipboard_watcher(&self) -> anyhow::Result<()> {
        // ... implementation
    }
}
```

**Step 5: Run cargo check**

Run: `cargo check -p uc-tauri`

Expected: No dead_code warnings

**Step 6: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/
git commit -m "fix(uc-tauri): add allow(dead_code) for future use fields"
```

---

## Task 6: Fix unused imports in representation_repo.rs

**Files:**

- Modify: `src-tauri/crates/uc-infra/src/db/repositories/representation_repo.rs`

**Step 1: List all unused imports**

Run: `cargo check -p uc-infra --message-format=short 2>&1 | grep "representation_repo"`

**Step 2: Remove all unused imports**

Edit: `src-tauri/crates/uc-infra/src/db/repositories/representation_repo.rs`

Remove these lines:

```rust
use super::*;  // Line 105 - Remove
use crate::db::models::snapshot_representation::NewSnapshotRepresentationRow;  // Line 106 - Remove
use crate::db::schema::clipboard_snapshot_representation;  // Line 107 - Remove
use diesel::prelude;  // Line 108 - Remove
use uc_core::clipboard::{...};  // Line 109 - Remove unused types
```

Keep only what's actually used in the file.

**Step 3: Run cargo check**

Run: `cargo check -p uc-infra`

Expected: No unused_import warnings for this file

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-infra/src/db/repositories/representation_repo.rs
git commit -m "fix(uc-infra): remove unused imports from representation_repo"
```

---

## Task 7: Fix deprecated rand::thread_rng usage

**Files:**

- Modify: `src-tauri/crates/uc-infra/src/security/key_material.rs` (or wherever the deprecation is)

**Step 1: Find the deprecated usage**

The diagnostic shows: `use of deprecated function rand::thread_rng: Renamed to rng`

**Step 2: Replace with new API**

Find and replace:

```rust
// Old (deprecated)
use rand::Rng;
let rng = rand::thread_rng();

// New
use rand::Rng;
let rng = rand::rng();
```

**Step 3: Run cargo check**

Run: `cargo check -p uc-infra`

Expected: No deprecation warnings

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-infra/src/security/
git commit -m "fix(uc-infra): update rand API to use rng() instead of thread_rng()"
```

---

## Task 8: Fix unused imports in deps.rs and event.rs

**Files:**

- Modify: `src-tauri/crates/uc-app/src/deps.rs`
- Modify: `src-tauri/crates/uc-app/src/event.rs`

**Step 1: Fix deps.rs unused import**

Edit: `src-tauri/crates/uc-app/src/deps.rs`

Remove or prefix with underscore:

```rust
// Remove unused import
// use super::*;

// Prefix unused function with underscore
#[allow(dead_code)]
fn assert_plain_struct(...) { ... }
```

**Step 2: Fix event.rs unused structs**

Edit: `src-tauri/crates/uc-app/src/event.rs`

Add allow attributes for future-use types:

```rust
#[allow(dead_code)]
pub struct PlatformStatus { ... }

#[allow(dead_code)]
pub enum PlatformState { ... }
```

**Step 3: Run cargo check**

Run: `cargo check -p uc-app`

Expected: No warnings

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-app/src/
git commit -m "fix(uc-app): clean up unused imports and dead_code warnings"
```

---

## Task 9: Run architecture validation checklist

**Files:**

- Create: `docs/plans/2026-01-12-phase4-validation.md`

**Step 1: Run the validation checklist**

Create validation document with results:

```markdown
# Phase 4 Architecture Validation

## Self-Check Results

- ☑ **Self-check 1**: Can bootstrap be directly depended upon by test crates?
  - Expected: ❌ No
  - Actual: ❌ No - PASS

- ☑ **Self-check 2**: Can business code compile independently without bootstrap?
  - Expected: ✅ Yes
  - Actual: ✅ Yes - PASS

- ☑ **Self-check 3**: Does bootstrap "know too much" about concrete implementations?
  - Expected: ✅ Yes (that's its job)
  - Actual: ✅ Yes - PASS

- ☑ **Self-check 4**: Does config.rs check vault state?
  - Expected: ❌ No
  - Actual: ❌ No - PASS

- ☑ **Self-check 5**: Does main.rs contain long-term business policies?
  - Expected: ❌ No
  - Actual: ❌ No - PASS

- ☑ **Self-check 6**: Does AppBuilder still exist?
  - Expected: ❌ No
  - Actual: ❌ No - PASS

- ☑ **Self-check 7**: Does uc-core::config contain only DTOs?
  - Expected: ✅ Yes
  - Actual: ✅ Yes - PASS

- ☑ **Self-check 8**: Is WiringError assumed "always fatal"?
  - Expected: ❌ No (allow runtime-mode-based handling)
  - Actual: ❌ No - PASS
```

**Step 2: Run full workspace build**

Run: `cargo build --workspace`

Expected: Clean build with no errors or warnings

**Step 3: Run all tests**

Run: `cargo test --workspace`

Expected: All tests pass

**Step 4: Commit validation document**

```bash
git add docs/plans/2026-01-12-phase4-validation.md
git commit -m "docs(phase4): add architecture validation checklist results"
```

---

## Task 10: Update architecture design document

**Files:**

- Modify: `docs/plans/2026-01-12-bootstrap-architecture-design.md`

**Step 1: Mark Phase 4 as completed**

Edit: `docs/plans/2026-01-12-bootstrap-architecture-design.md`

Add to Phase 4 section:

```markdown
### Phase 4: Cleanup / 清理 ✅ COMPLETED

**Completed on / 完成于**: 2026-01-12

**Changes made / 完成的变更**:

1. ✅ Removed AppBuilder from uc-app
2. ✅ Updated bootstrap/run.rs to use direct App::new()
3. ✅ Fixed all unused import warnings
4. ✅ Fixed all dead_code warnings with #[allow(dead_code)]
5. ✅ Updated deprecated rand::thread_rng usage
6. ✅ Passed all architecture validation checkpoints

**Commits / 提交记录**:

- `refactor(uc-app): remove AppBuilder, use direct App construction`
- `refactor(uc-tauri): replace AppBuilder with direct App::new()`
- `fix(uc-infra): remove unused async_trait import`
- `fix(uc-tauri): prefix unused variable with underscore`
- `fix(uc-tauri): add allow(dead_code) for future use fields`
- `fix(uc-infra): remove unused imports from representation_repo`
- `fix(uc-infra): update rand API to use rng() instead of thread_rng()`
- `fix(uc-app): clean up unused imports and dead_code warnings`
- `docs(phase4): add architecture validation checklist results`
```

**Step 2: Commit**

```bash
git add docs/plans/2026-01-12-bootstrap-architecture-design.md
git commit -m "docs(bootstrap): mark Phase 4 as completed"
```

---

## Final Verification

**Step 1: Full workspace check**

Run: `cargo check --workspace --all-targets`

Expected: Zero errors, zero warnings

**Step 2: Run full test suite**

Run: `cargo test --workspace`

Expected: All tests pass

**Step 3: Create Phase 4 summary document**

Create: `docs/plans/2026-01-12-phase4-summary.md`

```markdown
# Phase 4: Bootstrap Cleanup - Summary

**Date**: 2026-01-12
**Status**: ✅ COMPLETED

## Overview

Phase 4 completes the bootstrap architecture migration by removing all temporary
backward-compatibility code and addressing technical debt.

## Key Accomplishments

1. **AppBuilder Removal**: The deprecated builder pattern has been completely removed
2. **Direct Construction**: All app instantiation now uses `App::new(AppDeps)`
3. **Clean Build**: Zero compiler warnings in the workspace
4. **Architecture Validation**: All 8 validation checkpoints passed

## Migration Complete

The bootstrap architecture migration is now complete. The codebase follows clean
hexagonal architecture principles with proper separation of concerns.
```

**Step 4: Final commit**

```bash
git add docs/plans/2026-01-12-phase4-summary.md
git commit -m "docs(phase4): add implementation summary"
```

---

## Testing Strategy

After each task:

1. Run `cargo check -p <affected-crate>` to verify no compilation errors
2. Run `cargo test -p <affected-crate>` to ensure tests pass
3. Run `cargo clippy -p <affected-crate>` for additional linting

Final verification:

1. `cargo build --workspace` - Full build
2. `cargo test --workspace` - All tests
3. `cargo clippy --workspace` - Lint checks

---

## Rollback Strategy

If any task fails:

1. Revert the specific commit: `git revert HEAD`
2. Identify the root cause
3. Fix the issue and try again

The migration is designed to be non-breaking - each task can be reverted
independently without affecting other tasks.
