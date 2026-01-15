# Logging Architecture

## Overview

UniClipboard uses the `tauri-plugin-log` plugin for configurable, cross-platform logging.
This document describes the logging architecture, configuration, and usage patterns.

## Architecture

### Logging Framework

**Current Implementation: `log` crate (NOT `tracing`)**

The application uses the standard `log` crate for logging, which provides simple level-based logging (error, warn, info, debug, trace).

**❌ Does NOT support:**

- **Spans** - No structured context spans like `tracing::info_span!`
- **Structured fields** - No field-based structured logging
- **Distributed tracing** - No trace ID propagation
- **Span relationships** - No parent-child span relationships

**✅ Does support:**

- Simple level-based logging (`log::error!`, `log::info!`, etc.)
- Timestamp formatting
- Color-coded console output
- File/line/module information
- Environment-based filtering

**Why `log` instead of `tracing`?**

1. `tauri-plugin-log` is built on `log` crate
2. Simpler integration for Tauri apps
3. Sufficient for current debugging needs
4. Lower overhead than `tracing`

**Future migration to `tracing`:**

If span-based tracing becomes necessary (e.g., for distributed systems, complex request flows), consider:

1. Switch to `tauri-plugin-tracing` or custom `tracing` subscriber
2. Replace `log::info!` with `tracing::info!` macros
3. Add spans for request/context tracking
4. Integrate with OpenTelemetry for distributed tracing

### Module Location

The logging configuration is centralized in the Tauri integration layer:

```
src-tauri/crates/uc-tauri/src/bootstrap/logging.rs
```

This location was chosen because:

1. Logging is Tauri-specific infrastructure code
2. The bootstrap module handles app initialization concerns
3. It's not domain logic (not uc-core)
4. It's not a use case (not uc-app)

### Initialization Flow

```
main.rs
  └─> run_app()
       └─> Builder::default()
            └─> .plugin(logging::get_builder().build())
                 └─> All log::info! macros now produce output
```

## Configuration

### Development Mode

When `debug_assertions` is true (debug builds):

- **Level**: `Debug`
- **Target**: `Webview` (browser DevTools console)
- **Format**: Colored with timestamps
- **Filters**: Basic noise only

### Production Mode

When `debug_assertions` is false (release builds):

- **Level**: `Info`
- **Targets**: `LogDir` (file) + `Stdout` (terminal)
- **Format**: Colored with timestamps
- **Filters**: Basic noise + `ipc::request`

### Log Format

```
YYYY-MM-DD HH:MM:SS.mmm LEVEL [file:line] [module] message
```

Example:

```
2025-01-15 10:30:45.123 INFO [main.rs:140] [uniclipboard] Creating platform runtime with clipboard callback
2025-01-15 10:30:45.456 ERROR [clipboard.rs:52] [uc_platform::clipboard] Failed to read clipboard: NotFound
```

### Color Coding

- ERROR: Red (bold)
- WARN: Yellow
- INFO: Green
- DEBUG: Blue
- TRACE: Cyan

## Filtering

The logging system filters out noise from various sources:

### libp2p_mdns

Set to `LevelFilter::Warn` to avoid spam from harmless mDNS errors
caused by proxy software virtual network interfaces.

### Tauri Internal Events

Filtered to prevent infinite loops when using Webview target:

- `tauri::*` modules
- `tracing::*` modules
- `tauri-` prefixed modules
- `wry::*` modules (underlying WebView library)

### IPC Request Logs

In production, `ipc::request` logs are filtered to reduce verbosity.
In development, they're kept for debugging frontend-backend communication.

## Usage

### Basic Logging

```rust
use log::{error, warn, info, debug, trace};

pub fn process_clipboard(content: String) {
    debug!("Processing clipboard content: {} bytes", content.len());

    match parse(&content) {
        Ok(data) => info!("Successfully parsed clipboard data"),
        Err(e) => error!("Failed to parse clipboard: {}", e),
    }
}
```

### Structured Logging

For complex data, use formatting:

```rust
log::info!(
    "Device connected: id={}, name={}, platform={}",
    device.id,
    device.name,
    device.platform
);
```

### Error Logging with Context

Always include error context for better debugging:

```rust
match risky_operation().await {
    Ok(result) => log::info!("Operation succeeded"),
    Err(e) => log::error!("Operation failed: {} (context: {})", e, additional_context),
}
```

## Viewing Logs

### Development

**Terminal:**

```bash
bun tauri dev
# Logs appear in the running terminal
```

**Browser DevTools:**

1. Open app in development mode
2. Press F12 or right-click → Inspect
3. Go to Console tab
4. Logs appear with full formatting

### Production

**View log file:**

```bash
# macOS
tail -f ~/Library/Logs/com.uniclipboard/uniclipboard.log

# Linux
tail -f ~/.local/share/com.uniclipboard/logs/uniclipboard.log

# Windows (PowerShell)
Get-Content "$env:LOCALAPPDATA\com.uniclipboard\logs\uniclipboard.log" -Wait
```

**Filter for errors:**

```bash
grep ERROR ~/Library/Logs/com.uniclipboard/uniclipboard.log
```

**View last 100 lines:**

```bash
tail -n 100 ~/Library/Logs/com.uniclipboard/uniclipboard.log
```

## Log Rotation

Currently, the log file grows indefinitely. In production, you may want to:

1. **Manual rotation**: Periodically archive/delete old logs
2. **Log rotation tools**: Use system utilities (logrotate on Linux)
3. **Tauri plugin rotation**: Configure `.rotation_strategy()` in the builder

To enable automatic rotation:

```rust
tauri_plugin_log::Builder::new()
    .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
    // or
    .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepDays(7))
```

## Testing

### Unit Tests

The logging module includes basic tests:

```rust
#[test]
fn test_logger_builder() {
    let _builder = get_builder();
}
```

Run with: `cd src-tauri && cargo test --package uc-tauri`

### Manual Testing

1. **Development**: Run `bun tauri dev` and check terminal/console output
2. **Production**: Build and run, check log file exists and contains entries
3. **Level filtering**: Verify DEBUG logs appear in dev but not in production

## Troubleshooting

### No logs appearing

1. Check `main.rs` has `.plugin(logging::get_builder().build())`
2. Verify `log` crate dependency is present
3. Ensure you're using `log::info!` not `println!`

### Logs not appearing in browser

1. Check Webview target is enabled in development mode
2. Open browser DevTools and check Console tab
3. Verify there are no JavaScript errors preventing log display

### Log file not created

1. Check app has write permissions to log directory
2. Verify LogDir target is enabled in production mode
3. Check platform-specific log directory path

## References

- [Tauri Plugin Log Documentation](https://v2.tauri.app/plugin/logging/)
- [log Crate Documentation](https://docs.rs/log/)
- Source: `src-tauri/crates/uc-tauri/src/bootstrap/logging.rs`
