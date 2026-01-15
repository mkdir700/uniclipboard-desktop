# Tracing Migration Design

## Overview

This document describes the migration from `log` crate to `tracing` crate for UniClipboard's logging infrastructure. The migration follows a **gradual approach** where `log` and `tracing` coexist during the transition period.

**Date**: 2025-01-15
**Status**: Design Approved
**Migration Strategy**: Gradual (log + tracing coexistence)

---

## Design Goals

This migration solves four key problems:

1. **Cross-layer traceability**: Tauri Command → Use Case → Domain → Infra → Platform
2. **Fact vs. decision separation**: System facts, policy choices, persistence, and side effects are clearly separated
3. **async/multi-thread stability**: Spans must work correctly across `await` and `spawn_blocking`
4. **Future-proof**: Ready for GUI/file/OpenTelemetry integration without being locked in

---

## Architecture Principles

### 1. Layer = Span (No Mixing)

| Layer                  | Can Create Span? | Description                      |
| ---------------------- | ---------------- | -------------------------------- |
| Tauri Command          | ✅ root span     | User intent entry point          |
| Use Case (uc-app)      | ✅ child span    | Business lifecycle               |
| Domain (uc-core)       | ❌ (events only) | Pure logic, no runtime semantics |
| Infra (uc-infra)       | ✅ child span    | I/O / DB / Crypto                |
| Platform (uc-platform) | ✅ child span    | OS / Clipboard                   |

**Principle**: _Domain has no concept of time, it only states facts._

### 2. Span = Lifecycle, Event = Fact

- **Span**: A "currently ongoing operation"
- **Event**: A "fact that has occurred"

Forbidden: Using `info!` to replace spans.

---

## Span Naming Conventions

### Naming Format

```
<layer>.<module>.<action>
```

Examples:

- `command.clipboard.capture`
- `usecase.capture_clipboard.execute`
- `infra.sqlite.insert_clipboard_event`
- `platform.macos.read_clipboard`

**Forbidden**:

- `do_something`
- `handle`
- `process`

### Field Naming Conventions (Strict)

| Type | Rule                                          |
| ---- | --------------------------------------------- |
| ID   | `<entity>_id` (entry_id / event_id / blob_id) |
| Time | `_ms` suffix (captured_at_ms)                 |
| Size | `_bytes`                                      |
| Enum | snake_case string                             |
| bool | `is_*`                                        |

Examples:

```rust
entry_id = %entry_id
total_size_bytes = total_size
is_encrypted = is_encrypted
```

---

## Layer Specifications

### Root Span: Command Layer (User Intent)

Every Tauri Command **must** create a root span:

```rust
#[tauri::command]
pub async fn capture_clipboard(
    state: State<'_, AppRuntime>,
) -> Result<(), String> {
    let span = tracing::info_span!(
        "command.clipboard.capture",
        device_id = %state.device_id(),
    );
    let _enter = span.enter();

    state.usecases()
        .capture_clipboard()
        .execute()
        .await
        .map_err(|e| e.to_string())
}
```

**Rules**:

- Command layer **only creates spans, no business events**
- Expresses **"what the user requested"**

---

### Use Case Span: Business Lifecycle (Core)

Every Use Case **must** be a complete span:

```rust
pub async fn execute(&self) -> Result<()> {
    let span = tracing::info_span!(
        "usecase.capture_clipboard.execute",
        policy_version = "v1",
    );
    let _enter = span.enter();

    // 1. Read system facts
    let snapshot = self.system_clipboard.read_snapshot()?;
    tracing::debug!(
        representations = snapshot.representations.len(),
        "system snapshot captured"
    );

    // 2. Select strategy
    let decision = self.selector.select(&snapshot);
    tracing::info!(
        primary_rep_id = %decision.primary,
        "representation selected"
    );

    // 3. Persist
    self.repo.save(...).await?;

    Ok(())
}
```

**Use Case span must cover**:

- All business steps
- All `await` points
- All infra/platform calls

---

### Infra Layer: Side Effects and Costs

Every "heavy I/O" **must** have a span:

#### SQLite Example

```rust
let span = tracing::debug_span!(
    "infra.sqlite.insert_clipboard_event",
    table = "clipboard_event",
);
let _enter = span.enter();

diesel::insert_into(...).execute(conn)?;
```

#### Blob/FS

```rust
let span = tracing::debug_span!(
    "infra.blob.write",
    blob_id = %blob_id,
    size_bytes = size,
);
```

**Rules**:

- Infra spans are `debug` or `trace` level
- Don't guess what infra is doing from Use Case

---

### Platform Layer: Source of System Facts

Platform is the "fact producer":

```rust
let span = tracing::debug_span!(
    "platform.macos.read_clipboard",
    formats = formats.len(),
);
```

Allowed to log:

- OS-returned formats
- Data sizes
- System error codes

Forbidden:

- Business meaning
- Strategy explanations

---

## Error Handling Standards

### Errors Must Be Bound to Spans

```rust
tracing::error!(
    error = %err,
    error.kind = "permission_denied",
    "failed to read clipboard"
);
```

### No Silent Failures

Forbidden: Only `map_err(|e| e.to_string())` without logging first.

**Correct pattern**:

```rust
match risky_operation().await {
    Ok(result) => {
        tracing::info!("operation succeeded");
        Ok(result)
    }
    Err(e) => {
        tracing::error!(
            error = %e,
            error.kind = "permission_denied",
            operation = "read_clipboard",
            "failed to read clipboard"
        );
        Err(e.into())
    }
}
```

---

## Recommended Span Tree (Capture Clipboard Example)

```
command.clipboard.capture
└─ usecase.capture_clipboard.execute
   ├─ platform.macos.read_clipboard
   │  └─ event: formats=3
   ├─ event: representation selected
   ├─ infra.sqlite.insert_clipboard_event
   ├─ infra.blob.write
   └─ event: capture completed
```

This is the **complete form** you should see when debugging.

---

## Migration Strategy

### Dual-Track Logging System (Transition Period)

```
┌─────────────────────────────────────────────────────────┐
│                    Logging Facade                       │
└─────────────────────────────────────────────────────────┘
         │                           │
         ▼                           ▼
┌────────────────┐          ┌────────────────┐
│  tauri-plugin  │          │  tracing       │
│  -log (log)    │          │  subscriber    │
└────────────────┘          └────────────────┘
         │                           │
         └───────────┬───────────────┘
                     ▼
            ┌────────────────┐
            │  Output Targets │
            │  - Webview      │
            │  - LogDir       │
            │  - Stdout       │
            └────────────────┘
```

### Migration Phases

**Phase 0: Infrastructure Setup**

- Configure tracing-subscriber
- Enable tracing's log feature (compatibility layer)
- Verify dual-track system works

**Phase 1: Command Layer (Root Span)**

- All Tauri commands create root span
- Verify span tree roots are correct

**Phase 2: UseCase Layer (Business Lifecycle)**

- All use cases create child span
- Cover execute() methods
- Verify span hierarchy

**Phase 3: Infra/Platform Layer (Side Effect Logging)**

- SQLite operations add debug span
- Blob store operations add debug span
- Platform clipboard operations add debug span
- Verify complete span tree

**Phase 4: Cleanup (Optional)**

- Remove log crate dependency
- Remove tauri-plugin-log
- Pure tracing architecture

---

## Dependency Configuration

### uc-tauri (Entry Layer)

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt", "chrono"] }
tracing-log = "0.2"

# Tauri plugins
tauri-plugin-log = "2"  # Keep for Webview writer during transition
```

### uc-app (Application Layer)

```toml
[dependencies]
tracing = "0.1"

# Remove in Phase 4:
# log = "0.4"
```

### uc-infra (Infrastructure Layer)

```toml
[dependencies]
tracing = "0.1"
```

### uc-platform (Platform Adapter Layer)

```toml
[dependencies]
tracing = "0.1"
```

### uc-core (Domain Layer)

```toml
[dependencies]
# Domain layer: optional dependency, only for recording events
tracing = { version = "0.1", optional = true }

[features]
default = ["tracing"]
logging = ["tracing"]  # Allow disabling logging (pure computation scenarios)
```

---

## Code Templates

### Command Layer Template

```rust
use tracing::info_span;

#[tauri::command]
pub async fn capture_clipboard(
    runtime: State<'_, AppRuntime>,
) -> Result<String, String> {
    let span = info_span!(
        "command.clipboard.capture",
        device_id = %runtime.device_id(),
    );
    let _enter = span.enter();

    runtime.usecases()
        .capture_clipboard()
        .execute()
        .await
        .map_err(|e| e.to_string())?;

    Ok("Captured".to_string())
}
```

### UseCase Layer Template

```rust
use tracing::{info_span, info, debug};

pub async fn execute(&self) -> Result<()> {
    let span = info_span!(
        "usecase.capture_clipboard.execute",
        policy_version = "v1",
    );
    let _enter = span.enter();

    // 1. Read snapshot (platform creates child span)
    let snapshot = self.clipboard.read_snapshot()?;
    debug!(
        representations = snapshot.representations.len(),
        "system snapshot captured"
    );

    // 2. Business decision (domain records event)
    let selection = self.select_primary(&snapshot);

    // 3. Persist (infra creates child span)
    self.persist(snapshot, selection).await?;

    info!("capture completed");
    Ok(())
}
```

### Infra Layer Template

```rust
use tracing::debug_span;

async fn save_entry(&self, entry: &ClipboardEntry) -> Result<()> {
    let span = debug_span!(
        "infra.sqlite.insert_clipboard_entry",
        table = "clipboard_entry",
        entry_id = %entry.id,
    );
    let _enter = span.enter();

    diesel::insert_into(clipboard_entry::table)
        .values(entry)
        .execute(&mut self.conn)?;

    Ok(())
}
```

### Platform Layer Template

```rust
use tracing::debug_span;

fn read_snapshot(&self) -> Result<SystemClipboardSnapshot> {
    let span = debug_span!(
        "platform.macos.read_clipboard",
    );
    let _enter = span.enter();

    // ... read clipboard ...

    debug!(
        formats = snapshot.representations.len(),
        total_size_bytes = total_size,
        "clipboard snapshot captured"
    );

    Ok(snapshot)
}
```

---

## Span Field Reference Table

| Layer    | Span Name Format                | Default Level | Required Fields                  |
| -------- | ------------------------------- | ------------- | -------------------------------- |
| Command  | `command.<module>.<action>`     | INFO          | `device_id`                      |
| UseCase  | `usecase.<name>.execute`        | INFO          | `policy_version`                 |
| Infra    | `infra.<component>.<operation>` | DEBUG         | `table`, `blob_id`, `size_bytes` |
| Platform | `platform.<os>.<operation>`     | DEBUG         | `formats`, `error.code`          |

---

## Validation

### Manual Verification (Development)

```bash
# Set RUST_LOG=trace to see complete span tree
RUST_LOG=trace bun tauri dev

# Expected output:
# 2025-01-15 10:30:45.123 INFO [command.clipboard.capture{device_id=abc123}]
#   2025-01-15 10:30:45.124 INFO [usecase.capture_clipboard.execute{policy_version=v1}]
#     2025-01-15 10:30:45.125 DEBUG [platform.macos.read_clipboard] formats=3
```

### Unit Test Verification

```rust
#[tokio::test]
async fn test_capture_clipboard_use_case() {
    let (collector, result) = tracing::subscriber::with_default(
        tracing_subscriber::registry().with(TestLayer::new()),
        || {
            use_case.execute().await
        },
    );

    assert!(result.is_ok());
    assert!(collector.has_span("usecase.capture_clipboard.execute"));
}
```

---

## Performance Considerations

| Operation   | Overhead    | Notes                               |
| ----------- | ----------- | ----------------------------------- |
| Create span | ~50-100ns   | Negligible                          |
| Enter span  | ~10-20ns    | Negligible                          |
| Log field   | Depends     | Use `%` for Display, `=` for simple |
| async span  | Almost zero | tracing optimized for async         |

### Optimization Tips

```rust
// ✅ Correct: Lightweight fields
tracing::info_span!(
    "command.clipboard.capture",
    device_id = %device_id,    // Display impl (cheap)
    timestamp_ms = now_ms,      // Integer (cheap)
)

// ❌ Avoid: Expensive computation
tracing::info_span!(
    "command.clipboard.capture",
    full_config = ?serde_json::to_string(&config).unwrap(),
)

// ✅ Improved: Compute only when needed
tracing::info_span!(
    "command.clipboard.capture",
    config_hash = %config.hash(),
)
```

---

## Risks and Mitigations

| Risk                   | Impact                 | Mitigation                              |
| ---------------------- | ---------------------- | --------------------------------------- |
| Webview output fails   | Can't see logs in dev  | Keep tauri-plugin-log as fallback       |
| Span leaks             | Memory usage increases | Regular span lifecycle review           |
| Performance regression | Slower response        | Benchmark tests, verify in release mode |
| Incomplete migration   | Mixed log/tracing      | Mandatory code review                   |

---

## Migration Checklist

**Phase 0: Infrastructure**

- [ ] Add `tracing` dependencies to all crates
- [ ] Create `src-tauri/crates/uc-tauri/src/bootstrap/tracing.rs`
- [ ] Implement `init_tracing_subscriber()`
- [ ] Initialize in `main.rs` (before tauri setup)
- [ ] Verify `tracing-log` bridge works
- [ ] Update `docs/architecture/logging-architecture.md`

**Phase 1: Command Layer**

- [ ] Migrate `commands/clipboard.rs`
- [ ] Migrate `commands/encryption.rs`
- [ ] Migrate `commands/settings.rs`
- [ ] Verify root spans visible in logs

**Phase 2: UseCase Layer**

- [ ] Migrate `uc-app/src/usecases/clipboard/*.rs`
- [ ] Migrate `uc-app/src/usecases/encryption/*.rs`
- [ ] Verify span hierarchy correct

**Phase 3: Infra/Platform Layer**

- [ ] Migrate `uc-infra/src/db/repositories/*.rs`
- [ ] Migrate `uc-infra/src/blob/*.rs`
- [ ] Migrate `uc-platform/src/adapters/**/*.rs`
- [ ] Verify complete span tree

**Phase 4: Cleanup (Optional)**

- [ ] Remove `log` dependencies
- [ ] Remove `tauri-plugin-log`
- [ ] Update documentation

---

## One-Line Summary

> **Command defines intent, Use Case manages lifecycle, Platform provides facts, Infra records costs, Domain stays silent, it only computes.**

---

## References

- [tracing crate documentation](https://docs.rs/tracing/)
- [tracing-subscriber documentation](https://docs.rs/tracing-subscriber/)
- [Tauri Plugin Log](https://v2.tauri.app/plugin/logging/)
- Current logging architecture: `docs/architecture/logging-architecture.md`
