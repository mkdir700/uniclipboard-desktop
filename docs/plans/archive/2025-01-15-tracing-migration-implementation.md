# Tracing Migration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Migrate UniClipboard from `log` crate to `tracing` crate for structured, span-based logging across all layers.

**Architecture:** Gradual migration with dual-track logging (log + tracing coexistence). Start with infrastructure setup, then migrate layers top-down: Command → UseCase → Infra/Platform. Each layer creates appropriate spans: Command (root), UseCase (business lifecycle), Infra/Platform (debug-level operations), Domain (events only).

**Tech Stack:** tracing 0.1, tracing-subscriber 0.3, tracing-log 0.2, tauri-plugin-log 2 (transition only)

**Reference Design:** `docs/plans/2025-01-15-tracing-migration-design.md`

---

## Phase 0: Infrastructure Setup

### Task 0.1: Add tracing dependencies to uc-tauri

**Files:**

- Modify: `src-tauri/crates/uc-tauri/Cargo.toml`

**Step 1: Add tracing dependencies**

Add to `[dependencies]` section:

```toml
# Tracing (structured logging with spans)
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt", "chrono"] }
tracing-log = "0.2"

# Keep tauri-plugin-log during transition
tauri-plugin-log = "2"
```

**Step 2: Run cargo check**

Run: `cd src-tauri && cargo check --package uc-tauri`
Expected: No errors, dependencies resolve

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-tauri/Cargo.toml
git commit -m "feat(uc-tauri): add tracing dependencies

Add tracing, tracing-subscriber, and tracing-log for structured logging.
Keep tauri-plugin-log during transition period."
```

---

### Task 0.2: Add tracing dependencies to uc-app

**Files:**

- Modify: `src-tauri/crates/uc-app/Cargo.toml`

**Step 1: Replace log with tracing**

Change:

```toml
# Logging
log = "0.4"
```

To:

```toml
# Logging (tracing)
tracing = "0.1"
```

**Step 2: Run cargo check**

Run: `cd src-tauri && cargo check --package uc-app`
Expected: No errors (tracing already in dependencies)

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-app/Cargo.toml
git commit -m "feat(uc-app): migrate to tracing for logging"
```

---

### Task 0.3: Add tracing dependency to uc-infra

**Files:**

- Modify: `src-tauri/crates/uc-infra/Cargo.toml`

**Step 1: Add tracing dependency**

Add to `[dependencies]`:

```toml
tracing = "0.1"
```

**Step 2: Run cargo check**

Run: `cd src-tauri && cargo check --package uc-infra`
Expected: No errors

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-infra/Cargo.toml
git commit -m "feat(uc-infra): add tracing dependency"
```

---

### Task 0.4: Add tracing dependency to uc-platform

**Files:**

- Modify: `src-tauri/crates/uc-platform/Cargo.toml`

**Step 1: Add tracing dependency**

Add to `[dependencies]`:

```toml
tracing = "0.1"
```

**Step 2: Run cargo check**

Run: `cd src-tauri && cargo check --package uc-platform`
Expected: No errors

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-platform/Cargo.toml
git commit -m "feat(uc-platform): add tracing dependency"
```

---

### Task 0.5: Add optional tracing dependency to uc-core

**Files:**

- Modify: `src-tauri/crates/uc-core/Cargo.toml`

**Step 1: Add optional tracing dependency with feature**

Add to `[dependencies]`:

```toml
# Domain layer: optional tracing for events only
tracing = { version = "0.1", optional = true }
```

Add to `[features]`:

```toml
[features]
default = ["tracing"]
logging = ["tracing"]  # Allow disabling for pure computation
```

**Step 2: Run cargo check**

Run: `cd src-tauri && cargo check --package uc-core`
Expected: No errors

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-core/Cargo.toml
git commit -m "feat(uc-core): add optional tracing dependency

Domain layer uses tracing only for recording events (not spans).
Feature flag allows disabling for pure computation scenarios."
```

---

### Task 0.6: Create tracing subscriber module

**Files:**

- Create: `src-tauri/crates/uc-tauri/src/bootstrap/tracing.rs`

**Step 1: Write the tracing subscriber initialization**

Create `src-tauri/crates/uc-tauri/src/bootstrap/tracing.rs`:

````rust
//! Tracing subscriber configuration for UniClipboard
//!
//! This module configures the tracing subscriber to work alongside
//! the existing log-based system during the migration period.
//!
//! ## Output Targets
//!
//! - **Development**: Webview console (via custom writer)
//! - **Production**: LogDir file + Stdout
//!
//! ## Environment Behavior
//!
//! - **Development**: Debug level, outputs to Webview console
//! - **Production**: Info level, outputs to log file + stdout
//!
//! ## Log Locations
//!
//! - **macOS**: `~/Library/Logs/com.uniclipboard/uniclipboard.log`
//! - **Linux**: `~/.local/share/com.uniclipboard/logs/uniclipboard.log`
//! - **Windows**: `%LOCALAPPDATA%\com.uniclipboard\logs/uniclipboard.log`

use tracing_subscriber::{fmt, registry, prelude::*};
use tracing_log::LogTracer;
use std::io;

/// Check if running in development environment
fn is_development() -> bool {
    cfg!(debug_assertions)
}

/// Initialize the tracing subscriber
///
/// ## Behavior / 行为
///
/// - Development: Debug level, Webview console output
/// - Production: Info level, file + stdout output
/// - Filters noise from libp2p_mdns and Tauri internals
/// - Compatible with existing log calls via tracing-log bridge
///
/// ## English
///
/// Configures the tracing subscriber based on the build environment.
/// This should be called in main.rs before Tauri builder setup.
///
/// ## Example
///
/// ```no_run
/// use uc_tauri::bootstrap::tracing::init_tracing_subscriber;
///
/// fn main() {
///     init_tracing_subscriber().expect("failed to initialize tracing");
///     // ... rest of main
/// }
/// ```
pub fn init_tracing_subscriber() -> Result<(), Box<dyn std::error::Error>> {
    let is_dev = is_development();

    // Environment filter
    let env_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(if is_dev { "debug" } else { "info" }.parse()?)
        .with_directives([
            "libp2p_mdns=warn",           // Match existing log config
            "uc_platform=debug",           // Platform layer: debug
            "uc_infra=debug",              // Infra layer: debug
            "uc_app=info",                 // App layer: info
            "uc_core=info",                // Core layer: info
        ])
        .from_env_lossy();                // Allow RUST_LOG override

    // Fmt Layer (format compatible with existing log output)
    let fmt_layer = fmt::layer()
        .with_timer(fmt::time::ChronoUtc::with_format("%Y-%m-%d %H:%M:%S%.3f"))
        .with_level(true)
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .with_ansi(cfg!(not(test)))        // Disable colors in tests
        .with_writer(if is_dev {
            // Development: Use stdout for now (Webview writer to be added later)
            io::stdout
        } else {
            // Production: Output to stdout (file logging via tauri-plugin-log)
            io::stdout
        });

    // Register subscriber
    registry()
        .with(env_filter)
        .with(fmt_layer)
        .try_init()?;

    // Install log tracer (bridges log:: macros to tracing)
    LogTracer::new().init()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracing_init() {
        // Verify subscriber can be initialized without panicking
        // Note: This will fail if a subscriber is already registered
        // Run with: cargo test --package uc-tauri -- --test-threads=1
        let result = init_tracing_subscriber();
        // May fail if already initialized (expected in some test scenarios)
        assert!(result.is_ok() || result.unwrap_err().to_string().contains("already been set"));
    }

    #[test]
    fn test_development_detection() {
        let _is_dev = is_development();
        // Just verify the function works
    }
}
````

**Step 2: Add module to bootstrap lib.rs**

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/mod.rs`

Add:

```rust
pub mod tracing;

pub use tracing::init_tracing_subscriber;
```

**Step 3: Run cargo check**

Run: `cd src-tauri && cargo check --package uc-tauri`
Expected: No errors

**Step 4: Run tests**

Run: `cd src-tauri && cargo test --package uc-tauri -- bootstrap::tracing`
Expected: Tests pass

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/tracing.rs
git add src-tauri/crates/uc-tauri/src/bootstrap/mod.rs
git commit -m "feat(uc-tauri): add tracing subscriber module

Implement tracing subscriber initialization with:
- Environment-based log levels (dev: debug, prod: info)
- Filter configuration matching existing log setup
- tracing-log bridge for compatibility with existing log:: macros
- Chrono timestamp formatting matching log output

Tests included for initialization."
```

---

### Task 0.7: Initialize tracing in main.rs

**Files:**

- Modify: `src-tauri/src/main.rs`

**Step 1: Add tracing initialization before Tauri setup**

Find the `run_app` function or main setup code. Add tracing initialization **before** the Tauri builder.

The code should look like:

```rust
use uc_tauri::bootstrap::tracing;

fn run_app() -> Result<()> {
    // Initialize tracing FIRST (before any logging happens)
    tracing::init_tracing_subscriber()
        .expect("failed to initialize tracing subscriber");

    // ... rest of setup
}
```

**Step 2: Verify app still starts**

Run: `bun tauri dev`
Expected: App starts, logs appear in terminal with tracing format

**Step 3: Check log format**

Look for logs in terminal. Should see:

```
2025-01-15 10:30:45.123 INFO [main.rs:140] [uniclipboard] message
```

**Step 4: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "feat(main): initialize tracing subscriber

Initialize tracing before Tauri builder setup.
All existing log:: macros now flow through tracing subscriber."
```

---

### Task 0.8: Verify tracing-log bridge works

**Step 1: Add test log to verify bridge**

Temporarily add to any file with log usage:

```rust
// Temporary test - remove after verification
log::info!("TEST: tracing-log bridge working");
```

**Step 2: Run app and verify**

Run: `bun tauri dev`
Expected: See "TEST: tracing-log bridge working" in terminal logs

**Step 3: Remove test code**

Remove the test log added in step 1.

**Step 4: Commit (if changes were made)**

If any temporary code was added and removed, commit the removal:

```bash
git add -A
git commit -m "test: verify tracing-log bridge functionality

Confirmed that existing log:: macros are captured by tracing subscriber."
```

---

### Task 0.9: Update logging architecture documentation

**Files:**

- Modify: `docs/architecture/logging-architecture.md`

**Step 1: Update framework section**

Find the section that says:

```markdown
\*\*Current Implementation: `log` crate (NOT `tracing`)`
```

Replace with:

```markdown
**Current Implementation: `log` crate + `tracing` crate (Migration in Progress)**

The application is transitioning from the `log` crate to `tracing` for structured, span-based logging.

**Migration Status**: Phase 0 (Infrastructure) - See `docs/plans/2025-01-15-tracing-migration-implementation.md`

**Currently Supported:**

- Simple level-based logging (`log::error!`, `log::info!`, etc.)
- Span-based tracing (`tracing::info_span!`, etc.)
- Structured fields with spans
- Parent-child span relationships
- tracing-log bridge (log macros captured by tracing)

**Planned (Phase 4 - Cleanup):**

- Remove `log` crate dependency
- Pure tracing architecture
- OpenTelemetry integration (optional)
```

**Step 2: Add migration reference**

Add to "References" section:

```markdown
- [Tracing Migration Plan](../plans/2025-01-15-tracing-migration-implementation.md)
- [Tracing Migration Design](../plans/2025-01-15-tracing-migration-design.md)
```

**Step 3: Commit**

```bash
git add docs/architecture/logging-architecture.md
git commit -m "docs: update logging architecture for tracing migration

Document current migration status and reference implementation plan."
```

---

## Phase 1: Command Layer Migration (Root Spans)

### Task 1.1: Migrate clipboard commands to use tracing spans

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/clipboard.rs`

**Step 1: Add tracing import**

Add to imports:

```rust
use tracing::{info_span, Instrument};
```

**Step 2: Wrap get_clipboard_entries with root span**

Modify the function:

```rust
#[tauri::command]
pub async fn get_clipboard_entries(
    runtime: State<'_, AppRuntime>,
    limit: Option<usize>,
) -> Result<Vec<ClipboardEntryProjection>, String> {
    let span = info_span!(
        "command.clipboard.get_entries",
        device_id = %runtime.device_id(),
        limit = limit.unwrap_or(50),
    );
    async {
        // Use UseCases accessor pattern (consistent with other commands)
        let uc = runtime.usecases().list_clipboard_entries();
        let limit = limit.unwrap_or(50);

        // Query entries through use case
        let entries = uc.execute(limit, 0)
            .await
            .map_err(|e| e.to_string())?;

        // Convert domain models to DTOs
        let projections: Vec<ClipboardEntryProjection> = entries
            .into_iter()
            .map(|entry| ClipboardEntryProjection {
                id: range.entry_id.to_string(),
                preview: entry.title.unwrap_or_else(|| format!("Entry ({} bytes)", entry.total_size)),
                captured_at: entry.created_at_ms,
                content_type: "clipboard".to_string(),
                is_encrypted: false, // TODO: Determine from actual entry state
            })
            .collect();

        Ok(projections)
    }
    .instrument(span)
    .await
}
```

**Step 3: Wrap delete_clipboard_entry with root span**

Modify the function:

```rust
#[tauri::command]
pub async fn delete_clipboard_entry(
    runtime: State<'_, AppRuntime>,
    entry_id: String,
) -> Result<(), String> {
    let span = info_span!(
        "command.clipboard.delete_entry",
        device_id = %runtime.device_id(),
        entry_id = %entry_id,
    );
    async {
        // Parse entry_id
        let entry_id = uc_core::ids::EntryId::from(entry_id.clone());

        // Execute use case
        let use_case = runtime.usecases().delete_clipboard_entry();
        use_case.execute(&entry_id)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }
    .instrument(span)
    .await
}
```

**Step 4: Update capture_clipboard placeholder**

Add span even though it's not implemented:

```rust
#[tauri::command]
pub async fn capture_clipboard(
    runtime: State<'_, AppRuntime>,
) -> Result<String, String> {
    let span = info_span!(
        "command.clipboard.capture",
        device_id = %runtime.device_id(),
    );
    async {
        // TODO: Implement CaptureClipboard use case
        Err("Not yet implemented - requires CaptureClipboard use case with multiple ports".to_string())
    }
    .instrument(span)
    .await
}
```

**Step 5: Run cargo check**

Run: `cd src-tauri && cargo check --package uc-tauri`
Expected: No errors

**Step 6: Test in development**

Run: `bun tauri dev`
Expected: App starts, commands work, logs show span trees

**Step 7: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/clipboard.rs
git commit -m "feat(commands): add root spans to clipboard commands

All clipboard commands now create root spans with:
- command.clipboard.get_entries
- command.clipboard.delete_entry
- command.clipboard.capture

Each span includes device_id and relevant context."
```

---

### Task 1.2: Migrate encryption commands to use tracing spans

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/encryption.rs`

**Step 1: Read the file to understand structure**

Run: `cat src-tauri/crates/uc-tauri/src/commands/encryption.rs`
Expected: See encryption command implementations

**Step 2: Add tracing import and wrap commands**

Similar to clipboard commands, wrap each command with appropriate span:

```rust
use tracing::{info_span, Instrument};

#[tauri::command]
pub async fn initialize_encryption(
    runtime: State<'_, AppRuntime>,
    passphrase: String,
) -> Result<(), String> {
    let span = info_span!(
        "command.encryption.initialize",
        device_id = %runtime.device_id(),
    );
    async {
        // ... existing implementation
    }
    .instrument(span)
    .await
}

#[tauri::command]
pub async fn is_encryption_initialized(
    runtime: State<'_, AppRuntime>,
) -> Result<bool, String> {
    let span = info_span!(
        "command.encryption.is_initialized",
        device_id = %runtime.device_id(),
    );
    async {
        // ... existing implementation
    }
    .instrument(span)
    .await
}
```

**Step 3: Run cargo check**

Run: `cd src-tauri && cargo check --package uc-tauri`
Expected: No errors

**Step 4: Test in development**

Run: `bun tauri dev`
Expected: Commands work, logs show encryption command spans

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/encryption.rs
git commit -m "feat(commands): add root spans to encryption commands

Add root spans for:
- command.encryption.initialize
- command.encryption.is_initialized"
```

---

### Task 1.3: Migrate settings commands to use tracing spans

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/settings.rs`

**Step 1: Read the file and add spans**

Add tracing imports and wrap each command with appropriate span:

```rust
use tracing::{info_span, Instrument};

#[tauri::command]
pub async fn get_settings(
    runtime: State<'_, AppRuntime>,
) -> Result<Settings, String> {
    let span = info_span!(
        "command.settings.get",
        device_id = %runtime.device_id(),
    );
    async {
        // ... existing implementation
    }
    .instrument(span)
    .await
}

#[tauri::command]
pub async fn update_settings(
    runtime: State<'_, AppRuntime>,
    settings: Settings,
) -> Result<(), String> {
    let span = info_span!(
        "command.settings.update",
        device_id = %runtime.device_id(),
    );
    async {
        // ... existing implementation
    }
    .instrument(span)
    .await
}
```

**Step 2: Run cargo check**

Run: `cd src-tauri && cargo check --package uc-tauri`
Expected: No errors

**Step 3: Test in development**

Run: `bun tauri dev`
Expected: Settings commands work, logs show span trees

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/settings.rs
git commit -m "feat(commands): add root spans to settings commands

Add root spans for:
- command.settings.get
- command.settings.update"
```

---

### Task 1.4: Verify Command layer spans visible

**Step 1: Run app with trace logging**

Run: `RUST_LOG=trace bun tauri dev`
Expected: See span trees in terminal output

**Step 2: Trigger commands and verify logs**

- Open clipboard list → should see `command.clipboard.get_entries` span
- Delete an entry → should see `command.clipboard.delete_entry` span
- Open settings → should see `command.settings.get` span

**Step 3: Check span format**

Verify logs show:

```
2025-01-15 10:30:45.123 INFO [command.clipboard.get_entries{device_id=abc,limit=50}]
```

**Step 4: Create verification test**

**Files:**

- Create: `src-tauri/tests/command_spans_test.rs`

```rust
#[cfg(test)]
mod tests {
    use tracing_subscriber::{Layer, prelude::*};

    // Integration test to verify command spans are created
    // This requires a full Tauri app test setup
}
```

**Step 5: Commit**

```bash
git add src-tauri/tests/command_spans_test.rs
git commit -m "test: add command span verification test

Verify that all Tauri commands create proper root spans."
```

---

## Phase 2: UseCase Layer Migration (Business Lifecycle)

### Task 2.1: Migrate RestoreClipboardSelection use case

**Files:**

- Modify: `src-tauri/crates/uc-app/src/usecases/clipboard/restore_clipboard_selection.rs`

**Step 1: Add tracing import**

```rust
use tracing::{info_span, info, debug};
```

**Step 2: Wrap execute method with span**

```rust
pub async fn execute(&self, entry_id: &EntryId) -> Result<()> {
    let span = info_span!(
        "usecase.restore_clipboard_selection.execute",
        entry_id = %entry_id,
    );
    async move {
        // 1. 读取 Entry
        let entry = self
            .clipboard_repo
            .get_entry(entry_id)
            .await?
            .ok_or(anyhow::anyhow!("Entry not found"))?;
        debug!("entry loaded");

        // 2. 获取 Selection 决策
        let selection = self
            .selection_repo
            .get_selection(entry_id)
            .await?
            .ok_or(anyhow::anyhow!("Selection not found"))?;
        debug!("selection loaded");

        // 3. 收集要恢复的所有 representation IDs
        let rep_count = selection.selection.secondary_rep_ids.len() + 1;
        let mut rep_ids = vec![selection.selection.paste_rep_id];
        rep_ids.extend(selection.selection.secondary_rep_ids);

        // 4. 加载所有 representations 的数据
        let mut representations = Vec::new();
        for rep_id in rep_ids {
            let rep = self
                .representation_repo
                .get_representation(&entry.event_id, &rep_id)
                .await?
                .ok_or(anyhow::anyhow!(
                    "Representation {} not found for event {}",
                    rep_id,
                    entry.event_id
                ))?;

            // 加载字节数据
            let bytes = if let Some(inline_data) = rep.inline_data {
                inline_data
            } else if let Some(blob_id) = rep.blob_id {
                self.blob_store.get(&blob_id).await?
            } else {
                return Err(anyhow::anyhow!("Representation has no data: {}", rep_id));
            };

            representations.push(ObservedClipboardRepresentation {
                id: rep.id,
                format_id: rep.format_id,
                mime: rep.mime_type,
                bytes,
            });
        }
        debug!(
            representations_loaded = representations.len(),
            "all representations loaded"
        );

        // 5. 构造 Snapshot
        let snapshot = SystemClipboardSnapshot {
            ts_ms: chrono::Utc::now().timestamp_millis(),
            representations,
        };

        // 6. 写入系统剪贴板
        self.local_clipboard.write_snapshot(snapshot)?;
        info!(
            representations_count = rep_count,
            "clipboard selection restored"
        );

        Ok(())
    }
    .instrument(span)
    .await
}
```

**Step 3: Run cargo check**

Run: `cd src-tauri && cargo check --package uc-app`
Expected: No errors

**Step 4: Run existing tests**

Run: `cd src-tauri && cargo test --package uc-app restore_clipboard`
Expected: Tests pass

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/clipboard/restore_clipboard_selection.rs
git commit -m "feat(usecase): add tracing span to RestoreClipboardSelection

Span covers:
- Entry loading
- Selection retrieval
- Representation data loading
- Clipboard write

Events: entry loaded, selection loaded, representations loaded, restore completed"
```

---

### Task 2.2: Migrate ListClipboardEntries use case

**Files:**

- Locate and modify: `src-tauri/crates/uc-app/src/usecases/clipboard/list_clipboard_entries.rs`

**Step 1: Find the use case file**

Run: `find src-tauri/crates/uc-app -name "*list*clipboard*" -o -name "*entries*"`
Expected: Find the file path

**Step 2: Add tracing span**

Similar to Task 2.1, wrap execute with:

```rust
let span = info_span!(
    "usecase.list_clipboard_entries.execute",
    limit = limit,
    offset = offset,
);
```

**Step 3: Run cargo check**

Run: `cd src-tauri && cargo check --package uc-app`
Expected: No errors

**Step 4: Commit**

```bash
git add <path-to-list-usecase-file>
git commit -m "feat(usecase): add tracing span to ListClipboardEntries"
```

---

### Task 2.3: Migrate DeleteClipboardEntry use case

**Files:**

- Locate and modify: `src-tauri/crates/uc-app/src/usecases/clipboard/delete_clipboard_entry.rs`

**Step 1: Add tracing span**

```rust
let span = info_span!(
    "usecase.delete_clipboard_entry.execute",
    entry_id = %entry_id,
);
```

**Step 2: Run cargo check and test**

Run: `cd src-tauri && cargo check --package uc-app && cargo test --package uc-app delete_clipboard`
Expected: No errors, tests pass

**Step 3: Commit**

```bash
git add <path-to-delete-usecase-file>
git commit -m "feat(usecase): add tracing span to DeleteClipboardEntry"
```

---

### Task 2.4: Migrate encryption use cases

**Files:**

- Locate and modify: `src-tauri/crates/uc-app/src/usecases/encryption/*.rs`

**Step 1: Find encryption use cases**

Run: `find src-tauri/crates/uc-app/src/usecases -type f -name "*.rs"`
Expected: List all use case files

**Step 2: Add spans to encryption use cases**

For each encryption use case (initialize, check status, etc.):

```rust
let span = info_span!(
    "usecase.<use_case_name>.execute",
    // relevant fields
);
```

**Step 3: Run cargo check**

Run: `cd src-tauri && cargo check --package uc-app`
Expected: No errors

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/encryption/
git commit -m "feat(usecase): add tracing spans to encryption use cases"
```

---

### Task 2.5: Verify UseCase layer spans

**Step 1: Run app and verify span hierarchy**

Run: `RUST_LOG=debug bun tauri dev`
Expected: See span trees like:

```
command.clipboard.get_entries
  └─ usecase.list_clipboard_entries.execute
```

**Step 2: Test in UI**

- Load clipboard list → should see command + usecase spans
- Delete entry → should see command + usecase spans
- Restore entry → should see command + usecase spans

**Step 3: Commit verification notes**

```bash
git add -A
git commit -m "test: verify UseCase layer span hierarchy

Confirmed all use cases create proper child spans under command spans."
```

---

## Phase 3: Infra/Platform Layer Migration

### Task 3.1: Migrate SQLite repository operations

**Files:**

- Modify: `src-tauri/crates/uc-infra/src/db/repositories/clipboard.rs` (or equivalent)

**Step 1: Add tracing debug spans**

For each DB operation, add debug span:

```rust
use tracing::debug_span;

async fn save_entry(&self, entry: &ClipboardEntry) -> Result<()> {
    let span = debug_span!(
        "infra.sqlite.insert_clipboard_entry",
        table = "clipboard_entry",
        entry_id = %entry.id,
    );
    let _enter = span.enter();

    // ... diesel insert ...

    Ok(())
}

async fn get_entry(&self, id: &EntryId) -> Result<Option<ClipboardEntry>> {
    let span = debug_span!(
        "infra.sqlite.select_clipboard_entry",
        table = "clipboard_entry",
        entry_id = %id,
    );
    let _enter = span.enter();

    // ... diesel select ...

    Ok(result)
}
```

**Step 2: Run cargo check**

Run: `cd src-tauri && cargo check --package uc-infra`
Expected: No errors

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-infra/src/db/repositories/
git commit -m "feat(infra): add debug spans to SQLite operations

All DB operations now create debug-level spans with:
- Table name
- Relevant IDs
- Operation type"
```

---

### Task 3.2: Migrate blob store operations

**Files:**

- Modify: `src-tauri/crates/uc-infra/src/blob/*.rs`

**Step 1: Add debug spans to blob operations**

```rust
use tracing::debug_span;

async fn put(&self, id: &BlobId, data: Vec<u8>) -> Result<()> {
    let span = debug_span!(
        "infra.blob.write",
        blob_id = %id,
        size_bytes = data.len(),
    );
    let _enter = span.enter();

    // ... write blob ...

    Ok(())
}

async fn get(&self, id: &BlobId) -> Result<Option<Vec<u8>>> {
    let span = debug_span!(
        "infra.blob.read",
        blob_id = %id,
    );
    let _enter = span.enter();

    // ... read blob ...

    Ok(result)
}
```

**Step 2: Run cargo check**

Run: `cd src-tauri && cargo check --package uc-infra`
Expected: No errors

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-infra/src/blob/
git commit -m "feat(infra): add debug spans to blob store operations

Blob read/write operations include size_bytes and blob_id in spans."
```

---

### Task 3.3: Migrate platform clipboard adapters

**Files:**

- Modify: `src-tauri/crates/uc-platform/src/adapters/macos/clipboard.rs` (and other platforms)

**Step 1: Add debug spans to clipboard operations**

```rust
use tracing::debug_span;

impl SystemClipboardPort for MacosClipboard {
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

    fn write_snapshot(&self, snapshot: SystemClipboardSnapshot) -> Result<()> {
        let span = debug_span!(
            "platform.macos.write_clipboard",
            representations = snapshot.representations.len(),
        );
        let _enter = span.enter();

        // ... write clipboard ...

        Ok(())
    }
}
```

**Step 2: Do the same for other platforms**

Repeat for Windows, Linux adapters if they exist.

**Step 3: Run cargo check**

Run: `cd src-tauri && cargo check --package uc-platform`
Expected: No errors

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-platform/src/adapters/
git commit -m "feat(platform): add debug spans to clipboard operations

All platform clipboard adapters create debug spans with:
- Format count
- Size information
- Operation type"
```

---

### Task 3.4: Verify complete span tree

**Step 1: Run app with full logging**

Run: `RUST_LOG=trace bun tauri dev`

**Step 2: Trigger capture operation**

Should see complete span tree:

```
command.clipboard.capture
  └─ usecase.capture_clipboard.execute
      ├─ platform.macos.read_clipboard
      │   └─ event: formats=3
      ├─ event: representation selected
      ├─ infra.sqlite.insert_clipboard_event
      ├─ infra.blob.write
      └─ event: capture completed
```

**Step 3: Document example span tree**

**Files:**

- Create: `docs/tracing-span-tree-examples.md`

```markdown
# Tracing Span Tree Examples

This document shows example span trees for various operations.

## Capture Clipboard
```

command.clipboard.capture{device_id=abc123}
└─ usecase.capture_clipboard.execute{policy_version=v1}
├─ platform.macos.read_clipboard
│ └─ event: formats=3, total_size_bytes=1024
├─ event: representation selected, primary_rep_id=utf8
├─ infra.sqlite.insert_clipboard_event{table=clipboard_event}
├─ infra.blob.write{blob_id=xyz,size_bytes=1024}
└─ event: capture completed

```

```

**Step 4: Commit**

```bash
git add docs/tracing-span-tree-examples.md
git commit -m "docs: add tracing span tree examples

Document expected span tree structure for common operations."
```

---

## Phase 4: Cleanup (Optional)

### Task 4.1: Remove log crate dependencies

**Files:**

- Modify: All `Cargo.toml` files in src-tauri/crates

**Step 1: Remove log dependency from each crate**

For each crate that has `log = "0.4"`:

- Remove the line from `[dependencies]`

**Step 2: Update imports in code**

Find all `use log::` and replace with `use tracing::`

**Step 3: Replace log macros**

- `log::error!` → `tracing::error!`
- `log::warn!` → `tracing::warn!`
- `log::info!` → `tracing::info!`
- `log::debug!` → `tracing::debug!`
- `log::trace!` → `tracing::trace!`

**Step 4: Run full test suite**

Run: `cd src-tauri && cargo test`
Expected: All tests pass

**Step 5: Commit**

```bash
git add src-tauri/crates
git commit -m "refactor: remove log crate, use tracing exclusively

Migration complete. All logging now uses tracing crate."
```

---

### Task 4.2: Remove tauri-plugin-log (optional)

**Warning:** Only do this if Webview output is no longer needed or has been replaced with custom tracing layer.

**Step 1: Verify Webview output not needed**

Check if Webview console logs are still required for development.

**Step 2: Create custom Webview writer for tracing**

**Files:**

- Create: `src-tauri/crates/uc-tauri/src/bootstrap/webview_writer.rs`

```rust
//! Custom tracing writer for Tauri Webview output

use std::io::Write;

pub struct WebviewWriter {
    app_handle: tauri::AppHandle,
}

impl Write for WebviewWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let msg = String::from_utf8_lossy(buf);
        let _ = self.app_handle.emit_all("log://log", msg.to_string());
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
```

**Step 3: Update tracing.rs to use WebviewWriter**

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/tracing.rs`

Update the writer configuration.

**Step 4: Remove tauri-plugin-log from dependencies**

**Files:**

- Modify: `src-tauri/crates/uc-tauri/Cargo.toml`

Remove:

```toml
tauri-plugin-log = "2"
```

**Step 5: Run cargo check**

Run: `cd src-tauri && cargo check`
Expected: No errors

**Step 6: Commit**

```bash
git add src-tauri/crates/uc-tauri/
git commit -m "refactor: replace tauri-plugin-log with custom tracing writer

Pure tracing architecture without log crate dependency."
```

---

### Task 4.3: Update all documentation

**Files:**

- Modify: `docs/architecture/logging-architecture.md`
- Modify: `CLAUDE.md`
- Modify: `README.md` (if it mentions logging)

**Step 1: Update architecture doc**

Change to reflect pure tracing architecture.

**Step 2: Update CLAUDE.md**

Update logging section to use tracing examples.

**Step 3: Commit**

```bash
git add docs/
git commit -m "docs: update documentation for pure tracing architecture"
```

---

## Verification Checklist

After completing all phases:

- [ ] All crates compile without errors
- [ ] All tests pass
- [ ] App runs in development mode
- [ ] Logs appear with correct format
- [ ] Span trees are visible with RUST_LOG=debug
- [ ] All commands create root spans
- [ ] All use cases create child spans
- [ ] All infra/platform operations create debug spans
- [ ] No log crate references remain (if Phase 4 complete)
- [ ] Documentation updated

---

## Rollback Plan

If issues arise during migration:

1. **Identify the breaking commit**: `git log --oneline`
2. **Revert to last working state**: `git revert <commit>`
3. **Report issue**: Document what failed
4. **Fix and retry**: Address the issue and attempt migration again

The gradual migration strategy allows rolling back individual phases without losing work on completed phases.

---

**End of Implementation Plan**
