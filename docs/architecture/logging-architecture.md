# Logging Architecture

## Overview

UniClipboard uses **`tracing`** crate as the primary logging framework with structured logging and span-based context tracking. The system runs a **dual-track setup** during the transition from legacy `log` crate to `tracing`.

**Current Status**: Phases 0-3 complete, actively using `tracing` across all architectural layers.

## Architecture

### Primary Logging Framework: `tracing`

The application uses `tracing` crate for structured, span-aware logging:

**✅ Supported Features**:

- **Spans** - Structured context spans with parent-child relationships
- **Structured fields** - Field-based logging with typed values
- **Span hierarchy** - Cross-layer traceability
- **Instrumentation** - `.instrument()` for async operations
- **Event logging** - `tracing::info!`, `tracing::error!`, etc.

**Migration Status**:

| Phase   | Description                                             | Status          |
| ------- | ------------------------------------------------------- | --------------- |
| Phase 0 | Infrastructure setup (tracing dependencies, subscriber) | ✅ Complete     |
| Phase 1 | Command layer root spans                                | ✅ Complete     |
| Phase 2 | UseCase layer child spans                               | ✅ Complete     |
| Phase 3 | Infra/Platform layer debug spans                        | ✅ Complete     |
| Phase 4 | Remove `log` dependency (optional)                      | ⏸️ Not required |

### Dual-Track System

During the transition, both `log` and `tracing` coexist:

```rust
// Legacy code (still works via tauri-plugin-log)
log::info!("Application started");

// New code (preferred)
tracing::info!("Application started");
tracing::info_span!("command.clipboard.capture", device_id = %id);
```

**Note**: `tracing-log` bridge is NOT configured. The two systems operate independently:

- `log::` macros → `tauri-plugin-log` → Webview (dev) / file (prod)
- `tracing::` macros → `tracing-subscriber` → stdout

### Module Organization

#### 1. Bootstrap Configuration

**Location**: `src-tauri/crates/uc-tauri/src/bootstrap/`

```
bootstrap/
├── logging.rs       # tauri-plugin-log configuration (legacy)
└── tracing.rs       # tracing-subscriber configuration (primary)
```

**Initialization Flow**:

```
main.rs
  ├─> init_tracing_subscriber()     // Global tracing registry
  │    └─> All tracing::* macros now produce output
  │
  └─> Builder::default()
       └─> .plugin(logging::get_builder().build())
            └─> Legacy log::* macros still work
```

#### 2. Layer-Based Tracing

Each architectural layer has specific span naming conventions:

**Command Layer** (`uc-tauri/src/commands/`):

- Root spans for Tauri commands
- Naming: `command.{module}.{action}`
- Example: `command.clipboard.get_entries`, `command.encryption.initialize`

**UseCase Layer** (`uc-app/src/usecases/`):

- Business logic spans
- Naming: `usecase.{usecase_name}.{method}`
- Example: `usecase.list_clipboard_entries.execute`

**Infrastructure Layer** (`uc-infra/src/`):

- Database and repository operations
- Naming: `infra.{component}.{operation}`
- Example: `infra.sqlite.insert_clipboard_event`, `infra.blob.materialize`

**Platform Layer** (`uc-platform/src/`):

- Platform-specific operations
- Naming: `platform.{module}.{operation}`
- Example: `platform.linux.read_clipboard`, `platform.encryption.set_master_key`

## Configuration

### Development Mode

When `debug_assertions` is true (debug builds):

**tracing-subscriber**:

- **Level**: `Debug`
- **Targets**: `uc_platform=debug`, `uc_infra=debug`
- **Output**: `stdout` (terminal)
- **Filter**: `libp2p_mdns=warn`

**tauri-plugin-log**:

- **Level**: `Debug`
- **Target**: `Webview` (browser DevTools console)
- **Filters**: Tauri internals, wry noise

### Production Mode

When `debug_assertions` is false (release builds):

**tracing-subscriber**:

- **Level**: `Info`
- **Targets**: `uc_platform=info`, `uc_infra=info`
- **Output**: `stdout`
- **Filter**: `libp2p_mdns=warn`

**tauri-plugin-log**:

- **Level**: `Info`
- **Targets**: `LogDir` (file) + `Stdout`
- **Filters**: Tauri internals, wry noise, `ipc::request`

### Environment Variables

Override defaults with `RUST_LOG`:

```bash
# Enable trace for specific module
RUST_LOG=uc_platform::clipboard=trace,bun tauri dev

# Enable all debug logs
RUST_LOG=debug,bun tauri dev
```

### Log Format

Both systems output compatible formats:

```
YYYY-MM-DD HH:MM:SS.mmm LEVEL [file:line] [module] message
```

Example:

```
2025-01-15 10:30:45.123 INFO [clipboard.rs:51] [command.clipboard.get_entries] Fetching entries
2025-01-15 10:30:45.456 ERROR [clipboard.rs:52] [platform.linux.read_clipboard] Failed to read clipboard: NotFound
```

### Color Coding

- ERROR: Red (bold)
- WARN: Yellow
- INFO: Green
- DEBUG: Blue
- TRACE: Cyan

## Usage Patterns

### Basic Logging

```rust
use tracing::{info, error, warn, debug, trace};

pub fn process_clipboard(content: String) {
    debug!("Processing clipboard content: {} bytes", content.len());

    match parse(&content) {
        Ok(data) => info!("Successfully parsed clipboard data"),
        Err(e) => error!("Failed to parse clipboard: {}", e),
    }
}
```

### Span Creation

```rust
use tracing::info_span;

// Create span with fields
let span = info_span!(
    "command.clipboard.capture",
    device_id = %device.id,
    limit = limit,
    offset = offset
);

// Use with async operation
async move {
    // ... operation logic
}.instrument(span).await
```

### Structured Fields

Add context to spans with typed fields:

```rust
use tracing::{info_span, debug_span};

// Command layer - user-facing spans
info_span!(
    "command.encryption.initialize",
    passphrase_hash = %hash,
    salt_length = salt.len()
)

// Infra layer - debug spans
debug_span!(
    "infra.sqlite.insert",
    table = "clipboard_entries",
    entry_id = %id
)
```

### Span Hierarchy

Spans automatically form parent-child relationships:

```
command.clipboard.get_entries{device_id=abc123}
└─ usecase.list_clipboard_entries.execute{limit=50, offset=0}
   ├─ infra.sqlite.fetch_entries{sql="SELECT..."}
   └─ event: returning 42 entries
```

### Instrumentation Pattern

Standard pattern for async operations:

```rust
use tracing::{info_span, Instrument};
use tracing::debug_span;

// For async operations
pub async fn execute(&self, params: Params) -> Result<()> {
    let span = info_span!(
        "usecase.example.execute",
        param1 = %params.param1,
        param2 = params.param2
    );

    async move {
        // Business logic here
        self.inner_operation().await?;
        Ok(())
    }.instrument(span).await
}

// For debug-level operations (only in debug builds)
#[cfg(debug_assertions)]
fn debug_operation(&self) {
    let span = debug_span!("platform.debug.operation");
    span.in_scope(|| {
        // Debug logic here
    });
}
```

### Error Logging with Context

```rust
use tracing::error;

match risky_operation().await {
    Ok(result) => {
        tracing::info!("Operation succeeded");
    }
    Err(e) => {
        error!(
            error = %e,
            context = "failed to process clipboard",
            "Operation failed: {}", e
        );
    }
}
```

## Span Naming Conventions

### Standard Format

```
{layer}.{module}.{operation}
```

### Layer Prefixes

| Prefix      | Usage                        | Examples                            |
| ----------- | ---------------------------- | ----------------------------------- |
| `command.`  | Tauri command handlers       | `command.clipboard.get_entries`     |
| `usecase.`  | UseCase business logic       | `usecase.capture_clipboard.execute` |
| `infra.`    | Infrastructure (DB, storage) | `infra.sqlite.insert_blob`          |
| `platform.` | Platform adapters            | `platform.macos.read_clipboard`     |

### Field Naming

- **Use snake_case** for field names
- **Use `%` formatting** for types implementing `Display`
- **Use `?` formatting** for types implementing `Debug`

```rust
// Display formatting (cleaner output)
device_id = %device.id

// Debug formatting (detailed output)
config = ?config.options

// Direct values
count = 42
```

## Filtering

### Noise Reduction

**libp2p_mdns**:

- Set to `LevelFilter::Warn` to avoid spam from harmless mDNS errors
- Caused by proxy software virtual network interfaces

**Tauri Internal Events** (tauri-plugin-log only):

- Filtered to prevent infinite loops with Webview target
- `tauri::*` modules
- `tracing::*` modules
- `tauri-` prefixed modules
- `wry::*` modules

**IPC Request Logs**:

- Development: Enabled for debugging
- Production: Filtered to reduce verbosity

## Viewing Logs

### Development

**Terminal (tracing output)**:

```bash
bun tauri dev
# tracing::* macros appear here
```

**Browser DevTools (log output)**:

1. Open app in development mode
2. Press F12 or right-click → Inspect
3. Go to Console tab
4. `log::*` macros appear here

### Production

**Terminal**:

```bash
# Run the application
./uniclipboard

# tracing::* output appears in terminal
# log::* output appears in log file
```

**Log file**:

```bash
# macOS
tail -f ~/Library/Logs/com.uniclipboard/uniclipboard.log

# Linux
tail -f ~/.local/share/com.uniclipboard/logs/uniclipboard.log

# Windows (PowerShell)
Get-Content "$env:LOCALAPPDATA\com.uniclipboard\logs\uniclipboard.log" -Wait
```

**Filter for errors**:

```bash
grep ERROR ~/Library/Logs/com.uniclipboard/uniclipboard.log
```

**View last 100 lines**:

```bash
tail -n 100 ~/Library/Logs/com.uniclipboard/uniclipboard.log
```

## Testing

### Unit Tests

The tracing module includes basic tests:

```rust
#[test]
fn test_tracing_init() {
    let is_dev = is_development();
    let _ = is_dev;
}
```

Run with: `cd src-tauri && cargo test --package uc-tauri`

### Manual Testing

1. **Development**: Run `bun tauri dev` and check:
   - Terminal for `tracing::*` output
   - Browser DevTools for `log::*` output
2. **Production**: Build and run, check:
   - Log file exists and contains entries
   - Terminal shows `tracing::*` output
3. **Level filtering**: Verify DEBUG logs appear in dev but not in production

## Troubleshooting

### No logs appearing

**Check tracing initialization**:

1. Verify `main.rs` calls `init_tracing_subscriber()` before any logging
2. Check `tracing` dependency is present
3. Ensure you're using `tracing::info!` not `println!`

**Check log plugin**:

1. Verify `main.rs` has `.plugin(logging::get_builder().build())`
2. Check `log` crate dependency is present

### Logs not appearing in browser

1. Check Webview target is enabled in `logging.rs` for development mode
2. Open browser DevTools and check Console tab
3. Verify there are no JavaScript errors preventing log display

### Log file not created

1. Check app has write permissions to log directory
2. Verify LogDir target is enabled in production mode (`logging.rs`)
3. Check platform-specific log directory path

### Span hierarchy not visible

1. Ensure spans are created with `info_span!` or `debug_span!`
2. Verify `.instrument(span)` is used for async operations
3. Check that parent spans are not closed before child operations complete

## Migration Guide

### Adding Tracing to New Code

**1. Import tracing**:

```rust
use tracing::{info_span, info, Instrument};
```

**2. Create span for operations**:

```rust
let span = info_span!(
    "layer.module.operation",
    field1 = %value1,
    field2 = value2
);
```

**3. Instrument async operations**:

```rust
async move {
    // operation
}.instrument(span).await
```

### Converting Legacy Code

**Before** (log crate):

```rust
use log::info;

pub async fn get_entries(&self) -> Result<Vec<Entry>> {
    info!("Fetching entries");
    // ...
}
```

**After** (tracing crate):

```rust
use tracing::{info_span, info, Instrument};

pub async fn get_entries(&self) -> Result<Vec<Entry>> {
    let span = info_span!("usecase.get_entries.execute");
    async move {
        info!("Fetching entries");
        // ...
    }.instrument(span).await
}
```

## Best Practices

### DO ✅

- **Use spans for operations**: Every usecase/command should have a span
- **Add structured fields**: Include operation parameters as span fields
- **Follow naming conventions**: Use `{layer}.{module}.{operation}` format
- **Use appropriate log levels**: `error!`, `warn!`, `info!`, `debug!`, `trace!`
- **Instrument async operations**: Use `.instrument(span)` for async functions
- **Add context to errors**: Include error details and context in error logs

### DON'T ❌

- **Don't use `log::*` in new code**: Prefer `tracing::*` macros
- **Don't create spans for trivial operations**: Spans should represent meaningful work
- **Don't mix formatting styles**: Be consistent with field formatting
- **Don't forget to close spans**: Spans end when their scope ends
- **Don't use `unwrap()` in spans**: Handle errors explicitly

## Performance Considerations

### Span Creation Overhead

- Spans are **cheap** to create but not free
- Use `debug_span!` for operations that should only be traced in debug builds
- Avoid creating spans in tight loops

### Field Formatting

- **`%` formatting** (Display): Faster, cleaner output
- **`?` formatting** (Debug): Slower, detailed output
- Use `%` for production-critical fields
- Use `?` for development-only fields

### Level Filtering

- Spans below the configured level are **not created** (zero overhead)
- Set appropriate levels for each layer
- Use environment-specific filtering in production

## References

- [Tracing Crate Documentation](https://docs.rs/tracing/)
- [Tracing Subscriber Documentation](https://docs.rs/tracing-subscriber/)
- [Tauri Plugin Log Documentation](https://v2.tauri.app/plugin/logging/)
- Source:
  - `src-tauri/crates/uc-tauri/src/bootstrap/tracing.rs`
  - `src-tauri/crates/uc-tauri/src/bootstrap/logging.rs`
- Guides:
  - [Tracing Usage Guide](../guides/tracing.md)
  - [Coding Standards](../guides/coding-standards.md)
