# Phase 4: Complete Feature Implementation

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement complete end-to-end functionality by wiring use cases to commands, adding clipboard monitoring, and implementing encryption initialization flow.

**Architecture:** This phase completes the application layer by connecting the IPC commands (Phase 3) to business logic use cases, enabling full user-facing features.

**Tech Stack:**

- Rust 1.75+
- Tauri 2.0
- Tokio async runtime
- React 18 + TypeScript (frontend)

**Prerequisites:**

- ✅ Phase 1 complete (infrastructure, repositories, blob store)
- ✅ Phase 2 complete (materializers, encryption session, use case integration)
- ✅ Phase 3 complete (IPC commands, event system)

---

## Overview

This plan implements the complete feature set by:

1. **Use Case Factory** - Create factory functions to instantiate use cases with AppDeps
2. **Command Wiring** - Connect Tauri commands to use case execution
3. **Clipboard Monitoring** - Background clipboard capture service
4. **Encryption Flow** - Complete encryption initialization and session management
5. **Integration Tests** - End-to-end tests for critical flows

---

## Task 1: Implement Use Case Factory

**Goal:** Create factory functions that instantiate use cases from AppDeps.

**Files:**

- Modify: `src-tauri/crates/uc-app/src/usecase_factory.rs`

**Step 1: Analyze existing use cases**

Based on code review, the following use cases exist:

- `CaptureClipboardUseCase` - Capture clipboard content
- `ListClipboardEntryPreviewsUseCase` - List clipboard history (commented out)
- `MaterializeClipboardSelectionUseCase` - Restore clipboard content
- `InitializeEncryption` - Initialize encryption
- `ChangePassphrase` - Change encryption passphrase
- `ApplyAutostart` - Apply autostart settings
- `ApplyTheme` - Apply theme settings
- `OpenSettingsWindow` - Open settings window

**Step 2: Implement CaptureClipboardUseCase factory**

Modify `src-tauri/crates/uc-app/src/usecase_factory.rs`:

```rust
//! Factory functions for creating use cases with AppDeps
//! 使用 AppDeps 创建用例的工厂函数

use std::sync::Arc;
use uc_app::AppDeps;
use uc_app::usecases::internal::capture_clipboard::CaptureClipboardUseCase;
use uc_app::usecases::internal::materialize_clipboard_selection::MaterializeClipboardSelectionUseCase;
use uc_app::usecases::initialize_encryption::InitializeEncryption;

/// Create CaptureClipboardUseCase with dependencies from AppDeps
pub fn create_capture_clipboard_use_case(deps: &Arc<AppDeps>) -> CaptureClipboardUseCase<
    Arc<dyn uc_core::ports::SystemClipboardPort>,
    Arc<dyn uc_core::ports::ClipboardEntryRepositoryPort>,
    Arc<dyn uc_core::ports::ClipboardEventRepositoryPort>,
    Arc<dyn uc_core::ports::SelectRepresentationPolicyPort>,
    Arc<dyn uc_core::ports::ClipboardRepresentationMaterializerPort>,
    Arc<dyn uc_core::ports::DeviceIdentityPort>,
> {
    CaptureClipboardUseCase::new(
        Arc::clone(&deps.clipboard),
        Arc::clone(&deps.clipboard_entry_repo),
        Arc::clone(&deps.clipboard_event_repo),
        Arc::clone(&deps.representation_policy),
        Arc::clone(&deps.representation_materializer),
        Arc::clone(&deps.device_identity),
    )
}

/// Create MaterializeClipboardSelectionUseCase with dependencies from AppDeps
pub fn create_materialize_clipboard_selection_use_case(deps: &Arc<AppDeps>) -> MaterializeClipboardSelectionUseCase<
    Arc<dyn uc_core::ports::ClipboardSelectionRepositoryPort>,
    Arc<dyn uc_core::ports::ClipboardRepresentationRepositoryPort>,
    Arc<dyn uc_core::ports::BlobMaterializerPort>,
    Arc<dyn uc_core::ports::BlobStorePort>,
    Arc<dyn uc_core::ports::SystemClipboardPort>,
> {
    MaterializeClipboardSelectionUseCase::new(
        Arc::clone(&deps.selection_repo),
        Arc::clone(&deps.representation_repo),
        Arc::clone(&deps.blob_materializer),
        Arc::clone(&deps.blob_store),
        Arc::clone(&deps.clipboard),
    )
}

/// Create InitializeEncryption use case with dependencies from AppDeps
/// Note: KeyScopePort and EncryptionStatePort are not yet in AppDeps
pub fn create_initialize_encryption_use_case(
    deps: &Arc<AppDeps>,
) -> Option<InitializeEncryption<
    Arc<dyn uc_core::ports::EncryptionPort>,
    Arc<dyn uc_core::ports::KeyMaterialPort>,
    Arc<dyn uc_core::ports::KeyScopePort>,
    Arc<dyn uc_core::ports::EncryptionStatePort>,
>> {
    // TODO: Implement when KeyScopePort and EncryptionStatePort are added to AppDeps
    None
}
```

**Step 3: Verify compilation**

```bash
cd src-tauri && cargo check --package uc-app
```

Expected: No errors

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecase_factory.rs
git commit -m "feat(uc-app): implement use case factory functions

- Add create_capture_clipboard_use_case
- Add create_materialize_clipboard_selection_use_case
- Add placeholder for initialize_encryption

Generated with [Claude Code](https://claude.ai/code)
via [Happy](https://happy.engineering)

Co-Authored-By: Claude <noreply@anthropic.com>
Co-Authored-By: Happy <yesreply@happy.engineering>"
```

---

## Task 2: Wire Commands to Use Cases

**Goal:** Connect Tauri commands to use case execution.

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/clipboard.rs`
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`

**Step 1: Implement get_clipboard_entries command**

Modify `src-tauri/crates/uc-tauri/src/commands/clipboard.rs`:

```rust
use tauri::State;
use uc_app::AppDeps;
use uc_app::models::ClipboardEntryProjection;
use uc_core::ports::ClipboardEntryRepositoryPort;

/// Get clipboard history entries
/// 获取剪贴板历史条目
#[tauri::command]
pub async fn get_clipboard_entries(
    deps: State<'_, AppDeps>,
    limit: Option<usize>,
) -> Result<Vec<ClipboardEntryProjection>, String> {
    let limit = limit.unwrap_or(50);

    // Query entries from repository
    let entries = deps.clipboard_entry_repo
        .list_entries(limit, 0)
        .await
        .map_err(|e| format!("Failed to query entries: {}", e))?;

    // Convert to projections
    let projections: Vec<ClipboardEntryProjection> = entries
        .into_iter()
        .map(|entry| ClipboardEntryProjection {
            id: entry.id().to_string(),
            created_at: entry.created_at_ms(),
            preview: format!("Entry with {} bytes", entry.total_size_bytes()),
        })
        .collect();

    Ok(projections)
}
```

**Step 2: Implement capture_clipboard command**

Add to `src-tauri/crates/uc-tauri/src/commands/clipboard.rs`:

```rust
use uc_app::usecase_factory;

/// Capture current clipboard content
/// 捕获当前剪贴板内容
#[tauri::command]
pub async fn capture_clipboard(
    deps: State<'_, AppDeps>,
) -> Result<String, String> {
    let use_case = usecase_factory::create_capture_clipboard_use_case(&deps.inner());

    use_case
        .execute()
        .await
        .map(|event_id| event_id.to_string())
        .map_err(|e| format!("Failed to capture clipboard: {}", e))
}
```

**Step 3: Verify compilation**

```bash
cd src-tauri && cargo check --package uc-tauri
```

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/clipboard.rs
git commit -m "feat(uc-tauri): wire clipboard commands to use cases

- Implement get_clipboard_entries with repository query
- Implement capture_clipboard with use case factory
- Convert domain models to DTO projections

Generated with [Claude Code](https://claude.ai/code)
via [Happy](https://happy.engineering)

Co-Authored-By: Claude <noreply@anthropic.com>
Co-Authored-By: Happy <yesreply@happy.engineering>"
```

---

## Task 3: Implement Clipboard Monitoring Service

**Goal:** Background service that monitors clipboard for changes.

**Files:**

- Create: `src-tauri/crates/uc-tauri/src/services/clipboard_monitor.rs`
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`

**Step 1: Create clipboard monitor module**

Create `src-tauri/crates/uc-tauri/src/services/clipboard_monitor.rs`:

```rust
//! Clipboard Monitoring Service
//! 剪贴板监控服务

use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tauri::{AppHandle, Emitter};
use uc_app::AppDeps;
use uc_app::usecase_factory;

/// Clipboard monitoring service
pub struct ClipboardMonitor {
    app: AppHandle,
    deps: Arc<AppDeps>,
    interval_secs: u64,
}

impl ClipboardMonitor {
    pub fn new(app: AppHandle, deps: Arc<AppDeps>) -> Self {
        Self {
            app,
            deps,
            interval_secs: 1, // Check every second
        }
    }

    /// Start the clipboard monitoring loop
    pub async fn run(&self) -> anyhow::Result<()> {
        let mut timer = interval(Duration::from_secs(self.interval_secs));
        let use_case = usecase_factory::create_capture_clipboard_use_case(&self.deps);

        loop {
            timer.tick().await;

            // TODO: Check if clipboard content changed before capturing
            // For now, capture on every tick
            match use_case.execute().await {
                Ok(event_id) => {
                    tracing::debug!("Captured clipboard event: {}", event_id);

                    // Emit event to frontend
                    let _ = self.app.emit("clipboard://captured", event_id.to_string());
                }
                Err(e) => {
                    tracing::warn!("Failed to capture clipboard: {}", e);
                }
            }
        }
    }
}
```

**Step 2: Create services module**

Create `src-tauri/crates/uc-tauri/src/services/mod.rs`:

```rust
pub mod clipboard_monitor;
```

**Step 3: Verify compilation**

```bash
cd src-tauri && cargo check --package uc-tauri
```

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/services/
git commit -m "feat(uc-tauri): add clipboard monitoring service

- Create ClipboardMonitor for background capture
- Emit clipboard://captured events to frontend
- Poll every second for clipboard changes

Generated with [Claude Code](https://claude.ai/code)
via [Happy](https://happy.engineering)

Co-Authored-By: Claude <noreply@anthropic.com>
Co-Authored-By: Happy <yesreply@happy.engineering>"
```

---

## Task 4: Implement Encryption Initialization Flow

**Goal:** Complete encryption initialization with missing ports.

**Files:**

- Create: `src-tauri/crates/uc-infra/src/security/encryption_state_repo.rs`
- Create: `src-tauri/crates/uc-platform/src/key_scope.rs`
- Modify: `src-tauri/crates/uc-core/src/ports/mod.rs`
- Modify: `src-tauri/crates/uc-app/src/deps.rs`

**Step 1: Create EncryptionStatePort implementation**

Create `src-tauri/crates/uc-infra/src/security/encryption_state_repo.rs`:

```rust
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use uc_core::ports::EncryptionStatePort;
use uc_core::security::state::{EncryptionState, EncryptionStateError};

const ENCRYPTION_STATE_FILE: &str = ".initialized_encryption";

pub struct FileEncryptionStateRepository {
    state_file: PathBuf,
}

impl FileEncryptionStateRepository {
    pub fn new(config_dir: PathBuf) -> Self {
        let state_file = config_dir.join(ENCRYPTION_STATE_FILE);
        Self { state_file }
    }
}

#[async_trait::async_trait]
impl EncryptionStatePort for FileEncryptionStateRepository {
    async fn load_state(&self) -> Result<EncryptionState, EncryptionStateError> {
        if self.state_file.exists().await {
            Ok(EncryptionState::Initialized)
        } else {
            Ok(EncryptionState::NotInitialized)
        }
    }

    async fn persist_initialized(&self) -> Result<(), EncryptionStateError> {
        fs::write(&self.state_file, b"1")
            .await
            .map_err(|e| EncryptionStateError::PersistenceFailed(e.to_string()))?;
        Ok(())
    }
}
```

**Step 2: Create KeyScopePort implementation**

Create `src-tauri/crates/uc-platform/src/key_scope.rs`:

```rust
use anyhow::Result;
use uc_core::ports::KeyScopePort;
use uc_core::security::model::{KeyScope, ScopeError};

/// Default key scope implementation
pub struct DefaultKeyScope {
    scope: KeyScope,
}

impl DefaultKeyScope {
    pub fn new() -> Self {
        Self {
            scope: KeyScope::default(),
        }
    }
}

impl Default for DefaultKeyScope {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl KeyScopePort for DefaultKeyScope {
    async fn current_scope(&self) -> Result<KeyScope, ScopeError> {
        Ok(self.scope.clone())
    }
}
```

**Step 3: Add ports to uc-core**

Modify `src-tauri/crates/uc-core/src/ports/mod.rs` to export new ports.

**Step 4: Add to AppDeps**

Modify `src-tauri/crates/uc-app/src/deps.rs`:

```rust
pub struct AppDeps {
    // ... existing fields ...

    // Encryption state
    pub encryption_state: Arc<dyn EncryptionStatePort>,
    pub key_scope: Arc<dyn KeyScopePort>,
}
```

**Step 5: Wire in dependencies**

Modify `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`:

```rust
use uc_infra::security::FileEncryptionStateRepository;
use uc_platform::key_scope::DefaultKeyScope;

// In create_infra_layer:
let encryption_state: Arc<dyn EncryptionStatePort> =
    Arc::new(FileEncryptionStateRepository::new(vault_path.clone()));

// In create_platform_layer:
let key_scope: Arc<dyn KeyScopePort> = Arc::new(DefaultKeyScope::new());

// In wire_dependencies AppDeps:
AppDeps {
    // ... existing ...
    encryption_state: infra.encryption_state,
    key_scope: platform.key_scope,
}
```

**Step 6: Update use case factory**

Modify `src-tauri/crates/uc-app/src/usecase_factory.rs`:

```rust
pub fn create_initialize_encryption_use_case(
    deps: &Arc<AppDeps>,
) -> InitializeEncryption<
    Arc<dyn uc_core::ports::EncryptionPort>,
    Arc<dyn uc_core::ports::KeyMaterialPort>,
    Arc<dyn uc_core::ports::KeyScopePort>,
    Arc<dyn uc_core::ports::EncryptionStatePort>,
> {
    InitializeEncryption::new(
        Arc::clone(&deps.encryption),
        Arc::clone(&deps.key_material),
        Arc::clone(&deps.key_scope),
        Arc::clone(&deps.encryption_state),
    )
}
```

**Step 7: Verify compilation**

```bash
cd src-tauri && cargo check --workspace
```

**Step 8: Commit**

```bash
git add src-tauri/crates/uc-infra/src/security/encryption_state_repo.rs
git add src-tauri/crates/uc-platform/src/key_scope.rs
git add src-tauri/crates/uc-core/src/ports/mod.rs
git add src-tauri/crates/uc-app/src/deps.rs
git add src-tauri/crates/uc-app/src/usecase_factory.rs
git add src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs
git commit -m "feat: implement encryption initialization flow

- Add FileEncryptionStateRepository for encryption state
- Add DefaultKeyScope for key scope management
- Add ports to AppDeps
- Wire in encryption infrastructure

Generated with [Claude Code](https://claude.ai/code)
via [Happy](https://happy.engineering)

Co-Authored-By: Claude <noreply@anthropic.com>
Co-Authored-By: Happy <yesreply@happy.engineering>"
```

---

## Task 5: Implement Encryption Commands

**Goal:** Wire encryption initialization command.

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/encryption.rs`

**Step 1: Implement initialize_encryption command**

Modify `src-tauri/crates/uc-tauri/src/commands/encryption.rs`:

```rust
use tauri::State;
use uc_app::{AppDeps, usecase_factory};
use uc_core::security::model::Passphrase;

/// Initialize encryption with passphrase
/// 使用密码短语初始化加密
#[tauri::command]
pub async fn initialize_encryption(
    deps: State<'_, AppDeps>,
    passphrase: String,
) -> Result<(), String> {
    let use_case = usecase_factory::create_initialize_encryption_use_case(&deps.inner());
    let passphrase = Passphrase::new(passphrase);

    use_case
        .execute(passphrase)
        .await
        .map_err(|e| format!("Failed to initialize encryption: {}", e))
}

/// Check if encryption is initialized
/// 检查加密是否已初始化
#[tauri::command]
pub async fn is_encryption_initialized(
    deps: State<'_, AppDeps>,
) -> Result<bool, String> {
    use uc_core::ports::EncryptionStatePort;

    deps.encryption_state
        .load_state()
        .await
        .map(|state| state == uc_core::security::state::EncryptionState::Initialized)
        .map_err(|e| format!("Failed to check encryption state: {}", e))
}
```

**Step 2: Verify compilation**

```bash
cd src-tauri && cargo check --package uc-tauri
```

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/encryption.rs
git commit -m "feat(uc-tauri): implement encryption commands

- Implement initialize_encryption command
- Implement is_encryption_initialized command
- Wire to use case factory

Generated with [Claude Code](https://claude.ai/code)
via [Happy](https://happy.engineering)

Co-Authored-By: Claude <noreply@anthropic.com>
Co-Authored-By: Happy <yesreply@happy.engineering>"
```

---

## Task 6: Register All Commands

**Goal:** Register all commands in Tauri invoke_handler.

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/lib.rs`

**Step 1: Update invoke_handler**

Modify `src-tauri/crates/uc-tauri/src/lib.rs`:

```rust
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Bootstrap application
            uc_tauri::bootstrap::run(app)
        })
        .invoke_handler(tauri::generate_handler![
            // Clipboard commands
            commands::clipboard::get_clipboard_entries,
            commands::clipboard::delete_clipboard_entry,
            commands::clipboard::capture_clipboard,

            // Encryption commands
            commands::encryption::initialize_encryption,
            commands::encryption::is_encryption_initialized,

            // Settings commands
            commands::settings::get_settings,
            commands::settings::update_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Step 2: Verify compilation**

```bash
cd src-tauri && cargo check --package uc-tauri
```

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/lib.rs
git commit -m "feat(uc-tauri): register all commands in invoke_handler

- Register clipboard commands
- Register encryption commands
- Register settings commands

Generated with [Claude Code](https://claude.ai/code)
via [Happy](https://happy.engineering)

Co-Authored-By: Claude <noreply@anthropic.com>
Co-Authored-By: Happy <yesreply@happy.engineering>"
```

---

## Task 7: Create Completion Documentation

**Files:**

- Create: `docs/plans/PHASE4_COMPLETE.md`

**Step 1: Create completion document**

Create `docs/plans/PHASE4_COMPLETE.md`:

```markdown
# Phase 4: Complete Feature Implementation ✅

**Date:** 2026-01-13
**Status:** COMPLETED

## Deliverables

### Use Case Factory

- ✅ Implemented create_capture_clipboard_use_case
- ✅ Implemented create_materialize_clipboard_selection_use_case
- ✅ Implemented create_initialize_encryption_use_case

### Command Wiring

- ✅ get_clipboard_entries wired to repository
- ✅ capture_clipboard wired to use case
- ✅ initialize_encryption wired to use case
- ✅ is_encryption_initialized wired to state repository

### Clipboard Monitoring

- ✅ Created ClipboardMonitor service
- ✅ Background capture loop implemented
- ✅ Event emission to frontend

### Encryption Flow

- ✅ FileEncryptionStateRepository implemented
- ✅ DefaultKeyScope implemented
- ✅ Ports added to AppDeps
- ✅ Commands wired and registered

## Next Steps

- Frontend TypeScript type definitions
- Frontend event listeners
- UI components for encryption setup
- Clipboard history UI

Generated with [Claude Code](https://claude.ai/code)
via [Happy](https://happy.engineering)
```

**Step 2: Commit**

```bash
git add docs/plans/PHASE4_COMPLETE.md
git commit -m "docs(plans): mark Phase 4 as complete

Use Case Factory:
- All use cases have factory functions
- Dependencies properly wired from AppDeps

Command Wiring:
- Commands connected to use cases
- Error handling implemented

Clipboard Monitoring:
- Background service created
- Event emission implemented

Encryption Flow:
- State repository implemented
- Key scope implemented
- Commands working

Generated with [Claude Code](https://claude.ai/code)
via [Happy](https://happy.engineering)

Co-Authored-By: Claude <noreply@anthropic.com>
Co-Authored-By: Happy <yesreply@happy.engineering>"
```

---

## Summary

### Phase 4 Tasks

| Task | Component                | Est. Time |
| ---- | ------------------------ | --------- |
| 1    | Use Case Factory         | 30 min    |
| 2    | Command Wiring           | 30 min    |
| 3    | Clipboard Monitor        | 40 min    |
| 4    | Encryption Flow          | 60 min    |
| 5    | Encryption Commands      | 20 min    |
| 6    | Register Commands        | 15 min    |
| 7    | Completion Documentation | 10 min    |

**Total Estimated Time:** ~3 hours

### Key Implementation Notes

1. **Incremental Implementation**: Each task builds on the previous
2. **Type Safety**: Maintain Arc<dyn Trait> pattern throughout
3. **Error Handling**: Convert domain errors to String for Tauri
4. **Event-Driven**: Use Tauri event system for background updates
5. **Factory Pattern**: Centralized use case creation for consistency

### Pre-Execution Checklist

Before starting execution:

- [ ] Verify Phase 3 is complete
- [ ] Confirm workspace is clean
- [ ] Review use case implementations
- [ ] Understand port dependencies
