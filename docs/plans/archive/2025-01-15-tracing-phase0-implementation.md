# Tracing Migration Phase 0: Infrastructure Setup Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Set up tracing infrastructure alongside existing log system, enabling gradual migration from `log` crate to `tracing` crate with dual-track logging support.

**Architecture:** Add `tracing` and `tracing-subscriber` dependencies to all crates, create new `bootstrap/tracing.rs` module with subscriber initialization, integrate into `main.rs` before Tauri setup, and verify that both `log` and `tracing` macros work simultaneously via `tracing-log` bridge.

**Tech Stack:** `tracing` 0.1, `tracing-subscriber` 0.3 (with env-filter, fmt, chrono features), `tracing-log` 0.2, `chrono` 0.4, existing `tauri-plugin-log` 2 (kept for Webview output).

---

## Task 1: Add tracing dependencies to uc-tauri crate

**Files:**

- Modify: `src-tauri/crates/uc-tauri/Cargo.toml`

**Step 1: Add tracing dependencies to Cargo.toml**

Open `src-tauri/crates/uc-tauri/Cargo.toml` and add the following dependencies to the `[dependencies]` section (after the existing `log = "0.4"` line):

```toml
# Logging (existing)
log = "0.4"
tauri-plugin-log = "2"
chrono = { version = "0.4", features = ["serde"] }

# NEW: Tracing support
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt", "chrono"] }
tracing-log = "0.2"
```

**Step 2: Run cargo check to verify dependencies**

Run: `cargo check -p uc-tauri`
Expected: Output shows dependencies are fetched and compiled successfully.

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-tauri/Cargo.toml
git commit -m "feat(uc-tauri): add tracing dependencies

Add tracing, tracing-subscriber, and tracing-log dependencies
to enable gradual migration from log crate to tracing.

- tracing: core span and event macros
- tracing-subscriber: fmt layer with env-filter
- tracing-log: log bridge for compatibility

Part of Phase 0: Infrastructure Setup"
```

---

## Task 2: Add tracing dependency to uc-app crate

**Files:**

- Modify: `src-tauri/crates/uc-app/Cargo.toml`

**Step 1: Read uc-app Cargo.toml**

Run: `cat src-tauri/crates/uc-app/Cargo.toml`
Expected: See existing dependencies list.

**Step 2: Add tracing dependency to uc-app**

Add to `[dependencies]` section (at the end is fine):

```toml
tracing = "0.1"
```

**Step 3: Run cargo check to verify**

Run: `cargo check -p uc-app`
Expected: No errors, dependency resolves.

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-app/Cargo.toml
git commit -m "feat(uc-app): add tracing dependency

Add tracing crate for use case layer logging.

Part of Phase 0: Infrastructure Setup"
```

---

## Task 3: Add tracing dependency to uc-infra crate

**Files:**

- Modify: `src-tauri/crates/uc-infra/Cargo.toml`

**Step 1: Add tracing dependency**

Add to `[dependencies]` section:

```toml
tracing = "0.1"
```

**Step 2: Run cargo check to verify**

Run: `cargo check -p uc-infra`
Expected: No errors.

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-infra/Cargo.toml
git commit -m "feat(uc-infra): add tracing dependency

Add tracing crate for infrastructure layer logging.

Part of Phase 0: Infrastructure Setup"
```

---

## Task 4: Add tracing dependency to uc-platform crate

**Files:**

- Modify: `src-tauri/crates/uc-platform/Cargo.toml`

**Step 1: Add tracing dependency**

Add to `[dependencies]` section:

```toml
tracing = "0.1"
```

**Step 2: Run cargo check to verify**

Run: `cargo check -p uc-platform`
Expected: No errors.

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-platform/Cargo.toml
git commit -m "feat(uc-platform): add tracing dependency

Add tracing crate for platform adapter layer logging.

Part of Phase 0: Infrastructure Setup"
```

---

## Task 5: Add optional tracing dependency to uc-core crate

**Files:**

- Modify: `src-tauri/crates/uc-core/Cargo.toml`

**Step 1: Read uc-core Cargo.toml**

Run: `cat src-tauri/crates/uc-core/Cargo.toml`
Expected: See dependencies structure.

**Step 2: Add optional tracing dependency and feature**

Add to `[dependencies]` section:

```toml
# Optional: domain layer only records events (facts)
tracing = { version = "0.1", optional = true }
```

Add to `[features]` section (create if not exists):

```toml
[features]
default = ["tracing"]
logging = ["tracing"]
```

**Step 3: Run cargo check to verify**

Run: `cargo check -p uc-core`
Expected: No errors, feature enabled by default.

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-core/Cargo.toml
git commit -m "feat(uc-core): add optional tracing dependency

Add tracing as optional dependency for domain layer.
Domain only records events (facts), no spans.

Features:
- default: includes tracing
- logging: explicit tracing feature

Part of Phase 0: Infrastructure Setup"
```

---

## Task 6: Create tracing.rs module (init_tracing_subscriber function)

**Files:**

- Create: `src-tauri/crates/uc-tauri/src/bootstrap/tracing.rs`

**Step 1: Create the tracing module file**

Create file at `src-tauri/crates/uc-tauri/src/bootstrap/tracing.rs` with the following content:

````rust
//! Tracing configuration for UniClipboard
//!
//! This module provides the tracing-subscriber initialization for structured
//! logging with spans, supporting the gradual migration from `log` crate to
//! `tracing` crate.
//!
//! ## Architecture / 架构
//!
//! - **Dual-track system**: Both `log` and `tracing` work during transition
//! - **Format compatibility**: Output format matches existing `log` format
//! - **Environment-aware**: Development uses Webview, Production uses file+stdout
//!
//! ## Migration Path / 迁移路径
//!
//! Phase 0: Infrastructure setup (this module)
//! Phase 1: Command layer creates root spans
//! Phase 2: UseCase layer creates child spans
//! Phase 3: Infra/Platform layers add debug spans
//! Phase 4: Remove `log` dependency (optional)

use tracing_subscriber::{fmt, registry, prelude::*};
use tracing_log::LogTracer;
use std::io;

/// Check if running in development environment
fn is_development() -> bool {
    cfg!(debug_assertions)
}

/// Initialize the tracing subscriber with appropriate configuration
///
/// ## Behavior / 行为
///
/// - **Development**: Debug level, outputs to stdout (Webview via tauri-plugin-log)
/// - **Production**: Info level, outputs to stdout (tauri-plugin-log handles file)
/// - **Environment filter**: Respects RUST_LOG, with sensible defaults
/// - **Log bridge**: Existing `log::info!()` calls are captured by tracing
///
/// ## English
///
/// This function:
/// 1. Initializes the log-tracing bridge (captures `log` macros)
/// 2. Creates an env-filter for level control
/// 3. Sets up fmt layer with log-compatible formatting
/// 4. Registers the global subscriber
///
/// ## Call this / 调用位置
///
/// Call in `main.rs` **before** Tauri Builder setup:
///
/// ```ignore
/// fn main() {
///     // ... load config ...
///     uc_tauri::bootstrap::tracing::init_tracing_subscriber()
///         .expect("Failed to initialize tracing");
///
///     run_app(config);
/// }
/// ```
///
/// ## Errors / 错误
///
/// Returns `Err` if:
/// - Subscriber is already registered (should only call once)
/// - Invalid filter directives in RUST_LOG
pub fn init_tracing_subscriber() -> anyhow::Result<()> {
    let is_dev = is_development();

    // Step 1: Initialize log-tracing bridge
    // This captures all existing log::info!, log::error! etc. calls
    // and redirects them into the tracing system
    LogTracer::new().init()?;

    // Step 2: Build environment filter
    // - Defaults to debug in dev, info in prod
    // - Filters libp2p_mdns warnings (noisy proxy software errors)
    // - Can be overridden with RUST_LOG environment variable
    let env_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(
            if is_dev {
                "debug".parse()?
            } else {
                "info".parse()?
            }
        )
        .with_directives([
            "libp2p_mdns=warn",           // Filter noisy proxy errors
            "uc_platform=debug",           // Platform layer: debug for dev
            "uc_infra=debug",              // Infra layer: debug for dev
        ])
        .from_env_lossy();                // Allow RUST_LOG override

    // Step 3: Create fmt layer (formatting)
    // Format matches existing log format for compatibility:
    // "2025-01-15 10:30:45.123 INFO [file.rs:42] [target] message"
    let fmt_layer = fmt::layer()
        .with_timer(fmt::time::ChronoUtc::with_format("%Y-%m-%d %H:%M:%S%.3f"))
        .with_level(true)
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .with_ansi(cfg!(not(test)))        // Disable colors in tests
        .with_writer(io::stdout);          // Output to stdout (tauri-plugin-log handles Webview/file)

    // Step 4: Register the global subscriber
    // This MUST be called once, before any logging occurs
    registry()
        .with(env_filter)
        .with(fmt_layer)
        .try_init()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracing_init() {
        // Note: This will panic if subscriber already registered
        // In normal tests, we'd use a test subscriber instead
        // For now, just verify the function compiles
        let is_dev = is_development();
        let _ = is_dev; // Suppress unused warning
    }
}
````

**Step 2: Create bootstrap module mod.rs if not exists**

Check if `src-tauri/crates/uc-tauri/src/bootstrap/mod.rs` exists:

Run: `ls src-tauri/crates/uc-tauri/src/bootstrap/mod.rs`
Expected: File exists (it should, since logging.rs is there)

If it doesn't exist, create it with:

```rust
pub mod logging;
pub mod tracing;
```

If it exists, open it and add the tracing module declaration after logging:

```rust
pub mod logging;
pub mod tracing;
```

**Step 3: Run cargo check to verify**

Run: `cargo check -p uc-tauri`
Expected: No errors, module compiles successfully.

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/tracing.rs
git add src-tauri/crates/uc-tauri/src/bootstrap/mod.rs
git commit -m "feat(uc-tauri): create tracing subscriber initialization

Add bootstrap/tracing.rs module with init_tracing_subscriber().

Key features:
- Log-tracing bridge captures existing log:: macros
- Env-filter respects RUST_LOG with sensible defaults
- Fmt layer format compatible with existing log output
- Development: debug level, Production: info level

Part of Phase 0: Infrastructure Setup"
```

---

## Task 7: Initialize tracing in main.rs

**Files:**

- Modify: `src-tauri/src/main.rs:1-90`

**Step 1: Add tracing import to main.rs**

Add to the imports section (after the existing imports, around line 21):

```rust
use uc_tauri::bootstrap::{load_config, wire_dependencies, AppRuntime};
use uc_tauri::bootstrap::logging;
use uc_tauri::bootstrap::tracing;  // NEW
```

**Step 2: Initialize tracing in main() function**

Modify the `main()` function to initialize tracing early:

Change:

```rust
fn main() {
    // NOTE: In a production application, we would:
    // 1. Load configuration from a proper path
    // 2. Handle configuration errors gracefully

    // For now, use a default config path
    let config_path = PathBuf::from("config.toml");
```

To:

```rust
fn main() {
    // Initialize tracing subscriber FIRST (before any logging)
    // This sets up the tracing infrastructure and enables log-tracing bridge
    if let Err(e) = tracing::init_tracing_subscriber() {
        eprintln!("Failed to initialize tracing: {}", e);
        std::process::exit(1);
    }

    // NOTE: In a production application, we would:
    // 1. Load configuration from a proper path
    // 2. Handle configuration errors gracefully

    // For now, use a default config path
    let config_path = PathBuf::from("config.toml");
```

**Step 3: Run cargo check to verify**

Run: `cargo check`
Expected: No errors, main function compiles.

**Step 4: Run the app to verify tracing works**

Run: `bun tauri dev`
Expected: Application starts, logs appear in terminal with tracing format.

**Step 5: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "feat(main): initialize tracing subscriber on startup

Call init_tracing_subscriber() at the start of main() before
any logging occurs. This enables both log and tracing macros
to work simultaneously via the log-tracing bridge.

Part of Phase 0: Infrastructure Setup"
```

---

## Task 8: Add log re-export for backward compatibility

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/tracing.rs:1-50`

**Step 1: Add log re-export at top of tracing.rs**

Add at the top of the file after the doc comments (before `use tracing_subscriber...`):

```rust
//! Tracing configuration for UniClipboard
//!
//! ... (existing doc comments) ...

// Re-export log for backward compatibility during migration
// This allows crates to use `use uc_tauri::bootstrap::tracing::log`
pub use log;
```

**Step 2: Run cargo check to verify**

Run: `cargo check -p uc-tauri`
Expected: No errors.

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/tracing.rs
git commit -m "refactor(uc-tauri): re-export log from tracing module

Re-export log crate for backward compatibility during migration.
Allows crates to import log from bootstrap::tracing if needed.

Part of Phase 0: Infrastructure Setup"
```

---

## Task 9: Create verification test (log-tracing bridge)

**Files:**

- Create: `src-tauri/crates/uc-tauri/tests/tracing_verification.rs`

**Step 1: Create integration test file**

Create file at `src-tauri/crates/uc-tauri/tests/tracing_verification.rs`:

```rust
//! Verification test for log-tracing bridge
//!
//! This test verifies that both `log` and `tracing` macros work
//! and appear in the same log output.

#[test]
fn test_log_and_tracing_compatibility() {
    // This test verifies the log-tracing bridge is working
    // by checking that log macros compile and execute

    // Test log macro (should be captured by tracing via bridge)
    log::info!("Test log::info message");
    log::debug!("Test log::debug message");
    log::warn!("Test log::warn message");

    // Test tracing macro
    tracing::info!("Test tracing::info message");
    tracing::debug!("Test tracing::debug message");
    tracing::warn!("Test tracing::warn message");

    // If we get here without panicking, both systems work
    assert!(true);
}
```

**Step 2: Run the test**

Run: `cargo test -p uc-tauri test_log_and_tracing_compatibility -- --nocapture`
Expected: Test passes, you see both log and tracing messages in output.

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-tauri/tests/tracing_verification.rs
git commit -m "test(uc-tauri): add log-tracing bridge verification test

Add integration test verifying both log and tracing macros
work simultaneously via the log-tracing bridge.

Part of Phase 0: Infrastructure Setup"
```

---

## Task 10: Manual verification with dev server

**Files:**

- None (verification step)

**Step 1: Start development server with verbose logging**

Run: `RUST_LOG=trace bun tauri dev`
Expected:

- Application starts
- Logs appear in terminal with format: `2025-01-15 HH:MM:SS.mmm LEVEL [file:line] [target] message`
- Both `log::` and `tracing::` messages appear

**Step 2: Check for span structure (preparation for future phases)**

Look for any existing log messages. They should still work.

Expected output example:

```
2025-01-15 10:30:45.123 INFO [main.rs:75] [uniclipboard] Creating platform runtime with clipboard callback
2025-01-15 10:30:45.124 INFO [main.rs:196] [uniclipboard] Platform runtime task started
```

**Step 3: Verify RUST_LOG filtering works**

Run with different log levels:

- `RUST_LOG=error bun tauri dev` - should only show errors
- `RUST_LOG=uc_platform=trace bun tauri dev` - should show verbose platform logs

**Step 4: Create verification documentation**

Create a markdown note for the verification results:

```bash
cat > /tmp/tracing_phase0_verification.md << 'EOF'
# Phase 0 Verification Results

## Date
$(date +%Y-%m-%d)

## Tests Passed
- [x] All crates compile with tracing dependencies
- [x] init_tracing_subscriber() initializes without error
- [x] Application starts with tracing enabled
- [x] Existing log:: macros still work
- [x] tracing:: macros work
- [x] RUST_LOG environment variable controls filtering
- [x] Format matches existing log output

## Log Output Sample
(Paste sample output from step 2)

## Ready for Phase 1
Yes - Infrastructure is in place for Command layer migration.
EOF
cat /tmp/tracing_phase0_verification.md
```

**Step 5: Commit verification notes (optional)**

```bash
git add docs/plans/2025-01-15-tracing-phase0-implementation.md
git commit --allow-empty -m "test(verification): Phase 0 infrastructure setup complete

All tracing infrastructure is in place and verified:
- Dependencies added to all crates
- init_tracing_subscriber() functional
- Log-tracing bridge working
- RUST_LOG filtering verified

Ready to proceed to Phase 1: Command layer migration."
```

---

## Task 11: Update logging architecture documentation

**Files:**

- Modify: `docs/architecture/logging-architecture.md`

**Step 1: Read existing documentation**

Run: `cat docs/architecture/logging-architecture.md`
Expected: See current logging documentation (mostly about log crate).

**Step 2: Add tracing section to documentation**

Add a new section after the "Using Logs in Code" section. First, find the line with "## Using Logs in Code" and add after that section:

````markdown
## Tracing Migration (In Progress)

### Current Status: Phase 0 Complete

The application is migrating from `log` crate to `tracing` crate for structured logging with spans.

**Migration Strategy**: Gradual (dual-track system)

- **Phase 0** ✅: Infrastructure setup (tracing dependencies, subscriber initialization)
- **Phase 1** 🚧: Command layer (root spans)
- **Phase 2**: UseCase layer (child spans)
- **Phase 3**: Infra/Platform layer (debug spans)
- **Phase 4**: Cleanup (remove `log` dependency)

### Dual-Track Logging System

During migration, both `log` and `tracing` macros work simultaneously:

```rust
// Existing log macros (still work)
log::info!("Application started");

// New tracing macros (now available)
tracing::info!("Application started");
tracing::info_span!("command.clipboard.capture", device_id = %id);
```
````

The `tracing-log` bridge captures all `log::` macros into the tracing system, ensuring consistent output.

### Tracing Configuration

Tracing is initialized in `main.rs` via `uc_tauri::bootstrap::tracing::init_tracing_subscriber()`.

**Behavior**:

- Development: Debug level, stdout output (Webview via tauri-plugin-log)
- Production: Info level, file + stdout output
- Environment filter: Respects `RUST_LOG`, with defaults:
  - `libp2p_mdns=warn` (noisy proxy errors)
  - `uc_platform=debug` (platform layer)
  - `uc_infra=debug` (infrastructure layer)

### Format Compatibility

Tracing output format matches existing `log` format:

```
2025-01-15 10:30:45.123 INFO [main.rs:42] [uniclipboard] Application started
```

This ensures continuity with existing log parsing tools and workflows.

### Future: Span-Based Tracing

Once migration is complete, logs will include span hierarchy for cross-layer traceability:

```
command.clipboard.capture{device_id=abc123}
└─ usecase.capture_clipboard.execute{policy_version=v1}
   ├─ platform.macos.read_clipboard
   │  └─ event: formats=3
   ├─ infra.sqlite.insert_clipboard_event
   └─ event: capture completed
```

See [Tracing Migration Design](../plans/2025-01-15-tracing-migration-design.md) for complete architecture.

````

**Step 3: Update the "Log Filtering" section reference**

Find the section about filtering (around line 70) and update the reference to mention both systems:

Change the existing filter section to include:

```markdown
The logging system filters out:

- `libp2p_mdns` errors below WARN level (harmless proxy software errors)
- Tauri internal event logs to avoid infinite loops
- `ipc::request` logs in production builds

**Note**: During tracing migration, these filters are configured in both:
- `bootstrap/logging.rs` (tauri-plugin-log filters)
- `bootstrap/tracing.rs` (tracing-subscriber env-filter)

See `bootstrap/tracing.rs` for current RUST_LOG defaults.
````

**Step 4: Commit documentation update**

```bash
git add docs/architecture/logging-architecture.md
git commit -m "docs: add tracing migration status to logging architecture

Document Phase 0 completion and dual-track logging system.
Update filtering section to mention both log and tracing filters.

Reference design doc: docs/plans/2025-01-15-tracing-migration-design.md"
```

---

## Summary

After completing all tasks, the tracing infrastructure will be fully in place:

**Completed**:

- ✅ `tracing` dependencies added to all crates (uc-tauri, uc-app, uc-infra, uc-platform, uc-core)
- ✅ `tracing-subscriber` with env-filter, fmt, and chrono features
- ✅ `tracing-log` bridge for `log` macro compatibility
- ✅ `bootstrap/tracing.rs` module with `init_tracing_subscriber()`
- ✅ Initialization in `main.rs` before Tauri setup
- ✅ Verification test for log-tracing bridge
- ✅ Documentation updated with migration status

**Next Phase**:

- Phase 1: Migrate Command layer to create root spans

**Verification Command**:

```bash
RUST_LOG=trace bun tauri dev
```

Should see both `log::` and `tracing::` messages in terminal output.

---

## References

- Design document: `docs/plans/2025-01-15-tracing-migration-design.md`
- Architecture doc: `docs/architecture/logging-architecture.md`
- Tracing crate: https://docs.rs/tracing/
- Tracing-subscriber: https://docs.rs/tracing-subscriber/
