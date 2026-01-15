# Tracing Migration Phase 2: UseCase Layer Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add child spans to all UseCase execute methods in the uc-app layer, establishing business lifecycle tracing that automatically nests under Command layer root spans.

**Architecture:** Each UseCase execute() method creates a child `info_span` with consistent naming (`usecase.<name>.execute`) and policy metadata. Spans cover entire business logic, including all async operations and port calls.

**Tech Stack:** `tracing` 0.1 (info_span!, info!, debug!, macros), UseCase files in `src-tauri/crates/uc-app/src/usecases/`.

---

## Prerequisites

**Required**: Phase 1 (Command Layer) must be completed before starting this phase.

**Verification**: Run `RUST_LOG=info bun tauri dev` and trigger a command. Confirm you see root spans like `[command.clipboard.get_entries{...}]`.

---

## Task 1: Add tracing imports to uc-app lib.rs

**Files:**

- Modify: `src-tauri/crates/uc-app/src/lib.rs`

**Step 1: Read current lib.rs**

Run: `head -20 src-tauri/crates/uc-app/src/lib.rs`
Expected: See module declarations and exports.

**Step 2: Add tracing import**

Add to the imports section (after existing imports):

```rust
// Tracing support for use case instrumentation
pub use tracing;
```

**Step 3: Run cargo check to verify**

Run: `cargo check -p uc-app`
Expected: No errors.

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-app/src/lib.rs
git commit -m "refactor(uc-app): re-export tracing for use case layer

Make tracing macros available to all use case modules.

Part of Phase 2: UseCase layer migration"
```

---

## Task 2: Add tracing imports to usecases/mod.rs

**Files:**

- Modify: `src-tauri/crates/uc-app/src/usecases/mod.rs`

**Step 1: Read current mod.rs**

Run: `cat src-tauri/crates/uc-app/src/usecases/mod.rs`
Expected: See use case module exports.

**Step 2: Add tracing import at top**

Add at the top of the file (after doc comments if any):

```rust
// Use case tracing instrumentation
use tracing::{info_span, info, debug, error, warn};
```

**Step 3: Run cargo check to verify**

Run: `cargo check -p uc-app`
Expected: No errors.

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/mod.rs
git commit -m "refactor(usecases): add tracing imports to module

Import all tracing level macros for use case instrumentation.

Part of Phase 2: UseCase layer migration"
```

---

## Task 3: Add child span to ListClipboardEntries use case

**Files:**

- Modify: `src-tauri/crates/uc-app/src/usecases/list_clipboard_entries.rs`

**Step 1: Read current use case**

Run: `cat src-tauri/crates/uc-app/src/usecases/list_clipboard_entries.rs`
Expected: See the use case implementation.

**Step 2: Add tracing import and wrap execute() with span**

Modify the file to add span:

```rust
// Add at top with other imports
use tracing::{info_span, info};

// In the impl block, wrap the execute() method:
impl ListClipboardEntries {
    pub async fn execute(&self, limit: usize, offset: usize) -> anyhow::Result<Vec<uc_core::ClipboardEntry>> {
        // Create use case span (child of command's root span)
        let span = info_span!(
            "usecase.list_clipboard_entries.execute",
            limit = limit,
            offset = offset,
        );
        let _enter = span.enter();

        info!("Starting clipboard entries query");

        // ... existing implementation ...

        info!(count = result.len(), "Retrieved clipboard entries");
        result
    }
}
```

**Note**: Since the actual implementation may vary, adapt to wrap the existing logic. The key is:

1. Create span at start of execute()
2. Add info log at start
3. Add info log with count at end

**Step 3: Run cargo check to verify**

Run: `cargo check -p uc-app`
Expected: No errors.

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/list_clipboard_entries.rs
git commit -m "feat(usecases): add child span to ListClipboardEntries

Create info_span with limit and offset fields.
Log start and completion with entry count.

Span: usecase.list_clipboard_entries.execute
Fields: limit, offset

Nests under: command.clipboard.get_entries

Part of Phase 2: UseCase layer migration"
```

---

## Task 4: Add child span to DeleteClipboardEntry use case

**Files:**

- Modify: `src-tauri/crates/uc-app/src/usecases/delete_clipboard_entry.rs`

**Step 1: Read current use case**

Run: `cat src-tauri/crates/uc-app/src/usecases/delete_clipboard_entry.rs`

**Step 2: Add tracing import and wrap execute() with span**

Modify to add span instrumentation:

```rust
use tracing::{info_span, info};

impl DeleteClipboardEntry {
    pub async fn execute(&self, entry_id: &EntryId) -> anyhow::Result<()> {
        let span = info_span!(
            "usecase.delete_clipboard_entry.execute",
            entry_id = %entry_id,
        );
        let _enter = span.enter();

        info!(entry_id = %entry_id, "Starting clipboard entry deletion");

        // ... existing implementation with repository calls ...

        info!(entry_id = %entry_id, "Deleted clipboard entry successfully");
        Ok(())
    }
}
```

**Step 3: Run cargo check to verify**

Run: `cargo check -p uc-app`
Expected: No errors.

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/delete_clipboard_entry.rs
git commit -m "feat(usecases): add child span to DeleteClipboardEntry

Create info_span with entry_id field.
Log start and success with entry_id.

Span: usecase.delete_clipboard_entry.execute
Fields: entry_id

Nests under: command.clipboard.delete_entry

Part of Phase 2: UseCase layer migration"
```

---

## Task 5: Add child span to InitializeEncryption use case

**Files:**

- Modify: `src-tauri/crates/uc-app/src/usecases/initialize_encryption.rs`

**Step 1: Read current use case**

Run: `cat src-tauri/crates/uc-app/src/usecases/initialize_encryption.rs`

**Step 2: Add tracing import and wrap execute() with span**

Modify to add span instrumentation:

```rust
use tracing::{info_span, info};

impl InitializeEncryption {
    pub async fn execute(self, passphrase: Passphrase) -> anyhow::Result<()> {
        let span = info_span!(
            "usecase.initialize_encryption.execute",
        );
        let _enter = span.enter();

        info!("Starting encryption initialization");

        // ... existing implementation ...

        info!("Encryption initialized successfully");
        Ok(())
    }
}
```

**Note**: Do NOT include passphrase in span fields (sensitive data).

**Step 3: Run cargo check to verify**

Run: `cargo check -p uc-app`
Expected: No errors.

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/initialize_encryption.rs
git commit -m "feat(usecases): add child span to InitializeEncryption

Create info_span for encryption initialization.
Log start and success.
SECURITY: passphrase excluded from span fields.

Span: usecase.initialize_encryption.execute

Nests under: command.encryption.initialize

Part of Phase 2: UseCase layer migration"
```

---

## Task 6: Add child span to IsEncryptionInitialized use case

**Files:**

- Modify: `src-tauri/crates/uc-app/src/usecases/is_encryption_initialized.rs`

**Step 1: Read current use case**

Run: `cat src-tauri/crates/uc-app/src/usecases/is_encryption_initialized.rs`

**Step 2: Add tracing import and wrap execute() with span**

Modify to add span instrumentation:

```rust
use tracing::{info_span, info};

impl IsEncryptionInitialized {
    pub async fn execute(&self) -> anyhow::Result<bool> {
        let span = info_span!(
            "usecase.is_encryption_initialized.execute",
        );
        let _enter = span.enter();

        info!("Checking encryption initialization status");

        // ... existing implementation ...

        info!(is_initialized = result, "Encryption status checked");
        Ok(result)
    }
}
```

**Step 3: Run cargo check to verify**

Run: `cargo check -p uc-app`
Expected: No errors.

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/is_encryption_initialized.rs
git commit -m "feat(usecases): add child span to IsEncryptionInitialized

Create info_span for status check.
Log start and result (is_initialized).

Span: usecase.is_encryption_initialized.execute

Nests under: command.encryption.is_initialized

Part of Phase 2: UseCase layer migration"
```

---

## Task 7: Add child span to GetSettings use case

**Files:**

- Modify: `src-tauri/crates/uc-app/src/usecases/get_settings.rs`

**Step 1: Read current use case**

Run: `cat src-tauri/crates/uc-app/src/usecases/get_settings.rs`

**Step 2: Add tracing import and wrap execute() with span**

Modify to add span instrumentation:

```rust
use tracing::{info_span, info};

impl GetSettings {
    pub async fn execute(&self) -> anyhow::Result<uc_core::settings::model::Settings> {
        let span = info_span!(
            "usecase.get_settings.execute",
        );
        let _enter = span.enter();

        info!("Retrieving application settings");

        // ... existing implementation ...

        info!("Settings retrieved successfully");
        Ok(result)
    }
}
```

**Step 3: Run cargo check to verify**

Run: `cargo check -p uc-app`
Expected: No errors.

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/get_settings.rs
git commit -m "feat(usecases): add child span to GetSettings

Create info_span for settings retrieval.
Log start and success.

Span: usecase.get_settings.execute

Nests under: command.settings.get

Part of Phase 2: UseCase layer migration"
```

---

## Task 8: Add child span to UpdateSettings use case

**Files:**

- Modify: `src-tauri/crates/uc-app/src/usecases/update_settings.rs`

**Step 1: Read current use case**

Run: `cat src-tauri/crates/uc-app/src/usecases/update_settings.rs`

**Step 2: Add tracing import and wrap execute() with span**

Modify to add span instrumentation:

```rust
use tracing::{info_span, info};

impl UpdateSettings {
    pub async fn execute(&self, settings: uc_core::settings::model::Settings) -> anyhow::Result<()> {
        let span = info_span!(
            "usecase.update_settings.execute",
        );
        let _enter = span.enter();

        info!("Updating application settings");

        // ... existing implementation ...

        info!("Settings updated successfully");
        Ok(())
    }
}
```

**Step 3: Run cargo check to verify**

Run: `cargo check -p uc-app`
Expected: No errors.

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/update_settings.rs
git commit -m "feat(usecases): add child span to UpdateSettings

Create info_span for settings update.
Log start and success.

Span: usecase.update_settings.execute

Nests under: command.settings.update

Part of Phase 2: UseCase layer migration"
```

---

## Task 9: Add child span to CaptureClipboardUseCase (internal)

**Files:**

- Modify: `src-tauri/crates/uc-app/src/usecases/internal/capture_clipboard.rs`

**Step 1: Read current use case**

Run: `cat src-tauri/crates/uc-app/src/usecases/internal/capture_clipboard.rs`

**Step 2: Add tracing import and wrap execute() methods with span**

This use case has two execute methods. Modify both:

```rust
use tracing::{info_span, info, debug};

impl CaptureClipboardUseCase {
    pub async fn execute(&self) -> Result<EventId> {
        let span = info_span!(
            "usecase.capture_clipboard.execute",
            source = "platform_clipboard",
        );
        let _enter = span.enter();

        info!("Starting clipboard capture from platform");

        let snapshot = self.platform_clipboard_port.read_snapshot()?;
        debug!(
            representations = snapshot.representations.len(),
            "Captured system snapshot"
        );

        // ... rest of implementation ...

        info!(event_id = %event_id, "Clipboard capture completed");
        Ok(event_id)
    }

    pub async fn execute_with_snapshot(&self, snapshot: SystemClipboardSnapshot) -> Result<EventId> {
        let span = info_span!(
            "usecase.capture_clipboard.execute",
            source = "callback",
            representations = snapshot.representations.len(),
        );
        let _enter = span.enter();

        info!("Starting clipboard capture with provided snapshot");

        // ... rest of implementation ...

        info!(event_id = %event_id, "Clipboard capture completed");
        Ok(event_id)
    }
}
```

**Step 3: Run cargo check to verify**

Run: `cargo check -p uc-app`
Expected: No errors.

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/internal/capture_clipboard.rs
git commit -m "feat(usecases): add child span to CaptureClipboardUseCase

Create info_span for both execute methods.
- execute(): source=\"platform_clipboard\"
- execute_with_snapshot(): source=\"callback\"
Log representation count from snapshot.

Span: usecase.capture_clipboard.execute
Fields: source, representations

Called from: AppRuntime ClipboardChangeHandler callback

Part of Phase 2: UseCase layer migration"
```

---

## Task 10: Verify span hierarchy in development environment

**Files:**

- None (verification step)

**Step 1: Start development server with verbose logging**

Run: `RUST_LOG=info bun tauri dev`
Expected: Application starts.

**Step 2: Trigger a command from frontend**

Open the application and trigger an action (e.g., get clipboard entries).

**Step 3: Observe span hierarchy in terminal**

Look for nested span output:

```
2025-01-15 10:30:45.123 INFO [clipboard.rs:42] [command.clipboard.get_entries{device_id=abc123,limit=50}] Starting query
2025-01-15 10:30:45.124 INFO [list_clipboard_entries.rs:15] [usecase.list_clipboard_entries.execute{limit=50,offset=0}] Starting clipboard entries query
2025-01-15 10:30:45.125 INFO [list_clipboard_entries.rs:25] [usecase.list_clipboard_entries.execute{limit=50,offset=0}] Retrieved 10 entries
2025-01-15 10:30:45.126 INFO [clipboard.rs:52] [command.clipboard.get_entries{device_id=abc123,limit=50}] Retrieved clipboard entries
```

**Key observations:**

- Command span (root) appears first
- UseCase span (child) appears nested inside
- Fields are visible in braces
- Messages appear after span context

**Step 4: Test error handling span preservation**

Trigger an error condition (e.g., delete invalid entry_id):

```
2025-01-15 10:31:00.456 ERROR [clipboard.rs:72] [command.clipboard.delete_entry{device_id=abc,entry_id=invalid}] Failed to parse entry_id
```

**Step 5: Verify callback path (CaptureClipboard)**

Since CaptureClipboardUseCase is called from AppRuntime callback, trigger a clipboard change and check logs:

```
2025-01-15 10:32:00.789 INFO [capture_clipboard.rs:105] [usecase.capture_clipboard.execute{source=callback,representations=3}] Starting clipboard capture
```

**Step 6: Create verification note**

```bash
cat > /tmp/phase2_verification.md << 'EOF'
# Phase 2 Verification Results

## Date
$(date +%Y-%m-%d)

## Use Cases Migrated
- [x] ListClipboardEntries
- [x] DeleteClipboardEntry
- [x] InitializeEncryption
- [x] IsEncryptionInitialized
- [x] GetSettings
- [x] UpdateSettings
- [x] CaptureClipboardUseCase (both execute methods)

## Span Hierarchy Verified
- [x] Command spans (root) visible
- [x] UseCase spans (child) nested under commands
- [x] Fields present in both layers
- [x] Span format: usecase.<name>.execute
- [x] Callback path preserves spans

## Log Output Sample
(Paste sample output from Step 3)

## Span Tree Example
```

command.clipboard.get_entries{device_id=abc,limit=50}
└─ usecase.list_clipboard_entries.execute{limit=50,offset=0}
├─ info: Starting clipboard entries query
└─ info: Retrieved 10 entries

```

## Ready for Phase 3
Yes - UseCase layer child spans are in place.
Next: Migrate Infra/Platform layers for debug spans.

## Notes
- Callback path (CaptureClipboard) works correctly
- Error handling preserves span context
- No sensitive data logged (passphrase excluded)
EOF
cat /tmp/phase2_verification.md
```

**Step 7: Commit verification milestone**

```bash
git add docs/plans/2025-01-15-tracing-phase2-implementation.md
git commit --allow-empty -m "test(verification): Phase 2 UseCase layer complete

All UseCase execute methods now create child spans:
- 8 use cases migrated
- Consistent naming: usecase.<name>.execute
- Automatic nesting under command root spans
- Callback path verified (CaptureClipboard)

Span hierarchy verified:
command.clipboard.get_entries
└─ usecase.list_clipboard_entries.execute

Ready to proceed to Phase 3: Infra/Platform layer migration."
```

---

## Summary

After completing all tasks, the UseCase layer will have complete child span coverage:

**Use Cases Migrated**: 8 use cases

- Public use cases: 7 (ListClipboardEntries, DeleteClipboardEntry, InitializeEncryption, IsEncryptionInitialized, GetSettings, UpdateSettings)
- Internal use cases: 1 (CaptureClipboardUseCase with 2 execute methods)

**Span Naming Convention**:

- Format: `usecase.<name>.execute`
- Examples: `usecase.list_clipboard_entries.execute`, `usecase.initialize_encryption.execute`

**Span Fields**:

- Operation-specific: `limit`, `offset`, `entry_id`, `source`, `representations`
- Security-sensitive data excluded (e.g., passphrase)

**Span Hierarchy**:

```
command.clipboard.get_entries{device_id=abc,limit=50}
└─ usecase.list_clipboard_entries.execute{limit=50,offset=0}
```

**Next Phase**:

- Phase 3: Migrate Infra/Platform layers for debug-level spans
- Infra spans will nest under UseCase spans

**Verification Command**:

```bash
RUST_LOG=info bun tauri dev
```

Should see nested span output showing Command → UseCase hierarchy.

---

## References

- Design document: `docs/plans/2025-01-15-tracing-migration-design.md`
- Phase 0 plan: `docs/plans/2025-01-15-tracing-phase0-implementation.md`
- Phase 1 plan: `docs/plans/2025-01-15-tracing-phase1-implementation.md`
- UseCase spec: Section 4 of design document
- Span naming: Section 2.1 of design document
