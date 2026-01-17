# Watcher Start After Encryption Design

> **Goal:** Enable clipboard watcher to start immediately after user sets encryption password, without requiring app restart.

## Problem Statement

Currently, after a user sets their encryption password for the first time via onboarding, they must restart the application for the clipboard watcher to start. This is a poor user experience.

### Root Causes

1. **PlatformRuntime only starts during app initialization**: The `PlatformRuntime` with clipboard watcher is spawned once in `main.rs` setup block
2. **No in-process watcher control**: There's no mechanism for Command layer to trigger watcher start after initialization

## Solution Overview

Introduce a `WatcherControlPort` following Hexagonal Architecture principles:

1. **Port definition** in `uc-core` - Abstract interface for watcher lifecycle control
2. **Platform implementation** - `InMemoryWatcherControl` using existing `mpsc` channel
3. **Wiring integration** - Inject through dependency injection in `wiring.rs`
4. **Command usage** - Call `start_watcher()` after successful encryption initialization

---

## Architecture

### Component Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                          Command Layer                           │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  initialize_encryption command                             │ │
│  │  1. Execute InitializeEncryption use case                   │ │
│  │  2. Call watcher_control.start_watcher().await              │ │
│  │  3. Emit success/error events to frontend                   │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                           Core Layer                             │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  WatcherControlPort (trait)                                │ │
│  │  - async fn start_watcher() -> Result<(), Error>            │ │
│  │  - async fn stop_watcher() -> Result<(), Error>             │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         Platform Layer                           │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  InMemoryWatcherControl                                    │ │
│  │  - Holds mpsc::Sender<PlatformCommand>                     │ │
│  │  - Sends StartClipboardWatcher command to channel          │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        PlatformRuntime                           │
│  Receives StartClipboardWatcher command → starts watching       │
└─────────────────────────────────────────────────────────────────┘
```

### Data Flow

```
User enters password
        │
        ▼
Frontend calls initialize_encryption(passphrase)
        │
        ▼
Command executes InitializeEncryption use case
        │
        ├─→ Generate MasterKey
        ├─→ Persist keyslot + KEK
        ├─→ Set encryption_session
        │
        ▼ (success)
Command calls watcher_control.start_watcher()
        │
        ▼
InMemoryWatcherControl sends StartClipboardWatcher
        │
        ▼
PlatformRuntime receives command → starts clipboard monitoring
        │
        ▼
Emit onboarding-password-set event to frontend
        │
        ▼
Frontend shows success, watcher is now running
```

---

## Implementation Details

### 1. Port Definition

**File:** `src-tauri/crates/uc-core/src/ports/watcher_control.rs`

```rust
use async_trait::async_trait;

/// Port for controlling the clipboard watcher lifecycle.
#[async_trait]
pub trait WatcherControlPort: Send + Sync {
    /// Request the clipboard watcher to start.
    async fn start_watcher(&self) -> Result<(), WatcherControlError>;

    /// Request the clipboard watcher to stop.
    async fn stop_watcher(&self) -> Result<(), WatcherControlError>;
}

#[derive(Debug, thiserror::Error)]
pub enum WatcherControlError {
    #[error("Failed to send start command: {0}")]
    StartFailed(String),

    #[error("Failed to send stop command: {0}")]
    StopFailed(String),

    #[error("Watcher channel closed")]
    ChannelClosed,
}
```

---

### 2. Platform Implementation

**File:** `src-tauri/crates/uc-platform/src/adapters/in_memory_watcher_control.rs`

```rust
use std::sync::Arc;
use tokio::sync::mpsc;
use uc_core::ports::watcher_control::{WatcherControlPort, WatcherControlError};
use crate::ipc::PlatformCommand;

pub struct InMemoryWatcherControl {
    cmd_tx: mpsc::Sender<PlatformCommand>,
}

impl InMemoryWatcherControl {
    pub fn new(cmd_tx: mpsc::Sender<PlatformCommand>) -> Self {
        Self { cmd_tx }
    }
}

#[async_trait::async_trait]
impl WatcherControlPort for InMemoryWatcherControl {
    async fn start_watcher(&self) -> Result<(), WatcherControlError> {
        self.cmd_tx
            .send(PlatformCommand::StartClipboardWatcher)
            .await
            .map_err(|e| WatcherControlError::StartFailed(e.to_string()))?;
        Ok(())
    }

    async fn stop_watcher(&self) -> Result<(), WatcherControlError> {
        self.cmd_tx
            .send(PlatformCommand::StopClipboardWatcher)
            .await
            .map_err(|e| WatcherControlError::StopFailed(e.to_string()))?;
        Ok(())
    }
}
```

---

### 3. Wiring Changes

**File:** `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`

#### Add to `PlatformLayer` struct:

```rust
struct PlatformLayer {
    // ... existing fields ...
    watcher_control: Arc<dyn WatcherControlPort>,
}
```

#### Update `create_platform_layer` signature:

```rust
fn create_platform_layer(
    keyring: Arc<dyn KeyringPort>,
    config_dir: &PathBuf,
    cmd_tx: mpsc::Sender<PlatformCommand>,  // NEW parameter
) -> WiringResult<PlatformLayer> {
    // ... existing code ...

    let watcher_control: Arc<dyn WatcherControlPort> =
        Arc::new(InMemoryWatcherControl::new(cmd_tx));

    Ok(PlatformLayer {
        // ... existing fields ...
        watcher_control,
    })
}
```

#### Update `AppDeps` (in `uc-app/src/lib.rs`):

```rust
pub struct AppDeps {
    // ... existing fields ...

    /// Watcher control for starting/stopping clipboard monitoring
    pub watcher_control: Arc<dyn WatcherControlPort>,
}
```

#### Update `wire_dependencies` return:

```rust
let deps = AppDeps {
    // ... existing fields ...
    watcher_control: platform.watcher_control,
};
```

---

### 4. Command Layer Changes

**File:** `src-tauri/crates/uc-tauri/src/commands/encryption.rs`

```rust
#[tauri::command]
pub async fn initialize_encryption(
    runtime: State<'_, Arc<AppRuntime>>,
    app_handle: AppHandle,
    passphrase: String,
) -> Result<(), String> {
    let span = info_span!(
        "command.encryption.initialize",
        device_id = %runtime.deps.device_identity.current_device_id(),
    );

    let uc = runtime.usecases().initialize_encryption();

    uc.execute(uc_core::security::model::Passphrase(passphrase))
        .instrument(span)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to initialize encryption");
            e.to_string()
        })?;

    tracing::info!("Encryption initialized successfully");

    // === NEW: Start clipboard watcher ===
    match runtime.deps.watcher_control.start_watcher().await {
        Ok(_) => {
            tracing::info!("Clipboard watcher started successfully after encryption initialization");
        }
        Err(e) => {
            tracing::error!("Failed to start clipboard watcher: {}", e);
            let _ = app_handle
                .emit("encryption://watcher-start-failed", format!("{}", e))
                .await;
            return Err(format!(
                "Encryption initialized, but failed to start clipboard watcher: {}", e
            ));
        }
    }

    // Emit success event
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| format!("Failed to get timestamp: {}", e))?
        .as_millis() as u64;

    let event = OnboardingPasswordSetEvent { timestamp };
    app_handle
        .emit("onboarding-password-set", event)
        .map_err(|e| format!("Failed to emit event: {}", e))?;

    tracing::info!("Onboarding: encryption initialized, clipboard watcher running");

    Ok(())
}
```

---

### 5. main.rs Changes

**File:** `src-tauri/src/main.rs`

The key change is ensuring `platform_cmd_tx` is passed to `wire_dependencies`:

```rust
fn run_app(config: AppConfig) {
    use tauri::Builder;

    // Wire dependencies - now needs platform_cmd_tx
    let deps = match wire_dependencies(&config, platform_cmd_tx.clone()) {
        Ok(deps) => deps,
        Err(e) => {
            error!("Failed to wire dependencies: {}", e);
            panic!("Dependency wiring failed: {}", e);
        }
    };

    // ... rest of setup remains unchanged ...
}
```

---

## Error Handling

### Failure Scenarios

| Scenario                        | Handling                                                                 |
| ------------------------------- | ------------------------------------------------------------------------ |
| Encryption initialization fails | Return error to frontend immediately, no watcher start attempt           |
| Watcher start succeeds          | Emit success event, log info                                             |
| Watcher channel closed          | Emit `encryption://watcher-start-failed` event, return error to frontend |
| Auto-unlock on startup fails    | Don't start watcher, log error, user must retry                          |

### Frontend Error Handling

```typescript
// Listen for watcher start failures
useEffect(() => {
  const unlisten = listen('encryption://watcher-start-failed', event => {
    toast.error({
      title: '剪贴板监控器启动失败',
      description: event.payload as string,
    })
  })
  return unlisten
}, [])
```

---

## Testing

### Unit Tests

**File:** `uc-platform/tests/watcher_control_test.rs`

```rust
#[tokio::test]
async fn test_start_watcher_sends_command() {
    let (cmd_tx, mut cmd_rx) = mpsc::channel(10);
    let control = InMemoryWatcherControl::new(cmd_tx);

    control.start_watcher().await.unwrap();

    let received = cmd_rx.recv().await.unwrap();
    assert!(matches!(received, PlatformCommand::StartClipboardWatcher));
}

#[tokio::test]
async fn test_stop_watcher_sends_command() {
    let (cmd_tx, mut cmd_rx) = mpsc::channel(10);
    let control = InMemoryWatcherControl::new(cmd_tx);

    control.stop_watcher().await.unwrap();

    let received = cmd_rx.recv().await.unwrap();
    assert!(matches!(received, PlatformCommand::StopClipboardWatcher));
}

#[tokio::test]
async fn test_start_watcher_channel_closed() {
    let (cmd_tx, _) = mpsc::channel(1);
    let control = InMemoryWatcherControl::new(cmd_tx);
    drop(cmd_tx);

    let result = control.start_watcher().await;
    assert!(result.is_err());
}
```

### Integration Testing Scenarios

| Scenario                    | Expected Behavior                               |
| --------------------------- | ----------------------------------------------- |
| First-time password set     | Watcher starts automatically, no restart needed |
| App restart with encryption | Auto-unlock succeeds, watcher starts            |
| Watcher start failure       | Frontend receives error, graceful degradation   |
| Uninitialized app startup   | No watcher start, onboarding flow shown         |

---

## Implementation Checklist

### New Files

- [ ] `uc-core/src/ports/watcher_control.rs`
- [ ] `uc-core/src/ports/mod.rs` - Export `watcher_control` module
- [ ] `uc-platform/src/adapters/in_memory_watcher_control.rs`
- [ ] `uc-platform/src/adapters/mod.rs` - Export `InMemoryWatcherControl`
- [ ] `uc-platform/tests/watcher_control_test.rs`

### Modified Files

- [ ] `uc-app/src/lib.rs` - Add `watcher_control` to `AppDeps`
- [ ] `uc-tauri/src/bootstrap/wiring.rs` - Wire `WatcherControlPort`
- [ ] `uc-tauri/src/commands/encryption.rs` - Call `start_watcher()` after init
- [ ] `src-tauri/src/main.rs` - Pass `platform_cmd_tx` to wiring

---

## Acceptance Criteria

- [ ] User sets password → watcher starts immediately
- [ ] No app restart required after first-time setup
- [ ] Watcher start failures emit error event to frontend
- [ ] Auto-unlock on startup still works as before
- [ ] All existing tests pass
- [ ] New unit tests for `InMemoryWatcherControl` pass

---

## Architectural Benefits

1. **Hexagonal Architecture compliance**: Platform functionality accessed through Port
2. **Testability**: Easy to mock `WatcherControlPort` for testing
3. **Separation of concerns**: Command layer orchestrates, Platform layer executes
4. **Future extensibility**: Easy to add other watcher control operations (pause, resume, status check)
