# Onboarding Implementation Plan

## Overview

Implement onboarding flow following Hexagonal Architecture (Ports and Adapters) with complete migration to the new architecture.

**Status**: Design Approved
**Created**: 2025-01-15

## Architecture Diagram

```text
                ┌─────────────────────────────────────┐
                │         Frontend (React)            │
                │   OnboardingPage.tsx (已实现)        │
                └──────────────┬──────────────────────┘
                               │ Tauri Commands
                               ▼
                ┌─────────────────────────────────────┐
                │      uc-tauri (Commands Layer)      │
                │  commands/onboarding.rs (新建)       │
                └──────────────┬──────────────────────┘
                               │
                               ▼
                ┌─────────────────────────────────────┐
                │      uc-app (Application Layer)     │
                │  usecases/onboarding/               │
                │  ├── InitializeOnboardingUseCase    │
                │  ├── CompleteOnboardingUseCase      │
                │  └── GetOnboardingStateUseCase      │
                └──────────────┬──────────────────────┘
                               │
                               ▼
                ┌─────────────────────────────────────┐
                │      uc-core (Domain Layer)         │
                │  onboarding/mod.rs (新建)            │
                │  ports/onboarding.rs (新建)          │
                └─────────────────────────────────────┘
                               ▲
                               │
                ┌──────────────┴──────────────────────┐
                │      uc-infra (Infrastructure)      │
                │  onboarding_state.rs (新建)          │
                │  - FileOnboardingStateRepository    │
                └─────────────────────────────────────┘
```

## Use Cases

| Use Case | Description | Frontend Integration |
|----------|-------------|----------------------|
| `InitializeOnboardingUseCase` | Get initial onboarding state | `useInitializeOnboarding` hook |
| `GetOnboardingStateUseCase` | Get current state | Frontend polling |
| `CompleteOnboardingUseCase` | Mark onboarding complete | `OnboardingContext.complete()` |
| `InitializeEncryptionUseCase` (existing) | Initialize encryption password | `OnboardingContext.setPassword()` |

## Implementation Steps

### Phase 1: Domain Layer (uc-core)

**File: `uc-core/src/onboarding/mod.rs`** (新建)

```rust
/// Onboarding flow state
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OnboardingState {
    /// Whether onboarding has been completed
    pub has_completed: bool,
    /// Whether encryption password has been set
    pub encryption_password_set: bool,
    /// Whether device has been registered (auto-registered)
    pub device_registered: bool,
}

impl Default for OnboardingState {
    fn default() -> Self {
        Self {
            has_completed: false,
            encryption_password_set: false,
            device_registered: false,
        }
    }
}
```

**File: `uc-core/src/ports/onboarding.rs`** (新建)

```rust
use async_trait::async_trait;
use crate::onboarding::OnboardingState;

#[async_trait]
pub trait OnboardingStatePort: Send + Sync {
    /// Get current onboarding state
    async fn get_state(&self) -> anyhow::Result<OnboardingState>;

    /// Update onboarding state
    async fn set_state(&self, state: &OnboardingState) -> anyhow::Result<()>;

    /// Reset onboarding (for testing or re-onboarding)
    async fn reset(&self) -> anyhow::Result<()>;

    /// Check if onboarding is completed
    async fn is_completed(&self) -> anyhow::Result<bool> {
        Ok(self.get_state().await?.has_completed)
    }
}
```

**Modify: `uc-core/src/ports/mod.rs`**

```rust
// Add:
pub mod onboarding;
pub use onboarding::OnboardingStatePort;
```

**Modify: `uc-core/src/lib.rs`**

```rust
// Add:
pub mod onboarding;
```

---

### Phase 2: Infrastructure Layer (uc-infra)

**File: `uc-infra/src/onboarding_state.rs`** (新建)

```rust
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uc_core::ports::OnboardingStatePort;
use uc_core::onboarding::OnboardingState;

pub const DEFAULT_ONBOARDING_STATE_FILE: &str = ".onboarding_state";

pub struct FileOnboardingStateRepository {
    state_file_path: PathBuf,
}

impl FileOnboardingStateRepository {
    /// Create repository with custom file path
    pub fn new(state_file_path: PathBuf) -> Self {
        Self { state_file_path }
    }

    /// Create repository with base dir and filename
    pub fn with_base_dir(base_dir: PathBuf, filename: impl Into<String>) -> Self {
        Self {
            state_file_path: base_dir.join(filename.into()),
        }
    }

    /// Create repository with defaults
    pub fn with_defaults(base_dir: PathBuf) -> Self {
        Self {
            state_file_path: base_dir.join(DEFAULT_ONBOARDING_STATE_FILE),
        }
    }

    async fn ensure_parent_dir(&self) -> anyhow::Result<()> {
        if let Some(parent) = self.state_file_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl OnboardingStatePort for FileOnboardingStateRepository {
    async fn get_state(&self) -> anyhow::Result<OnboardingState> {
        if !self.state_file_path.exists() {
            return Ok(OnboardingState::default());
        }

        self.ensure_parent_dir().await?;
        let content = fs::read_to_string(&self.state_file_path).await?;

        if content.trim().is_empty() {
            return Ok(OnboardingState::default());
        }

        let state: OnboardingState = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse onboarding state: {}", e))?;

        Ok(state)
    }

    async fn set_state(&self, state: &OnboardingState) -> anyhow::Result<()> {
        self.ensure_parent_dir().await?;

        let json = serde_json::to_string_pretty(state)
            .map_err(|e| anyhow::anyhow!("Failed to serialize onboarding state: {}", e))?;

        let mut file = fs::File::create(&self.state_file_path).await
            .map_err(|e| anyhow::anyhow!("Failed to create state file: {}", e))?;

        file.write_all(json.as_bytes()).await
            .map_err(|e| anyhow::anyhow!("Failed to write state file: {}", e))?;

        file.sync_all().await
            .map_err(|e| anyhow::anyhow!("Failed to sync state file: {}", e))?;

        Ok(())
    }

    async fn reset(&self) -> anyhow::Result<()> {
        if self.state_file_path.exists() {
            fs::remove_file(&self.state_file_path).await?;
        }
        Ok(())
    }
}
```

**Modify: `uc-infra/src/lib.rs`**

```rust
pub mod onboarding_state;
pub use onboarding_state::FileOnboardingStateRepository, DEFAULT_ONBOARDING_STATE_FILE;
```

---

### Phase 3: Application Layer (uc-app)

**File: `uc-app/src/usecases/onboarding/mod.rs`** (新建)

```rust
pub mod initialize;
pub mod complete;
pub mod get_state;

pub use initialize::InitializeOnboardingUseCase;
pub use complete::CompleteOnboardingUseCase;
pub use get_state::GetOnboardingStateUseCase;

#[derive(Debug, Clone, serde::Serialize)]
pub struct OnboardingStateDto {
    pub has_completed: bool,
    pub encryption_password_set: bool,
    pub device_registered: bool,
}
```

**File: `uc-app/src/usecases/onboarding/initialize.rs`** (新建)

```rust
use crate::dependencies::ApplicationDeps;
use super::OnboardingStateDto;
use anyhow::Result;

pub struct InitializeOnboardingUseCase {
    deps: ApplicationDeps,
}

impl InitializeOnboardingUseCase {
    pub fn new(deps: ApplicationDeps) -> Self {
        Self { deps }
    }

    /// Get initial onboarding state
    /// Checks:
    /// - Whether onboarding is completed
    /// - Whether encryption password is initialized
    /// - Whether device is registered (auto-registered, always true)
    pub async fn execute(&self) -> Result<OnboardingStateDto> {
        let onboarding_state = self.deps.onboarding_state().get_state().await?;
        let encryption_initialized = self.deps.encryption_state()
            .is_initialized()
            .await
            .unwrap_or(false);

        // Device is auto-registered on app startup
        let device_registered = true;

        Ok(OnboardingStateDto {
            has_completed: onboarding_state.has_completed,
            encryption_password_set: encryption_initialized,
            device_registered,
        })
    }
}
```

**File: `uc-app/src/usecases/onboarding/complete.rs`** (新建)

```rust
use crate::dependencies::ApplicationDeps;
use uc_core::onboarding::OnboardingState;
use anyhow::Result;

pub struct CompleteOnboardingUseCase {
    deps: ApplicationDeps,
}

impl CompleteOnboardingUseCase {
    pub fn new(deps: ApplicationDeps) -> Self {
        Self { deps }
    }

    /// Mark onboarding as complete
    pub async fn execute(&self) -> Result<()> {
        let mut state = self.deps.onboarding_state().get_state().await?;
        state.has_completed = true;
        self.deps.onboarding_state().set_state(&state).await
    }
}
```

**File: `uc-app/src/usecases/onboarding/get_state.rs`** (新建)

```rust
use crate::dependencies::ApplicationDeps;
use super::OnboardingStateDto;
use anyhow::Result;

pub struct GetOnboardingStateUseCase {
    deps: ApplicationDeps,
}

impl GetOnboardingStateUseCase {
    pub fn new(deps: ApplicationDeps) -> Self {
        Self { deps }
    }

    /// Get current onboarding state
    pub async fn execute(&self) -> Result<OnboardingStateDto> {
        let onboarding_state = self.deps.onboarding_state().get_state().await?;
        let encryption_initialized = self.deps.encryption_state()
            .is_initialized()
            .await
            .unwrap_or(false);

        Ok(OnboardingStateDto {
            has_completed: onboarding_state.has_completed,
            encryption_password_set: encryption_initialized,
            device_registered: true,
        })
    }
}
```

**Modify: `uc-app/src/usecases/mod.rs`**

```rust
pub mod onboarding;
```

---

### Phase 4: Dependency Injection (uc-tauri)

**Modify: `src-tauri/crates/uc-tauri/src/bootstrap/dependencies.rs`**

```rust
use uc_core::ports::OnboardingStatePort;
use uc_infra::onboarding_state::FileOnboardingStateRepository;

#[derive(Clone)]
pub struct ApplicationDeps {
    // ... existing fields

    onboarding_state: Arc<dyn OnboardingStatePort>,
}

impl ApplicationDeps {
    // ... existing methods

    pub fn onboarding_state(&self) -> Arc<dyn OnboardingStatePort> {
        self.onboarding_state.clone()
    }
}

pub struct ApplicationDepsBuilder {
    // ... existing fields

    onboarding_state: Option<Arc<dyn OnboardingStatePort>>,
}

impl ApplicationDepsBuilder {
    pub fn new() -> Self {
        Self {
            // ... existing field initializations
            onboarding_state: None,
        }
    }

    // ... existing methods

    pub fn with_onboarding_state(
        mut self,
        repo: Arc<dyn OnboardingStatePort>,
    ) -> Self {
        self.onboarding_state = Some(repo);
        self
    }

    pub fn build(self) -> anyhow::Result<ApplicationDeps> {
        Ok(ApplicationDeps {
            // ... existing fields
            onboarding_state: self.onboarding_state
                .ok_or_else(|| anyhow::anyhow!("onboarding_state is required"))?,
        })
    }
}
```

**Modify: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`**

```rust
use uc_app::usecases::onboarding::{
    InitializeOnboardingUseCase,
    CompleteOnboardingUseCase,
    GetOnboardingStateUseCase,
};

impl UseCases {
    // ... existing methods

    pub fn initialize_onboarding(&self) -> InitializeOnboardingUseCase {
        InitializeOnboardingUseCase::new(self.deps.clone())
    }

    pub fn complete_onboarding(&self) -> CompleteOnboardingUseCase {
        CompleteOnboardingUseCase::new(self.deps.clone())
    }

    pub fn get_onboarding_state(&self) -> GetOnboardingStateUseCase {
        GetOnboardingStateUseCase::new(self.deps.clone())
    }
}
```

---

### Phase 5: Tauri Commands (uc-tauri)

**File: `src-tauri/crates/uc-tauri/src/commands/onboarding.rs`** (新建)

```rust
use tauri::State;
use crate::bootstrap::runtime::AppRuntime;
use uc_app::usecases::onboarding::OnboardingStateDto;

#[tauri::command]
pub async fn get_onboarding_state(
    runtime: State<'_, AppRuntime>,
) -> Result<OnboardingStateDto, String> {
    let uc = runtime.usecases().get_onboarding_state();
    uc.execute()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn complete_onboarding(
    runtime: State<'_, AppRuntime>,
) -> Result<(), String> {
    let uc = runtime.usecases().complete_onboarding();
    uc.execute()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn initialize_onboarding(
    runtime: State<'_, AppRuntime>,
) -> Result<OnboardingStateDto, String> {
    let uc = runtime.usecases().initialize_onboarding();
    uc.execute()
        .await
        .map_err(|e| e.to_string())
}
```

**Modify: `src-tauri/crates/uc-tauri/src/commands/mod.rs`**

```rust
pub mod onboarding;
```

**Modify: `src-tauri/src/main.rs`**

```rust
// In invoke_handler, use tauri::generate_handler! macro:
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    uc_tauri::commands::onboarding::get_onboarding_state,
    uc_tauri::commands::onboarding::complete_onboarding,
    uc_tauri::commands::onboarding::initialize_onboarding,
])
```

**Modify: `src-tauri/src/main.rs` (initialize repository)**

```rust
fn main() {
    // ... existing setup

    let onboarding_repo = Arc::new(FileOnboardingStateRepository::with_defaults(
        app.path().app_data_dir().expect("Failed to get app data dir"),
    )) as Arc<dyn OnboardingStatePort>;

    let deps = ApplicationDepsBuilder::new()
        // ... existing dependencies
        .with_onboarding_state(onboarding_repo)
        .build()
        .expect("Failed to build dependencies");

    // ... rest of setup
}
```

---

### Phase 6: Cleanup

**Remove: `src-tauri/src/onboarding.rs`** (legacy placeholder)

**Update: Frontend API bindings** (if needed)

The frontend `useOnboarding` hook should already be compatible with the new command names. Verify that:
- `get_onboarding_state` matches the expected API
- `complete_onboarding` matches the expected API
- `initialize_onboarding` matches the expected API

---

## Testing Strategy

1. **Unit Tests**: Test `FileOnboardingStateRepository` file operations
2. **Integration Tests**: Test use cases with mock dependencies
3. **E2E Tests**: Test full onboarding flow through Tauri commands

---

## Migration Notes

- Device registration is handled automatically on app startup (via `PlatformRuntime`)
- Encryption password initialization uses existing `InitializeEncryptionUseCase`
- Legacy onboarding code in `src-tauri/src-legacy/` can be removed after verification

---

## Checklist

- [ ] Phase 1: Domain Layer (uc-core)
  - [ ] Create `onboarding/mod.rs`
  - [ ] Create `ports/onboarding.rs`
  - [ ] Update `ports/mod.rs`
  - [ ] Update `lib.rs`

- [ ] Phase 2: Infrastructure Layer (uc-infra)
  - [ ] Create `onboarding_state.rs`
  - [ ] Update `lib.rs`

- [ ] Phase 3: Application Layer (uc-app)
  - [ ] Create `usecases/onboarding/mod.rs`
  - [ ] Create `usecases/onboarding/initialize.rs`
  - [ ] Create `usecases/onboarding/complete.rs`
  - [ ] Create `usecases/onboarding/get_state.rs`
  - [ ] Update `usecases/mod.rs`

- [ ] Phase 4: Dependency Injection (uc-tauri)
  - [ ] Update `bootstrap/dependencies.rs`
  - [ ] Update `bootstrap/runtime.rs`

- [ ] Phase 5: Tauri Commands (uc-tauri)
  - [ ] Create `commands/onboarding.rs`
  - [ ] Update `commands/mod.rs`
  - [ ] Update `src/main.rs`

- [ ] Phase 6: Cleanup
  - [ ] Remove `src-tauri/src/onboarding.rs`
  - [ ] Verify frontend integration
  - [ ] Test full onboarding flow
