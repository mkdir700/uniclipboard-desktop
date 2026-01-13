# uc-tauri Separation Design

**Date**: 2025-01-12
**Status**: Design Approved
**Author**: AI Assistant

## Overview

This document describes the architectural refactoring to separate all Tauri-specific code from `uc-platform` into a dedicated `uc-tauri` crate. This separation establishes clear architectural boundaries and enables the platform layer to remain UI framework agnostic.

## Motivation

### Current Problem

The `uc-platform` crate currently contains both OS-level platform code (clipboard, keyring, file system) and Tauri-specific adapters (`TauriAutostart`, `TauriUiPort`). This creates two issues:

1. **Architectural Boundary Violation**: Platform layer should not depend on UI framework
2. **Reduced Reusability**: Platform code cannot be reused in CLI/Daemon contexts

### Target Architecture

```
src-tauri/main.rs
    ↓
uc-tauri (Adapter + Bootstrap)
    ↓
uc-app (Use Cases)
    ↓
uc-core (Ports) ← uc-platform (OS implementations)
                    ↑
              uc-infra (Infrastructure)
```

## Directory Structure

```
crates/uc-tauri/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   │
│   ├── adapters/
│   │   ├── mod.rs
│   │   ├── autostart.rs    # TauriAutostart implementation
│   │   └── ui.rs           # TauriUiPort implementation
│   │
│   ├── bootstrap/
│   │   ├── mod.rs
│   │   ├── runtime.rs      # AppRuntimeSeed, create_runtime()
│   │   └── run.rs          # run_app()
│   │
│   └── commands/
│       ├── mod.rs          # register_all()
│       ├── dto.rs          # DTO definitions
│       ├── error.rs        # Unified error handling
│       ├── clipboard.rs    # Clipboard commands
│       ├── device.rs       # Device commands
│       ├── settings.rs     # Settings commands
│       └── encryption.rs   # Encryption commands
```

## Bootstrap Layer

### Key Design: Two-Phase Initialization

```rust
// bootstrap/runtime.rs
pub struct AppRuntimeSeed {
    pub app_builder: AppBuilder,
}

pub fn create_runtime() -> anyhow::Result<AppRuntimeSeed> {
    Ok(AppRuntimeSeed {
        app_builder: AppBuilder::new(),
    })
}

// bootstrap/run.rs
pub fn run_app(seed: AppRuntimeSeed) -> anyhow::Result<()> {
    tauri::Builder::default()
        .setup(|tauri_app| {
            let autostart = Arc::new(TauriAutostart::new(tauri_app.handle().clone()));
            let ui_port = Arc::new(TauriUiPort::new(tauri_app.handle().clone(), "settings"));

            let app = Arc::new(
                seed.app_builder
                    .with_autostart(autostart)
                    .with_ui_port(ui_port)
                    .build()?
            );

            tauri_app.manage(AppRuntime { app });
            Ok(())
        })
        .invoke_handler(crate::commands::register_all())
        .run(tauri::generate_context!())?;
    Ok(())
}
```

**Why `AppRuntimeSeed` not `AppRuntime`?**

- **Runtime** = completed, ready-to-run system
- **Seed** = assembly context for building the runtime
- Builder is a phase object, should not cross phase boundaries

### Bootstrap Layer Iron Laws

1. **Builder exists only during assembly phase**
2. **Runtime must be a completed object**
3. **`create_*` must not touch Tauri**
4. **`run_*` must not new business objects**
5. **Commands may only depend on Runtime**
6. **`main.rs` must not contain decision logic**

## Adapters Layer

### TauriAutostart

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

### TauriUiPort (with Window Label Injection)

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
        Err(anyhow::anyhow!("Settings window '{}' not found", self.settings_window_label))
    }
}
```

**Key Design Decisions:**

- **`pub(crate)` visibility**: Only `uc-tauri::bootstrap` can create adapters
- **Doc comment annotations**: Explicitly mark as "Runtime-bound"
- **Injected window label**: Adapter adapts capability, not UI policy

## Commands Layer

### Architecture: Thin Wrappers

Commands are minimal wrappers that only handle parameter conversion and error mapping. Business logic lives in `uc-app` Use Cases.

### DTO Pattern

Commands return DTOs, not Domain Models:

```rust
// commands/dto.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItemDto {
    pub id: String,
    pub content: String,
    pub timestamp: i64,
    pub content_type: String,
    pub preview: Option<String>,
}

impl From<uc_core::clipboard::ClipboardItem> for ClipboardItemDto {
    fn from(item: uc_core::clipboard::ClipboardItem) -> Self {
        Self {
            id: item.id().to_string(),
            content: item.content().to_string(),
            timestamp: item.timestamp().timestamp(),
            content_type: item.content_type().to_string(),
            preview: item.preview().map(|p| p.to_string()),
        }
    }
}
```

**Why DTOs?**

- **Frontend API stability**: Domain changes don't break frontend
- **Transport layer boundary**: Commands should not leak domain semantics
- **Explicit serialization control**: Optimize for frontend needs

### Unified Error Handling

```rust
// commands/error.rs
use anyhow::Error as AnyhowError;

pub type CommandResult<T> = Result<T, CommandError>;

#[derive(Debug)]
pub struct CommandError {
    pub code: &'static str,
    pub message: String,
}

impl CommandError {
    pub fn from_anyhow(err: AnyhowError) -> Self {
        Self {
            code: "INTERNAL_ERROR",
            message: err.to_string(),
        }
    }
}

impl Into<String> for CommandError {
    fn into(self) -> String {
        self.message
    }
}

/// Centralized error mapping for future upgrade path
pub fn map_err(err: anyhow::Error) -> String {
    err.to_string()
}
```

### Example Command

```rust
// commands/clipboard.rs
use super::{dto::ClipboardItemDto, error::map_err};
use crate::bootstrap::runtime::AppRuntime;

#[tauri::command]
pub async fn get_clipboard_items(
    state: tauri::State<'_, AppRuntime>,
) -> Result<Vec<ClipboardItemDto>, String> {
    state
        .app
        .list_clipboard_items()
        .await
        .map(|items| items.into_iter().map(ClipboardItemDto::from).collect())
        .map_err(map_err)
}
```

### Command Registration

```rust
// commands/mod.rs
pub fn register_all(builder: tauri::Builder) -> tauri::Builder {
    builder.invoke_handler(tauri::generate_handler![
        clipboard::get_clipboard_items,
        clipboard::delete_clipboard_item,
        settings::get_setting,
        settings::save_setting,
        encryption::get_encryption_password,
        encryption::set_encryption_password,
    ])
}

mod clipboard;
mod device;
mod settings;
mod encryption;
```

## Dependencies

### uc-tauri/Cargo.toml

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

### uc-platform/Cargo.toml (Cleaned)

Remove:

- `tauri`
- `tauri-plugin-autostart`

## Migration Steps

### Phase 1: Create uc-tauri Crate

```bash
mkdir -p src-tauri/crates/uc-tauri/src/{adapters,bootstrap,commands}
```

### Phase 2: Migrate Code

1. Move `uc-platform/src/tauri/*` → `uc-tauri/src/adapters/*`
2. Apply corrections:
   - Add `pub(crate)` to constructors
   - Add Runtime-bound doc comments
   - Inject window label into `TauriUiPort`
3. Implement bootstrap layer
4. Implement commands layer with DTOs

### Phase 3: Clean uc-platform

1. Remove Tauri dependencies from `Cargo.toml`
2. Delete `src/tauri/` directory

### Phase 4: Update main.rs

```rust
fn main() {
    let seed = uc_tauri::bootstrap::create_runtime()
        .expect("Failed to create runtime seed");
    uc_tauri::bootstrap::run_app(seed)
        .expect("Failed to run app");
}
```

### Phase 5: Verification

```bash
# uc-platform should compile without Tauri
cargo check -p uc-platform

# Full build
cargo check --manifest-path src-tauri/Cargo.toml
```

## Validation Checklist

| Check                              | Expected Result      |
| ---------------------------------- | -------------------- |
| `uc-platform` depends on Tauri     | ❌ Should NOT depend |
| `uc-tauri` depends on `uc-core`    | ✅ Should depend     |
| `uc-tauri` depends on `uc-app`     | ✅ Should depend     |
| `uc-platform` depends on `uc-core` | ✅ Should depend     |
| Commands return Domain Model       | ❌ Should return DTO |
| Adapter constructors visibility    | `pub(crate)`         |
| Adapters have doc comments         | ✅ Runtime-bound     |

## Naming Conventions

- **Adapters**: `TauriXxx` prefix (e.g., `TauriAutostart`, `TauriUiPort`)
- **Modules**: `adapters::`, `bootstrap::`, `commands::`
- **Runtime assembly**: `AppRuntimeSeed` (not `AppRuntime`)
- **Completed runtime**: `AppRuntime { app: Arc<App> }`

## Principles Established

1. **uc-platform prohibits UI runtime dependencies**
   - No `tauri`, `winit`, `egui`, `iced`

2. **Naming as architecture constraint**
   - `platform::` → forbidden in uc-tauri
   - Use `adapters::`, `bootstrap::`, `plugins::`

3. **Adapter adapts capability, not policy**
   - Window labels injected, not hardcoded
   - Adapters are OS capability wrappers

4. **Commands are transport layer**
   - Return DTOs, not Domain Models
   - Thin wrappers only

5. **Two-phase initialization**
   - `create_*`: No Tauri
   - `run_*`: No business object creation
   - Setup bridges the gap

## References

- Original discussion: [Context](#)
- Hexagonal Architecture: [Ports and Adapters Pattern](https://herbertograca.com/2017/09/14/ports-adapters-architecture/)
