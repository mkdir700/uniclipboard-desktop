# Tracing Migration Phase 3: Infra/Platform Layer Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add debug-level spans to Infra and Platform layer I/O operations, completing the distributed tracing stack with fine-grained instrumentation for database, blob storage, and system operations.

**Architecture:** Infra and Platform layers create `debug_span` for significant I/O operations (SQLite queries, blob reads/writes, platform clipboard access). These automatically nest under UseCase spans, providing full traceability from user intent to system facts.

**Tech Stack:** `tracing` 0.1 (debug_span!, debug!, macros), Infra files in `src-tauri/crates/uc-infra/src/`, Platform files in `src-tauri/crates/uc-platform/src/`.

---

## Prerequisites

**Required**: Phase 2 (UseCase Layer) must be completed before starting this phase.

**Verification**: Run `RUST_LOG=info bun tauri dev` and trigger a command. Confirm you see nested Command → UseCase spans.

---

## Part 1: Infra Layer (Database and Storage)

### Task 1: Add tracing imports to uc-infra lib.rs

**Files:**

- Modify: `src-tauri/crates/uc-infra/src/lib.rs`

**Step 1: Read current lib.rs**

Run: `head -20 src-tauri/crates/uc-infra/src/lib.rs`
Expected: See module declarations.

**Step 2: Add tracing import**

Add to imports section:

```rust
// Tracing support for infra layer instrumentation
pub use tracing;
```

**Step 3: Run cargo check**

Run: `cargo check -p uc-infra`
Expected: No errors.

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-infra/src/lib.rs
git commit -m "refactor(uc-infra): re-export tracing for infra layer

Make tracing available to all infra modules.

Part of Phase 3: Infra/Platform layer migration"
```

---

### Task 2: Add debug span to clipboard_entry_repo queries

**Files:**

- Modify: `src-tauri/crates/uc-infra/src/db/repositories/clipboard_entry_repo.rs`

**Step 1: Read current repository**

Run: `cat src-tauri/crates/uc-infra/src/db/repositories/clipboard_entry_repo.rs`

**Step 2: Add debug spans to repository methods**

Wrap database operations with debug spans:

```rust
use tracing::debug_span;

// In the impl block, add spans to key methods:
impl SqliteClipboardEntryRepository {
    async fn get_entries(&self, limit: usize, offset: usize) -> anyhow::Result<Vec<ClipboardEntry>> {
        let span = debug_span!(
            "infra.sqlite.query_clipboard_entries",
            table = "clipboard_entry",
            limit = limit,
            offset = offset,
        );
        let _enter = span.enter();

        // ... existing query implementation ...
    }

    async fn save_entry(&self, entry: &ClipboardEntry) -> anyhow::Result<()> {
        let span = debug_span!(
            "infra.sqlite.insert_clipboard_entry",
            table = "clipboard_entry",
            entry_id = %entry.id,
        );
        let _enter = span.enter();

        // ... existing insert implementation ...
    }
}
```

**Step 3: Run cargo check**

Run: `cargo check -p uc-infra`
Expected: No errors.

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-infra/src/db/repositories/clipboard_entry_repo.rs
git commit -m "feat(infra): add debug spans to clipboard_entry_repo

Add debug_span for database operations.
- query_clipboard_entries: table, limit, offset
- insert_clipboard_entry: table, entry_id

Span level: debug
Nests under: UseCase spans

Part of Phase 3: Infra layer migration"
```

---

### Task 3: Add debug span to clipboard_event_repo operations

**Files:**

- Modify: `src-tauri/crates/uc-infra/src/db/repositories/clipboard_event_repo.rs`

**Step 1: Read current repository**

Run: `cat src-tauri/crates/uc-infra/src/db/repositories/clipboard_event_repo.rs`

**Step 2: Add debug spans**

Add spans to event and representation operations:

```rust
use tracing::debug_span;

impl SqliteClipboardEventRepository {
    async fn insert_event(&self, event: &ClipboardEvent) -> anyhow::Result<()> {
        let span = debug_span!(
            "infra.sqlite.insert_clipboard_event",
            table = "clipboard_event",
            event_id = %event.id,
        );
        let _enter = span.enter();

        // ... existing implementation ...
    }

    async fn insert_representations(&self, representations: &[SnapshotRepresentation]) -> anyhow::Result<()> {
        let span = debug_span!(
            "infra.sqlite.insert_representations",
            table = "snapshot_representation",
            count = representations.len(),
        );
        let _enter = span.enter();

        // ... existing implementation ...
    }
}
```

**Step 3: Run cargo check**

Run: `cargo check -p uc-infra`
Expected: No errors.

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-infra/src/db/repositories/clipboard_event_repo.rs
git commit -m "feat(infra): add debug spans to clipboard_event_repo

Add debug_span for event and representation operations.
- insert_clipboard_event: table, event_id
- insert_representations: table, count

Span level: debug
Nests under: UseCase spans

Part of Phase 3: Infra layer migration"
```

---

### Task 4: Add debug span to blob_repo operations

**Files:**

- Modify: `src-tauri/crates/uc-infra/src/db/repositories/blob_repo.rs`

**Step 1: Read current repository**

Run: `cat src-tauri/crates/uc-infra/src/db/repositories/blob_repo.rs`

**Step 2: Add debug spans**

Add spans to blob operations:

```rust
use tracing::debug_span;

impl SqliteBlobRepository {
    async fn save_blob(&self, blob_id: &str, data: &[u8]) -> anyhow::Result<()> {
        let span = debug_span!(
            "infra.sqlite.insert_blob",
            table = "blob",
            blob_id = %blob_id,
            size_bytes = data.len(),
        );
        let _enter = span.enter();

        // ... existing implementation ...
    }

    async fn get_blob(&self, blob_id: &str) -> anyhow::Result<Option<Vec<u8>>> {
        let span = debug_span!(
            "infra.sqlite.query_blob",
            table = "blob",
            blob_id = %blob_id,
        );
        let _enter = span.enter();

        // ... existing implementation ...
    }
}
```

**Step 3: Run cargo check**

Run: `cargo check -p uc-infra`
Expected: No errors.

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-infra/src/db/repositories/blob_repo.rs
git commit -m "feat(infra): add debug spans to blob_repo

Add debug_span for blob storage operations.
- insert_blob: table, blob_id, size_bytes
- query_blob: table, blob_id

Span level: debug
Nests under: UseCase spans

Part of Phase 3: Infra layer migration"
```

---

### Task 5: Add debug span to blob_materializer

**Files:**

- Modify: `src-tauri/crates/uc-infra/src/blob/blob_materializer.rs`

**Step 1: Read current materializer**

Run: `cat src-tauri/crates/uc-infra/src/blob/blob_materializer.rs`

**Step 2: Add debug span**

Add span to materialization operation:

```rust
use tracing::debug_span;

impl ClipboardRepresentationMaterializer for FsBlobMaterializer {
    async fn materialize(&self, representation: &ObservedClipboardRepresentation) -> anyhow::Result<MaterializedRepresentation> {
        let span = debug_span!(
            "infra.blob.materialize_representation",
            format_id = %representation.format_id,
            size_hint = representation.bytes.len(),
        );
        let _enter = span.enter();

        // ... existing implementation ...
    }
}
```

**Step 3: Run cargo check**

Run: `cargo check -p uc-infra`
Expected: No errors.

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-infra/src/blob/blob_materializer.rs
git commit -m "feat(infra): add debug span to blob_materializer

Add debug_span for representation materialization.
- format_id, size_hint

Span level: debug
Nests under: UseCase spans

Part of Phase 3: Infra layer migration"
```

---

## Part 2: Platform Layer (System Operations)

### Task 6: Add tracing imports to uc-platform lib.rs

**Files:**

- Modify: `src-tauri/crates/uc-platform/src/lib.rs`

**Step 1: Read current lib.rs**

Run: `head -20 src-tauri/crates/uc-platform/src/lib.rs`

**Step 2: Add tracing import**

Add to imports:

```rust
// Tracing support for platform layer instrumentation
pub use tracing;
```

**Step 3: Run cargo check**

Run: `cargo check -p uc-platform`
Expected: No errors.

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-platform/src/lib.rs
git commit -m "refactor(uc-platform): re-export tracing for platform layer

Make tracing available to all platform modules.

Part of Phase 3: Infra/Platform layer migration"
```

---

### Task 7: Add debug span to macOS clipboard adapter

**Files:**

- Modify: `src-tauri/crates/uc-platform/src/clipboard/platform/macos.rs`

**Step 1: Read current adapter**

Run: `cat src-tauri/crates/uc-platform/src/clipboard/platform/macos.rs`

**Step 2: Add debug spans**

Add spans to clipboard operations:

```rust
use tracing::{debug_span, debug};

impl SystemClipboardPort for MacosClipboard {
    fn read_snapshot(&self) -> Result<SystemClipboardSnapshot> {
        let span = debug_span!(
            "platform.macos.read_clipboard",
        );
        let _enter = span.enter();

        // ... existing implementation ...

        debug!(
            formats = snapshot.representations.len(),
            total_size_bytes = snapshot.total_size_bytes(),
            "Captured system clipboard snapshot"
        );

        Ok(snapshot)
    }

    fn write_snapshot(&self, snapshot: SystemClipboardSnapshot) -> Result<()> {
        let span = debug_span!(
            "platform.macos.write_clipboard",
            representations = snapshot.representations.len(),
        );
        let _enter = span.enter();

        // ... existing implementation ...

        debug!("Wrote clipboard snapshot to system");
        Ok(())
    }
}
```

**Step 3: Run cargo check**

Run: `cargo check -p uc-platform`
Expected: No errors.

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-platform/src/clipboard/platform/macos.rs
git commit -m "feat(platform): add debug spans to macOS clipboard adapter

Add debug_span for clipboard operations.
- read_clipboard: (logs formats, total_size_bytes)
- write_clipboard: representations

Span level: debug
Logs system facts (OS-level information)

Part of Phase 3: Platform layer migration"
```

---

### Task 8: Add debug span to Linux clipboard adapter

**Files:**

- Modify: `src-tauri/crates/uc-platform/src/clipboard/platform/linux.rs`

**Step 1: Add debug spans (similar to macOS)**

Follow the same pattern as Task 7, adapting for Linux-specific implementation.

**Step 2: Run cargo check**

Run: `cargo check -p uc-platform`
Expected: No errors.

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-platform/src/clipboard/platform/linux.rs
git commit -m "feat(platform): add debug spans to Linux clipboard adapter

Add debug_span for clipboard operations.
Same pattern as macOS adapter.

Span level: debug

Part of Phase 3: Platform layer migration"
```

---

### Task 9: Add debug span to Windows clipboard adapter

**Files:**

- Modify: `src-tauri/crates/uc-platform/src/clipboard/platform/windows.rs`

**Step 1: Add debug spans (similar to macOS)**

Follow the same pattern as Task 7, adapting for Windows-specific implementation.

**Step 2: Run cargo check**

Run: `cargo check -p uc-platform`
Expected: No errors.

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-platform/src/clipboard/platform/windows.rs
git commit -m "feat(platform): add debug spans to Windows clipboard adapter

Add debug_span for clipboard operations.
Same pattern as macOS adapter.

Span level: debug

Part of Phase 3: Platform layer migration"
```

---

### Task 10: Add debug span to encryption adapter

**Files:**

- Modify: `src-tauri/crates/uc-platform/src/adapters/encryption.rs`

**Step 1: Read current adapter**

Run: `cat src-tauri/crates/uc-platform/src/adapters/encryption.rs`

**Step 2: Add debug spans**

Add spans to encryption operations:

```rust
use tracing::{debug_span, debug};

impl EncryptionPort for StrongholdEncryptionAdapter {
    async fn initialize(&self, passphrase: &str) -> anyhow::Result<()> {
        let span = debug_span!(
            "platform.stronghold.initialize_encryption",
        );
        let _enter = span.enter();

        // ... existing implementation ...

        debug!("Encryption initialized successfully");
        Ok(())
    }

    async fn encrypt(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        let span = debug_span!(
            "platform.stronghold.encrypt",
            data_len = data.len(),
        );
        let _enter = span.enter();

        // ... existing implementation ...

        debug!(result_len = result.len(), "Data encrypted");
        Ok(result)
    }

    async fn decrypt(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        let span = debug_span!(
            "platform.stronghold.decrypt",
            data_len = data.len(),
        );
        let _enter = span.enter();

        // ... existing implementation ...

        debug!(result_len = result.len(), "Data decrypted");
        Ok(result)
    }
}
```

**Note**: Do NOT log actual data content (sensitive).

**Step 3: Run cargo check**

Run: `cargo check -p uc-platform`
Expected: No errors.

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-platform/src/adapters/encryption.rs
git commit -m "feat(platform): add debug spans to encryption adapter

Add debug_span for encryption operations.
- initialize_encryption
- encrypt: data_len, result_len
- decrypt: data_len, result_len
SECURITY: No sensitive data logged.

Span level: debug

Part of Phase 3: Platform layer migration"
```

---

## Part 3: Verification

### Task 11: Verify complete span tree

**Files:**

- None (verification step)

**Step 1: Start development server with debug logging**

Run: `RUST_LOG=debug bun tauri dev`
Expected: Application starts with verbose logging.

**Step 2: Trigger a clipboard capture operation**

Either:

- Trigger from frontend UI, or
- Copy something to clipboard (triggers callback)

**Step 3: Observe complete span tree in terminal**

Look for the full hierarchy:

```
2025-01-15 10:30:45.100 INFO [clipboard.rs:42] [command.clipboard.get_entries{device_id=abc,limit=50}] Starting query
2025-01-15 10:30:45.101 INFO [list_clipboard_entries.rs:15] [usecase.list_clipboard_entries.execute{limit=50,offset=0}] Starting query
2025-01-15 10:30:45.102 DEBUG [clipboard_entry_repo.rs:25] [infra.sqlite.query_clipboard_entries{table=\"clipboard_entry\",limit=50,offset=0}] Executing query
2025-01-15 10:30:45.105 DEBUG [clipboard_entry_repo.rs:35] [infra.sqlite.query_clipboard_entries{table=\"clipboard_entry\",limit=50,offset=0}] Query returned 10 entries
2025-01-15 10:30:45.106 INFO [list_clipboard_entries.rs:25] [usecase.list_clipboard_entries.execute{limit=50,offset=0}] Retrieved 10 entries
2025-01-15 10:30:45.107 INFO [clipboard.rs:52] [command.clipboard.get_entries{device_id=abc,limit=50}] Retrieved clipboard entries
```

**Expected span tree structure:**

```
command.clipboard.get_entries{device_id,limit}
└─ usecase.list_clipboard_entries.execute{limit,offset}
   └─ infra.sqlite.query_clipboard_entries{table,limit,offset}
```

**Step 4: Verify platform layer spans**

Trigger a clipboard change and look for platform spans:

```
2025-01-15 10:31:00.200 INFO [capture_clipboard.rs:105] [usecase.capture_clipboard.execute{source=callback,representations=3}] Starting capture
2025-01-15 10:31:00.201 DEBUG [macos.rs:45] [platform.macos.read_clipboard] Capturing system clipboard
2025-01-15 10:31:00.202 DEBUG [macos.rs:55] [platform.macos.read_clipboard] Captured system clipboard snapshot: formats=3, total_size_bytes=1024
2025-01-15 10:31:00.203 DEBUG [clipboard_entry_repo.rs:65] [infra.sqlite.insert_clipboard_entry{table=\"clipboard_entry\",entry_id=xyz}] Inserting entry
2025-01-15 10:31:00.204 INFO [capture_clipboard.rs:125] [usecase.capture_clipboard.execute{source=callback,representations=3}] Capture completed
```

**Step 5: Verify span field naming**

Check that all fields follow naming conventions:

- IDs: `entry_id`, `event_id`, `blob_id`
- Sizes: `size_bytes`, `total_size_bytes`, `data_len`
- Counts: `count`, `representations`, `limit`, `offset`
- Tables: `table = \"clipboard_entry\"` (quoted string)
- Booleans: `is_initialized`, `is_encrypted`

**Step 6: Verify security constraints**

Confirm that sensitive data is NOT logged:

- No passphrase in encryption spans
- No actual data content in encryption/decryption
- Only lengths and counts

**Step 7: Create verification note**

```bash
cat > /tmp/phase3_verification.md << 'EOF'
# Phase 3 Verification Results

## Date
$(date +%Y-%m-%d)

## Infra Layer Spans
- [x] SQLite query/insert operations (clipboard_entry_repo, clipboard_event_repo, blob_repo)
- [x] Blob materialization (blob_materializer)
- [x] All spans use debug level
- [x] Table names and IDs logged

## Platform Layer Spans
- [x] macOS clipboard adapter
- [x] Linux clipboard adapter
- [x] Windows clipboard adapter
- [x] Encryption adapter
- [x] All spans use debug level
- [x] System facts logged (formats, sizes)
- [x] No sensitive data logged

## Complete Span Tree Verified
```

command.clipboard.get_entries{device_id,limit}
└─ usecase.list_clipboard_entries.execute{limit,offset}
└─ infra.sqlite.query_clipboard_entries{table,limit,offset}

```

## Span Field Naming
- [x] IDs use _id suffix
- [x] Sizes use _bytes suffix
- [x] Tables quoted as strings
- [x] No sensitive data logged

## Log Output Sample
(Paste sample output from Step 3/4)

## Migration Complete
All phases (0, 1, 2, 3) now complete.
Application has full distributed tracing stack.

## Optional Next Steps
Phase 4: Remove log crate dependency (optional, requires full validation)
EOF
cat /tmp/phase3_verification.md
```

**Step 8: Commit verification milestone**

```bash
git add docs/plans/2025-01-15-tracing-phase3-implementation.md
git commit --allow-empty -m "test(verification): Phase 3 Infra/Platform layer complete

All Infra and Platform operations now have debug spans:
- Infra: SQLite queries, blob storage, materialization
- Platform: Clipboard adapters (macOS/Linux/Windows), encryption

Complete span tree verified:
command → usecase → infra/platform

Span naming conventions followed:
- IDs: *_id suffix
- Sizes: *_bytes suffix
- Security: No sensitive data logged

Migration Status:
- Phase 0: ✅ Infrastructure
- Phase 1: ✅ Command layer
- Phase 2: ✅ UseCase layer
- Phase 3: ✅ Infra/Platform layer
- Phase 4: ⏸️ Optional (remove log crate)

Application has full distributed tracing capability."
```

---

## Summary

After completing all tasks, the Infra and Platform layers will have complete debug-level span coverage:

**Infra Layer Spans**:

- `infra.sqlite.query_clipboard_entries`: table, limit, offset
- `infra.sqlite.insert_clipboard_entry`: table, entry_id
- `infra.sqlite.insert_clipboard_event`: table, event_id
- `infra.sqlite.insert_representations`: table, count
- `infra.sqlite.insert_blob`: table, blob_id, size_bytes
- `infra.sqlite.query_blob`: table, blob_id
- `infra.blob.materialize_representation`: format_id, size_hint

**Platform Layer Spans**:

- `platform.macos.read_clipboard`: (logs formats, total_size_bytes)
- `platform.macos.write_clipboard`: representations
- `platform.linux.read_clipboard`: (same as macOS)
- `platform.linux.write_clipboard`: representations
- `platform.windows.read_clipboard`: (same as macOS)
- `platform.windows.write_clipboard`: representations
- `platform.stronghold.initialize_encryption`: (logs success)
- `platform.stronghold.encrypt`: data_len, result_len
- `platform.stronghold.decrypt`: data_len, result_len

**Complete Span Tree**:

```
command.clipboard.get_entries{device_id=abc,limit=50}
└─ usecase.list_clipboard_entries.execute{limit=50,offset=0}
   └─ infra.sqlite.query_clipboard_entries{table="clipboard_entry",limit=50,offset=0}
      └─ debug: Query returned 10 entries
```

**Span Level Guide**:

- Command: `INFO` (user intent)
- UseCase: `INFO` (business lifecycle)
- Infra: `DEBUG` (I/O operations)
- Platform: `DEBUG` (system facts)

**Next Phase (Optional)**:

- Phase 4: Remove `log` crate dependency
- Requires full validation that all logging has migrated to `tracing`

**Verification Command**:

```bash
RUST_LOG=debug bun tauri dev
```

Should see complete 4-layer span hierarchy with all fields and debug-level I/O operations.

---

## References

- Design document: `docs/plans/2025-01-15-tracing-migration-design.md`
- Phase 0 plan: `docs/plans/2025-01-15-tracing-phase0-implementation.md`
- Phase 1 plan: `docs/plans/2025-01-15-tracing-phase1-implementation.md`
- Phase 2 plan: `docs/plans/2025-01-15-tracing-phase2-implementation.md`
- Infra spec: Section 5 of design document
- Platform spec: Section 6 of design document
- Field naming: Section 2.2 of design document
