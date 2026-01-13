# uc-tauri Separation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Separate all Tauri-specific code from `uc-platform` into a dedicated `uc-tauri` crate, establishing clear architectural boundaries.

**Architecture:**

- Create `uc-tauri` crate containing Tauri adapters, bootstrap layer, and commands
- Remove all Tauri dependencies from `uc-platform`
- Implement two-phase initialization (AppRuntimeSeed → setup → AppRuntime)
- Use DTO pattern for commands to decouple frontend from domain models

**Tech Stack:**

- Rust with Tauri 2
- Hexagonal Architecture (Ports and Adapters)
- Tokio async runtime
- anyhow for error handling

---

## Task 1: Create uc-tauri Crate Structure

**Files:**

- Create: `src-tauri/crates/uc-tauri/Cargo.toml`
- Create: `src-tauri/crates/uc-tauri/src/lib.rs`
- Create: `src-tauri/crates/uc-tauri/src/adapters/mod.rs`
- Create: `src-tauri/crates/uc-tauri/src/adapters/autostart.rs`
- Create: `src-tauri/crates/uc-tauri/src/adapters/ui.rs`
- Create: `src-tauri/crates/uc-tauri/src/bootstrap/mod.rs`
- Create: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`
- Create: `src-tauri/crates/uc-tauri/src/bootstrap/run.rs`
- Create: `src-tauri/crates/uc-tauri/src/commands/mod.rs`
- Create: `src-tauri/crates/uc-tauri/src/commands/dto.rs`
- Create: `src-tauri/crates/uc-tauri/src/commands/error.rs`

**Step 1: Create directory structure**

```bash
mkdir -p src-tauri/crates/uc-tauri/src/{adapters,bootstrap,commands}
```

**Step 2: Create Cargo.toml**

Create: `src-tauri/crates/uc-tauri/Cargo.toml`

```toml
[package]
name = "uc-tauri"
version = "0.1.0"
edition = "2021"
description = "Tauri adapter layer for UniClipboard"

[dependencies]
# Workspace crates
uc-core = { path = "../uc-core" }
uc-app = { path = "../uc-app" }

# Tauri
tauri = { workspace = true }
tauri-plugin-autostart = { workspace = true }

# Async
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Error handling
anyhow = "1.0"
thiserror = "2.0"

# Logging
log = "0.4"
```

**Step 3: Verify crate is added to workspace**

Modify: `src-tauri/Cargo.toml` - Add `uc-tauri` to workspace members

```toml
[workspace]
members = [
  "crates/uc-core",
  "crates/uc-app",
  "crates/uc-platform",
  "crates/uc-infra",
  "crates/uc-clipboard-probe",
  "crates/uc-tauri",  # Add this line
]
```

**Step 4: Create lib.rs skeleton**

Create: `src-tauri/crates/uc-tauri/src/lib.rs`

```rust
//! # uc-tauri
//!
//! Tauri adapter layer for UniClipboard.
//!
//! This crate contains Tauri-specific implementations of ports from uc-core,
//! bootstrap logic for application initialization, and Tauri command handlers.

pub mod adapters;
pub mod bootstrap;
pub mod commands;
```

**Step 5: Run cargo check to verify crate structure**

```bash
cargo check -p uc-tauri
```

Expected: OK or warnings about unused modules (acceptable)

**Step 6: Commit**

```bash
git add src-tauri/crates/uc-tauri/ src-tauri/Cargo.toml
git commit -m "feat(uc-tauri): create crate structure

Add uc-tauri crate with basic directory structure:
- adapters/ for Tauri-specific port implementations
- bootstrap/ for two-phase initialization
- commands/ for Tauri command handlers

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 2: Implement Adapters Layer

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/adapters/mod.rs`
- Modify: `src-tauri/crates/uc-tauri/src/adapters/autostart.rs`
- Modify: `src-tauri/crates/uc-tauri/src/adapters/ui.rs`
- Copy from: `src-tauri/crates/uc-platform/src/tauri/tauri_autostart.rs`
- Copy from: `src-tauri/crates/uc-platform/src/tauri/ui_port.rs`

**Step 1: Implement adapters/mod.rs**

Modify: `src-tauri/crates/uc-tauri/src/adapters/mod.rs`

```rust
pub mod autostart;
pub mod ui;

pub use autostart::TauriAutostart;
pub use ui::TauriUiPort;
```

**Step 2: Implement TauriAutostart adapter**

Modify: `src-tauri/crates/uc-tauri/src/adapters/autostart.rs`

```rust
use anyhow::Result;
use tauri::{AppHandle, ManagerExt as _};
use uc_core::ports::AutostartPort;

/// Tauri-specific runtime adapter for autostart functionality.
///
/// This adapter must only be constructed inside Tauri setup phase
/// and must not be used outside uc-tauri.
pub struct TauriAutostart {
    app_handle: AppHandle,
}

impl TauriAutostart {
    pub(crate) fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }
}

impl AutostartPort for TauriAutostart {
    fn is_enabled(&self) -> Result<bool> {
        self.app_handle
            .autolaunch()
            .is_enabled()
            .map_err(anyhow::Error::from)
    }

    fn enable(&self) -> Result<()> {
        self.app_handle.autolaunch().enable()?;
        Ok(())
    }

    fn disable(&self) -> Result<()> {
        self.app_handle.autolaunch().disable()?;
        Ok(())
    }
}
```

**Step 3: Implement TauriUiPort adapter**

Modify: `src-tauri/crates/uc-tauri/src/adapters/ui.rs`

```rust
use anyhow::Result;
use tauri::{AppHandle, Manager};
use uc_core::ports::UiPort;

/// Tauri-specific runtime adapter for UI operations.
///
/// This adapter must only be constructed inside Tauri setup phase
/// and must not be used outside uc-tauri.
pub struct TauriUiPort {
    app: AppHandle,
    settings_window_label: String,
}

impl TauriUiPort {
    pub(crate) fn new(
        app: AppHandle,
        settings_window_label: impl Into<String>,
    ) -> Self {
        Self {
            app,
            settings_window_label: settings_window_label.into(),
        }
    }
}

#[async_trait::async_trait]
impl UiPort for TauriUiPort {
    async fn open_settings(&self) -> Result<()> {
        if let Some(win) = self.app.get_webview_window(&self.settings_window_label) {
            win.show()?;
            win.set_focus()?;
            return Ok(());
        }
        Err(anyhow::anyhow!(
            "Settings window '{}' not found",
            self.settings_window_label
        ))
    }
}
```

**Step 4: Run cargo check to verify adapters compile**

```bash
cargo check -p uc-tauri
```

Expected: OK

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/adapters/
git commit -m "feat(uc-tauri): implement adapters layer

Add TauriAutostart and TauriUiPort adapters:
- pub(crate) constructors for architecture guardrails
- Runtime-bound doc comments
- Injected window label for UI policy separation

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 3: Implement Bootstrap Layer (Part 1 - RuntimeSeed)

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/mod.rs`
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`

**Step 1: Implement bootstrap/mod.rs**

Modify: `src-tauri/crates/uc-tauri/src/bootstrap/mod.rs`

```rust
pub mod runtime;
pub mod run;

pub use runtime::{create_runtime, AppRuntimeSeed};
pub use run::run_app;
```

**Step 2: Create a minimal AppBuilder in uc-app (if needed)**

First, check if `uc-app` has an AppBuilder. If not, create a minimal one:

Create: `src-tauri/crates/uc-app/src/builder.rs`

```rust
use std::sync::Arc;
use uc_core::ports::{AutostartPort, UiPort};

/// Builder for assembling the application runtime.
pub struct AppBuilder {
    autostart: Option<Arc<dyn AutostartPort>>,
    ui_port: Option<Arc<dyn UiPort>>,
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            autostart: None,
            ui_port: None,
        }
    }

    pub fn with_autostart(mut self, autostart: Arc<dyn AutostartPort>) -> Self {
        self.autostart = Some(autostart);
        self
    }

    pub fn with_ui_port(mut self, ui_port: Arc<dyn UiPort>) -> Self {
        self.ui_port = Some(ui_port);
        self
    }

    pub fn build(self) -> anyhow::Result<App> {
        Ok(App {
            autostart: self.autostart.ok_or_else(|| {
                anyhow::anyhow!("AutostartPort is required")
            })?,
            ui_port: self.ui_port.ok_or_else(|| {
                anyhow::anyhow!("UiPort is required")
            })?,
        })
    }
}

/// The application runtime.
pub struct App {
    pub autostart: Arc<dyn AutostartPort>,
    pub ui_port: Arc<dyn UiPort>,
}
```

Update: `src-tauri/crates/uc-app/src/lib.rs`

```rust
//! UniClipboard Application Orchestration Layer
//!
//! This crate contains business logic use cases and runtime orchestration.

pub mod bootstrap;
pub mod builder;
pub mod models;
pub mod ports;
pub mod usecases;

pub use builder::{App, AppBuilder};
pub use models::ClipboardEntryProjection;
```

**Step 3: Implement bootstrap/runtime.rs**

Modify: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`

```rust
use uc_app::AppBuilder;

/// Seed for creating the application runtime.
///
/// This is an assembly context that holds the AppBuilder
/// before Tauri setup phase completes. It does NOT contain
/// a fully constructed runtime - that happens in the setup phase.
pub struct AppRuntimeSeed {
    pub app_builder: AppBuilder,
}

/// Create the runtime seed without touching Tauri.
///
/// This function must not depend on Tauri or any UI framework.
pub fn create_runtime() -> anyhow::Result<AppRuntimeSeed> {
    Ok(AppRuntimeSeed {
        app_builder: AppBuilder::new(),
    })
}
```

**Step 4: Run cargo check**

```bash
cargo check -p uc-tauri
cargo check -p uc-app
```

Expected: OK

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/ src-tauri/crates/uc-app/
git commit -m "feat(uc-tauri): implement AppRuntimeSeed and AppBuilder

Add two-phase initialization support:
- AppRuntimeSeed holds AppBuilder before Tauri setup
- AppBuilder for dependency injection
- create_runtime() does not touch Tauri

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 4: Implement Bootstrap Layer (Part 2 - run_app)

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/run.rs`

**Step 1: Implement bootstrap/run.rs**

Modify: `src-tauri/crates/uc-tauri/src/bootstrap/run.rs`

```rust
use super::runtime::AppRuntimeSeed;
use crate::adapters::{TauriAutostart, TauriUiPort};
use std::sync::Arc;
use tauri::Builder;
use uc_app::App;

/// The completed application runtime.
///
/// This struct holds the fully assembled App instance
/// and is managed by Tauri's state system.
pub struct Runtime {
    pub app: Arc<App>,
}

/// Run the Tauri application with the given runtime seed.
///
/// This function handles the Tauri setup phase where
/// AppHandle-dependent adapters are created and injected.
pub fn run_app(seed: AppRuntimeSeed) -> anyhow::Result<()> {
    Builder::default()
        .setup(|tauri_app| {
            // Create Tauri-specific adapters
            let autostart = Arc::new(TauriAutostart::new(tauri_app.handle().clone()));
            let ui_port = Arc::new(TauriUiPort::new(
                tauri_app.handle().clone(),
                "settings",
            ));

            // Build the App with injected dependencies
            let app = Arc::new(
                seed.app_builder
                    .with_autostart(autostart)
                    .with_ui_port(ui_port)
                    .build()?,
            );

            // Register the completed runtime with Tauri
            tauri_app.manage(Runtime { app });

            Ok(())
        })
        .invoke_handler(crate::commands::register_all)
        .run(tauri::generate_context!())
        .map_err(|e| anyhow::anyhow!("Tauri error: {}", e))?;

    Ok(())
}
```

**Step 2: Add placeholder register_all function in commands**

Modify: `src-tauri/crates/uc-tauri/src/commands/mod.rs`

```rust
use tauri::Builder;

pub fn register_all(builder: Builder) -> Builder {
    // TODO: Add commands in Task 6
    builder
}
```

**Step 3: Run cargo check**

```bash
cargo check -p uc-tauri
```

Expected: OK

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/ src-tauri/crates/uc-tauri/src/commands/mod.rs
git commit -m "feat(uc-tauri): implement run_app with two-phase initialization

Complete the bootstrap layer:
- run_app() handles Tauri setup phase
- Creates Tauri-specific adapters with AppHandle
- Builds completed App and registers with Tauri state
- Runtime struct wraps Arc<App> for commands access

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 5: Implement Commands DTO and Error Infrastructure

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/dto.rs`
- Modify: `src-tauri/crates/uc-tauri/src/commands/error.rs`

**Step 1: Implement commands/dto.rs**

Modify: `src-tauri/crates/uc-tauri/src/commands/dto.rs`

```rust
use serde::{Deserialize, Serialize};

/// Clipboard item DTO for frontend API.
///
/// This DTO separates the frontend API from internal domain models,
/// allowing domain evolution without breaking the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItemDto {
    pub id: String,
    pub content: String,
    pub timestamp: i64,
    pub content_type: String,
    pub preview: Option<String>,
}

// Conversion implementations will be added after checking uc-core models
```

**Step 2: Implement commands/error.rs**

Modify: `src-tauri/crates/uc-tauri/src/commands/error.rs`

```rust
use anyhow::Error as AnyhowError;

/// Centralized error mapping for commands.
///
/// This function provides a single upgrade path for future
/// CommandError enhancements (e.g., error codes).
pub fn map_err(err: anyhow::Error) -> String {
    err.to_string()
}
```

**Step 3: Update commands/mod.rs to export DTO and error**

Modify: `src-tauri/crates/uc-tauri/src/commands/mod.rs`

```rust
pub mod dto;
pub mod error;

use tauri::Builder;

pub fn register_all(builder: Builder) -> Builder {
    // TODO: Add commands in Task 6
    builder
}

pub use error::map_err;
```

**Step 4: Run cargo check**

```bash
cargo check -p uc-tauri
```

Expected: OK

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/
git commit -m "feat(uc-tauri): implement commands infrastructure

Add DTO and error handling infrastructure:
- ClipboardItemDto for frontend API (will add From impl later)
- Centralized map_err() function for future error upgrades
- Commands module structure with dto/ and error/ submodules

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 6: Update src-tauri/main.rs to Use New Bootstrap

**Files:**

- Modify: `src-tauri/src/main.rs` (check existing structure first)

**Step 1: Check current main.rs structure**

```bash
head -50 src-tauri/src/main.rs
```

**Step 2: Backup current main.rs**

```bash
cp src-tauri/src/main.rs src-tauri/src/main.rs.backup
```

**Step 3: Update main.rs to use uc-tauri bootstrap**

Modify: `src-tauri/src/main.rs`

```rust
fn main() {
    let seed = uc_tauri::bootstrap::create_runtime()
        .expect("Failed to create runtime seed");

    uc_tauri::bootstrap::run_app(seed)
        .expect("Failed to run app");
}
```

Note: Keep any existing setup code that's critical (like panic hooks, log init, etc.)

**Step 4: Update src-tauri/Cargo.toml to depend on uc-tauri**

Modify: `src-tauri/Cargo.toml` - Add to dependencies:

```toml
[dependencies]
# ... existing dependencies ...
uc-tauri = { path = "crates/uc-tauri" }
```

**Step 5: Run cargo check**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: May fail due to uc-platform still having tauri dependencies (will fix in Task 7)

**Step 6: Commit**

```bash
git add src-tauri/src/main.rs src-tauri/Cargo.toml
git commit -m "feat(main): use uc-tauri bootstrap layer

Update main.rs to use two-phase initialization:
- create_runtime() for dependency setup
- run_app() for Tauri startup

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 7: Remove Tauri Dependencies from uc-platform

**Files:**

- Modify: `src-tauri/crates/uc-platform/Cargo.toml`
- Delete: `src-tauri/crates/uc-platform/src/tauri/`
- Modify: `src-tauri/crates/uc-platform/src/lib.rs`

**Step 1: Remove Tauri dependencies from uc-platform/Cargo.toml**

Modify: `src-tauri/crates/uc-platform/Cargo.toml`

Remove these lines:

```toml
tauri = { workspace = true }
tauri-plugin-autostart = { workspace = true }
```

**Step 2: Remove tauri module from uc-platform/lib.rs**

Modify: `src-tauri/crates/uc-platform/src/lib.rs`

Remove the `pub mod tauri;` line:

```rust
//! # uc-platform
//!
//! Platform-specific implementations for UniClipboard.
//!
//! This crate contains infrastructure implementations that interact with
//! the operating system, external services, and hardware.

pub mod bootstrap;
pub mod clipboard;
pub mod ipc;
pub mod keyring;
pub mod ports;
pub mod runtime;
// pub mod tauri;  // Remove this line
```

**Step 3: Delete the tauri directory**

```bash
rm -rf src-tauri/crates/uc-platform/src/tauri/
```

**Step 4: Verify uc-platform compiles without Tauri**

```bash
cargo check -p uc-platform
```

Expected: OK (if there are other errors, fix them as they arise)

**Step 5: Run full build check**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: OK

**Step 6: Commit**

```bash
git add src-tauri/crates/uc-platform/
git commit -m "refactor(uc-platform): remove Tauri dependencies

Remove all Tauri-specific code from uc-platform:
- Remove tauri and tauri-plugin-autostart dependencies
- Delete tauri/ module directory
- Remove tauri module from lib.rs

This enforces the architectural boundary that uc-platform
should only contain OS-level code, not UI framework code.

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 8: Fix Any Compilation Errors

**Files:**

- Various (depending on what breaks)

**Step 1: Run full cargo check**

```bash
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tee /tmp/cargo-check.log
```

**Step 2: Review and fix errors**

For each error:

1. Identify the root cause
2. Implement the fix
3. Re-verify

Common issues may include:

- Missing use statements
- Type mismatches after refactoring
- Missing trait implementations

**Step 3: Ensure successful compilation**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: OK with no errors

**Step 4: Commit**

```bash
git add -A
git commit -m "fix: resolve compilation errors after uc-tauri separation

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 9: Validate Architecture Boundaries

**Files:**

- None (validation only)

**Step 1: Verify uc-platform has no Tauri dependency**

```bash
grep -r "tauri" src-tauri/crates/uc-platform/Cargo.toml
```

Expected: No results

**Step 2: Verify uc-tauri depends on correct crates**

```bash
grep "uc-" src-tauri/crates/uc-tauri/Cargo.toml
```

Expected:

```
uc-core = { path = "../uc-core" }
uc-app = { path = "../uc-app" }
```

**Step 3: Run tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

**Step 4: Build release version**

```bash
cargo build --manifest-path src-tauri/Cargo.toml --release
```

**Step 5: Manual smoke test**

```bash
cargo run --manifest-path src-tauri/Cargo.toml
```

Verify:

- Application starts
- No panic on startup
- Settings window can open (if applicable)
- Autostart toggle works (if applicable)

**Step 6: Create validation checklist document**

Create: `docs/plans/2025-01-12-uc-tauri-validation.md`

```markdown
# uc-tauri Separation Validation Checklist

## Architecture Boundary Validation

- [ ] uc-platform does NOT depend on Tauri
- [ ] uc-tauri depends on uc-core
- [ ] uc-tauri depends on uc-app
- [ ] uc-platform depends on uc-core
- [ ] Adapter constructors are pub(crate)
- [ ] Adapters have Runtime-bound doc comments

## Functional Validation

- [ ] Application starts without panic
- [ ] Autostart enable/disable works
- [ ] Settings window opens correctly
- [ ] All clipboard operations work
- [ ] No console errors on startup

## Build Validation

- [ ] `cargo check -p uc-platform` passes
- [ ] `cargo check -p uc-tauri` passes
- [ ] `cargo build --release` succeeds
```

**Step 7: Commit validation document**

```bash
git add docs/plans/2025-01-12-uc-tauri-validation.md
git commit -m "docs(uc-tauri): add validation checklist

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 10: Final Documentation and Cleanup

**Files:**

- Update: `docs/plans/2025-01-12-uc-tauri-separation-design.md`
- Update: `CLAUDE.md` (if needed)

**Step 1: Update design document with implementation notes**

Add to end of design document:

```markdown
## Implementation Notes

### Completed Tasks

- [x] Create uc-tauri crate structure
- [x] Implement adapters layer
- [x] Implement bootstrap layer (two-phase initialization)
- [x] Implement commands infrastructure
- [x] Update main.rs to use new bootstrap
- [x] Remove Tauri dependencies from uc-platform
- [x] Fix compilation errors
- [x] Validate architecture boundaries

### Post-Implementation TODOs

- Add comprehensive commands implementation (clipboard, settings, encryption)
- Implement full DTO conversions for all domain models
- Add integration tests for bootstrap layer
- Consider adding CommandError with error codes
```

**Step 2: Update CLAUDE.md with new crate info**

Add to the Architecture section:

```markdown
#### New Architecture (Current)

The architecture follows **Hexagonal Architecture (Ports and Adapters)**:
```

src-tauri/crates/
├── uc-core/ # Core domain layer
│ └── ports/ # Port definitions (traits)
├── uc-app/ # Application layer (use cases, AppBuilder)
│ └── builder.rs # Runtime assembly
├── uc-platform/ # Platform adapter layer (OS-level only)
│ ├── clipboard/ # Platform-specific clipboard
│ ├── keyring.rs # Platform-specific keyring
│ └── runtime/ # Event bus, runtime (no Tauri)
├── uc-tauri/ # Tauri adapter layer (NEW)
│ ├── adapters/ # Tauri-specific port implementations
│ ├── bootstrap/ # Two-phase initialization
│ └── commands/ # Tauri command handlers
└── uc-infra/ # Infrastructure implementations

```

```

**Step 3: Final commit**

```bash
git add docs/plans/ CLAUDE.md
git commit -m "docs(uc-tauri): update documentation after separation

Update design document and CLAUDE.md with:
- Completed implementation tasks
- Post-implementation TODOs
- New crate architecture overview

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Summary

This implementation plan separates all Tauri-specific code from `uc-platform` into a dedicated `uc-tauri` crate. The key architectural improvements are:

1. **Clear Boundaries**: `uc-platform` contains only OS-level code, no UI framework dependencies
2. **Two-Phase Initialization**: `create_runtime()` (no Tauri) → `run_app()` (Tauri setup)
3. **DTO Pattern**: Commands return DTOs, not domain models
4. **Architecture Guardrails**: `pub(crate)` constructors, Runtime-bound doc comments

**Total estimated steps**: ~50 atomic steps across 10 tasks
**Recommended commit frequency**: Every task (1-2 commits per task)
**Testing strategy**: Compile checks after each task, manual smoke test at end
