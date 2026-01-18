# Snapshot Cache + Background Blob Worker Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a three-layer availability system (Memory Cache → Disk Spool → Blob Store) for large clipboard payloads with non-blocking capture and restart recovery.

**Architecture:**

- Capture path writes only to DB + memory cache + queues (no disk I/O)
- SpoolerTask writes to disk spool asynchronously
- BackgroundBlobWorker materializes blobs from cache→spool with atomic deduplication
- Resolver reads only (inline/blob-ready) or re-queues (staged/processing)

**Tech Stack:**

- Rust async (tokio) with channels (mpsc)
- SQLite with Diesel ORM (transactional updates)
- Bounded memory cache (tokio::sync::Mutex)
- Disk spool with OS cache dir + secure permissions

---

## Overview

This plan implements the design in `docs/plans/2026-01-18-snapshot-cache-background-worker-design.md`.

**Problem:** Large clipboard payloads (images) lose original bytes during normalize phase because `inline_data: Some(vec![])` is used as placeholder without preserving snapshot bytes.

**Solution:** Three-phase approach:

1. **Phase 0**: Foundation - State machine + core ports
2. **Phase 1**: Infrastructure - Cache + Spool + Tasks
3. **Phase 2**: Integration - Wiring + recovery + testing

---

## Phase 0: Foundation (Domain Model + Ports)

**Estimated:** 3-4 hours
**Goal:** Establish explicit state machine and atomic port contracts

### Task 1: Add PayloadAvailability State Enum

**Files:**

- Create: `src-tauri/crates/uc-core/src/clipboard/payload_availability.rs`
- Modify: `src-tauri/crates/uc-core/src/clipboard/mod.rs`

**Step 1: Write the failing test**

Create test file: `src-tauri/crates/uc-core/src/clipboard/payload_availability.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_state_requires_inline_data() {
        let state = PayloadAvailability::Inline;
        assert_eq!(state.requires_inline_data(), true);
        assert_eq!(state.requires_blob_id(), false);
    }

    #[test]
    fn test_blob_ready_state_requires_blob_id() {
        let state = PayloadAvailability::BlobReady;
        assert_eq!(state.requires_inline_data(), false);
        assert_eq!(state.requires_blob_id(), true);
    }

    #[test]
    fn test_staged_state_requires_neither() {
        let state = PayloadAvailability::Staged;
        assert_eq!(state.requires_inline_data(), false);
        assert_eq!(state.requires_blob_id(), false);
    }
}
```

**Step 2: Run test to verify it fails**

```bash
cd src-tauri && cargo test payload_availability --lib
```

Expected: Compile error "PayloadAvailability not found"

**Step 3: Write minimal implementation**

```rust
/// Explicit state machine for clipboard payload availability.
///
/// Key principle: This enum ONLY expresses state, never carries data.
/// Data carriers are inline_data and blob_id on PersistedClipboardRepresentation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayloadAvailability {
    /// Small content stored inline - inline_data=Some, blob_id=None
    Inline,

    /// Blob materialized and ready - inline_data=None, blob_id=Some
    BlobReady,

    /// Large content awaiting worker - inline_data=None, blob_id=None
    /// Data should be available in cache or spool
    Staged,

    /// Worker is processing - inline_data=None, blob_id=None
    Processing,

    /// Worker failed - inline_data=None, blob_id=None
    Failed { last_error: String },

    /// Data permanently lost - inline_data=None, blob_id=None
    Lost,
}

impl PayloadAvailability {
    /// Whether this state requires inline_data to be Some
    pub fn requires_inline_data(&self) -> bool {
        matches!(self, PayloadAvailability::Inline)
    }

    /// Whether this state requires blob_id to be Some
    pub fn requires_blob_id(&self) -> bool {
        matches!(self, PayloadAvailability::BlobReady)
    }

    /// Whether data should be available in cache or spool
    pub fn is_cache_or_spool_expected(&self) -> bool {
        matches!(self, PayloadAvailability::Staged | PayloadAvailability::Processing)
    }
}
```

**Step 4: Run test to verify it passes**

```bash
cd src-tauri && cargo test payload_availability --lib
```

Expected: PASS

**Step 5: Export from clipboard module**

Modify: `src-tauri/crates/uc-core/src/clipboard/mod.rs`

```rust
mod availability; // Add this line
// ... existing mods ...
pub use availability::PayloadAvailability; // Add this line
```

**Step 6: Commit**

```bash
git add src-tauri/crates/uc-core/src/clipboard/payload_availability.rs
git add src-tauri/crates/uc-core/src/clipboard/mod.rs
git commit -m "feat(core): add PayloadAvailability state enum"
```

---

### Task 2: Update PersistedClipboardRepresentation

**Files:**

- Modify: `src-tauri/crates/uc-core/src/clipboard/snapshot.rs`

**Step 1: Write the failing test**

Add to `src-tauri/crates/uc-core/src/clipboard/snapshot.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_representation_valid_state() {
        let rep = PersistedClipboardRepresentation::new(
            RepresentationId::new(),
            FormatId::new(),
            Some(MimeType::text_plain()),
            100,
            Some(vec![1, 2, 3]),
            None,
        );
        assert!(rep.is_inline());
        assert_eq!(rep.payload_state(), PayloadAvailability::Inline);
    }

    #[test]
    fn test_staged_representation_no_inline_no_blob() {
        let rep = PersistedClipboardRepresentation::new_staged(
            RepresentationId::new(),
            FormatId::new(),
            Some(MimeType::from_str("image/png").unwrap()),
            1024000,
        );
        assert_eq!(rep.inline_data, None);
        assert_eq!(rep.blob_id, None);
        assert_eq!(rep.payload_state(), PayloadAvailability::Staged);
    }
}
```

**Step 2: Run test to verify it fails**

```bash
cd src-tauri && cargo test persisted_representation --lib
```

Expected: Compile error "missing methods: new_staged, payload_state"

**Step 3: Update implementation**

Modify: `src-tauri/crates/uc-core/src/clipboard/snapshot.rs`

```rust
use crate::ids::{FormatId, RepresentationId};
use crate::{BlobId, MimeType};
use crate::clipboard::PayloadAvailability; // Add import

#[derive(Debug, Clone)]
pub struct PersistedClipboardRepresentation {
    pub id: RepresentationId,
    pub format_id: FormatId,
    pub mime_type: Option<MimeType>,
    pub size_bytes: i64,
    pub inline_data: Option<Vec<u8>>,
    pub blob_id: Option<BlobId>,
    pub payload_state: PayloadAvailability, // Add field
    pub last_error: Option<String>, // Add field
}

impl PersistedClipboardRepresentation {
    pub fn new(
        id: RepresentationId,
        format_id: FormatId,
        mime_type: Option<MimeType>,
        size_bytes: i64,
        inline_data: Option<Vec<u8>>,
        blob_id: Option<BlobId>,
    ) -> Self {
        let payload_state = match (&inline_data, &blob_id) {
            (Some(_), None) => PayloadAvailability::Inline,
            (None, Some(_)) => PayloadAvailability::BlobReady,
            _ => PayloadAvailability::Staged, // Default for new creations
        };

        Self {
            id,
            format_id,
            mime_type,
            size_bytes,
            inline_data,
            blob_id,
            payload_state,
            last_error: None,
        }
    }

    /// Create a staged representation for large content awaiting blob materialization
    pub fn new_staged(
        id: RepresentationId,
        format_id: FormatId,
        mime_type: Option<MimeType>,
        size_bytes: i64,
    ) -> Self {
        Self {
            id,
            format_id,
            mime_type,
            size_bytes,
            inline_data: None,
            blob_id: None,
            payload_state: PayloadAvailability::Staged,
            last_error: None,
        }
    }

    pub fn payload_state(&self) -> PayloadAvailability {
        self.payload_state.clone()
    }

    // ... keep existing is_inline, is_blob methods ...
}
```

**Step 4: Run test to verify it passes**

```bash
cd src-tauri && cargo test persisted_representation --lib
```

Expected: PASS

**Step 5: Update database model and migration**

Create migration: `src-tauri/crates/uc-infra/migrations/2026-01-18-000001_add_payload_state/up.sql`

```sql
ALTER TABLE clipboard_snapshot_representation ADD COLUMN payload_state TEXT DEFAULT 'Staged';
ALTER TABLE clipboard_snapshot_representation ADD COLUMN last_error TEXT;

-- Update existing rows
UPDATE clipboard_snapshot_representation SET payload_state = 'Inline' WHERE inline_data IS NOT NULL;
UPDATE clipboard_snapshot_representation SET payload_state = 'BlobReady' WHERE blob_id IS NOT NULL;
UPDATE clipboard_snapshot_representation SET payload_state = 'Staged' WHERE inline_data IS NULL AND blob_id IS NULL;

-- Make payload_state NOT NULL after migration
-- Note: SQLite doesn't support ALTER COLUMN directly, use CREATE TABLE approach if needed
```

**Step 6: Update schema**

Run: `cd src-tauri && diesel migration run`

**Step 7: Update SnapshotRepresentationRow model**

Modify: `src-tauri/crates/uc-infra/src/db/models/snapshot_representation.rs`

```rust
#[derive(Queryable)]
#[diesel(table_name = clipboard_snapshot_representation)]
pub struct SnapshotRepresentationRow {
    pub id: String,
    pub event_id: String,
    pub format_id: String,
    pub mime_type: Option<String>,
    pub size_bytes: i64,
    pub inline_data: Option<Vec<u8>>,
    pub blob_id: Option<String>,
    pub payload_state: Option<String>, // Add
    pub last_error: Option<String>,    // Add
}
```

**Step 8: Commit**

```bash
git add src-tauri/crates/uc-core/src/clipboard/snapshot.rs
git add src-tauri/crates/uc-infra/migrations/2026-01-18-000001_add_payload_state
git add src-tauri/crates/uc-infra/src/db/models/snapshot_representation.rs
git commit -m "feat: add payload_state and last_error to representation"
```

---

### Task 3: Update BlobWriterPort with Atomic Semantics

**Files:**

- Modify: `src-tauri/crates/uc-core/src/ports/blob_writer.rs`

**Step 1: Write the failing test**

Create: `src-tauri/crates/uc-core/src/ports/blob_writer_test.rs`

```rust
use uc_core::ports::BlobWriterPort;
use uc_core::{Blob, BlobId, ContentHash, BlobStorageLocator, EncryptionAlgo};

#[async_trait::async_trait]
impl BlobWriterPort for MockBlobWriter {
    async fn write_if_absent(&self, content_id: &ContentHash, encrypted_bytes: &[u8]) -> anyhow::Result<Blob> {
        // Mock implementation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_write_if_absent_returns_existing_blob() {
        // Should return existing BlobId if content_id already exists
    }

    #[tokio::test]
    async fn test_write_if_absent_creates_new_blob() {
        // Should create new Blob if content_id doesn't exist
    }
}
```

**Step 2: Run test to verify it fails**

Expected: Method `write_if_absent` not found

**Step 3: Update port definition**

Modify: `src-tauri/crates/uc-core/src/ports/blob_writer.rs`

```rust
//! Blob Writer Port
//!
//! This port writes raw bytes to blob store with deduplication.
//!
//! **Semantic:** "write_if_absent" = atomic write-if-absent with deduplication

use crate::{Blob, ContentHash};

#[async_trait::async_trait]
pub trait BlobWriterPort: Send + Sync {
    /// Write bytes to blob store if content_id doesn't exist.
    ///
    /// # Atomic semantics
    /// - If `content_id` already exists → return existing BlobId
    /// - If `content_id` doesn't exist → write and return new BlobId
    ///
    /// # Idempotence guarantee
    /// - Multiple concurrent calls with same content_id produce same BlobId
    /// - Data is written only once per content_id
    ///
    /// # Parameters
    /// - `content_id`: Hash-based identifier for deduplication (use keyed hash)
    /// - `encrypted_bytes`: Encrypted payload to persist
    async fn write_if_absent(
        &self,
        content_id: &ContentHash,
        encrypted_bytes: &[u8],
    ) -> anyhow::Result<Blob>;

    /// Legacy write method (deprecated, use write_if_absent)
    #[deprecated(note = "Use write_if_absent for atomic semantics")]
    async fn write(&self, data: &[u8], content_hash: &ContentHash) -> anyhow::Result<Blob> {
        // Default implementation calls write_if_absent
        self.write_if_absent(content_hash, data).await
    }
}
```

**Step 4: Update implementation**

Modify: `src-tauri/crates/uc-infra/src/db/repositories/blob_repo.rs`

```rust
#[async_trait]
impl BlobWriterPort for DieselBlobRepository<E> {
    async fn write_if_absent(
        &self,
        content_id: &ContentHash,
        encrypted_bytes: &[u8],
    ) -> Result<Blob> {
        let content_hash_str = content_id.to_string();
        let blob_id = BlobId::new();

        // Try to find existing blob by content hash
        if let Some(existing) = self.find_by_hash(content_id).await? {
            return Ok(existing);
        }

        // Insert new blob (UNIQUE constraint on content_hash provides atomicity)
        // ... implementation ...
    }
}
```

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-core/src/ports/blob_writer.rs
git add src-tauri/crates/uc-infra/src/db/repositories/blob_repo.rs
git commit -m "feat(ports): add atomic write_if_absent to BlobWriterPort"
```

---

### Task 4: Add CAS Update to Representation Repository

**Files:**

- Modify: `src-tauri/crates/uc-core/src/ports/clipboard/representation_repository.rs`
- Modify: `src-tauri/crates/uc-infra/src/db/repositories/representation_repo.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn test_update_processing_result_cas() {
    // Should only update when current state is in expected_states
    // Should return error if state doesn't match
}
```

**Step 2: Run test to verify it fails**

**Step 3: Add port method**

Modify: `src-tauri/crates/uc-core/src/ports/clipboard/representation_repository.rs`

```rust
use crate::clipboard::{PersistedClipboardRepresentation, PayloadAvailability};
use crate::ids::{EventId, RepresentationId};
use crate::BlobId;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ClipboardRepresentationRepositoryPort: Send + Sync {
    // ... existing methods ...

    /// Atomically update blob_id and payload_state with CAS semantics.
    ///
    /// # Transactional update
    /// - Single UPDATE statement sets blob_id, payload_state, last_error
    /// - WHERE clause filters by expected_states
    /// - Returns updated row or error if no rows matched
    ///
    /// # Concurrency safety
    /// - Only updates if current state is in expected_states
    /// - Returns error if state changed by another worker
    async fn update_processing_result(
        &self,
        rep_id: &RepresentationId,
        expected_states: &[PayloadAvailability],
        blob_id: Option<&BlobId>,
        new_state: PayloadAvailability,
        last_error: Option<&str>,
    ) -> Result<PersistedClipboardRepresentation>;
}
```

**Step 4: Implement with Diesel**

Modify: `src-tauri/crates/uc-infra/src/db/repositories/representation_repo.rs`

```rust
async fn update_processing_result(
    &self,
    rep_id: &RepresentationId,
    expected_states: &[PayloadAvailability],
    blob_id: Option<&BlobId>,
    new_state: PayloadAvailability,
    last_error: Option<&str>,
) -> Result<PersistedClipboardRepresentation> {
    // Use transaction with WHERE IN clause for CAS
}
```

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-core/src/ports/clipboard/representation_repository.rs
git add src-tauri/crates/uc-infra/src/db/repositories/representation_repo.rs
git commit -m "feat(repo): add update_processing_result with CAS"
```

---

## Phase 1: Infrastructure (Cache + Spool + Workers)

**Estimated:** 6-8 hours
**Goal:** Build three-layer availability with bounded cache and spool

### Task 5: Implement RepresentationCache

**Files:**

- Create: `src-tauri/crates/uc-infra/src/clipboard/representation_cache.rs`

**Step 1: Write test**

```rust
#[tokio::test]
async fn test_cache_put_get() {
    let cache = RepresentationCache::new(100, 10000);
    let rep_id = RepresentationId::new();
    let bytes = vec![1, 2, 3];

    cache.put(&rep_id, bytes.clone()).await;
    let retrieved = cache.get(&rep_id).await;
    assert_eq!(retrieved, Some(bytes));
}

#[tokio::test]
async fn test_cache_eviction_when_full() {
    // Should evict oldest entry when max_entries reached
}

#[tokio::test]
async fn test_cache_eviction_when_bytes_limit() {
    // Should evict until under max_bytes
}
```

**Step 2: Implement**

```rust
use tokio::sync::Mutex;
use uc_core::ids::RepresentationId;

pub struct RepresentationCache {
    inner: Mutex<Inner>,
}

struct Inner {
    entries: HashMap<RepresentationId, CacheEntry>,
    queue: VecDeque<RepresentationId>,
    max_entries: usize,
    max_bytes: usize,
    current_bytes: usize,
}

struct CacheEntry {
    raw_bytes: Vec<u8>,
    status: CacheEntryStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CacheEntryStatus {
    Pending,
    Processing,
    Completed,
}

impl RepresentationCache {
    pub fn new(max_entries: usize, max_bytes: usize) -> Self {
        Self {
            inner: Mutex::new(Inner {
                entries: HashMap::new(),
                queue: VecDeque::new(),
                max_entries,
                max_bytes,
                current_bytes: 0,
            }),
        }
    }

    // All methods take &self (not &mut self) with internal Mutex
    pub async fn put(&self, rep_id: &RepresentationId, bytes: Vec<u8>) {
        let mut inner = self.inner.lock().await;
        // Evict if needed, then insert
    }

    pub async fn get(&self, rep_id: &RepresentationId) -> Option<Vec<u8>> {
        let inner = self.inner.lock().await;
        inner.entries.get(rep_id).map(|e| e.raw_bytes.clone())
    }
}
```

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-infra/src/clipboard/representation_cache.rs
git commit -m "feat(infra): add bounded RepresentationCache"
```

---

### Task 6: Implement SpoolManager

**Files:**

- Create: `src-tauri/crates/uc-infra/src/clipboard/spool_manager.rs`

**Step 1: Write test**

```rust
#[tokio::test]
async fn test_spool_write_read() {
    let temp_dir = tempfile::tempdir().unwrap();
    let spool = SpoolManager::new(temp_dir.path(), 1000000);

    let rep_id = RepresentationId::new();
    let bytes = vec![1, 2, 3];

    spool.write(&rep_id, &bytes).await.unwrap();
    let retrieved = spool.read(&rep_id).await.unwrap();
    assert_eq!(retrieved, bytes);
}

#[tokio::test]
async fn test_spool_delete_after_blob() {
    // Should delete spool entry after successful blob write
}
```

**Step 2: Implement**

```rust
use std::path::PathBuf;
use uc_core::ids::RepresentationId;
use tokio::fs;

pub struct SpoolManager {
    spool_dir: PathBuf,
    max_bytes: usize,
}

pub struct SpoolEntry {
    pub representation_id: RepresentationId,
    pub file_path: PathBuf,
    pub size: usize,
}

impl SpoolManager {
    pub fn new(spool_dir: PathBuf, max_bytes: usize) -> Self {
        fs::create_dir_all(&spool_dir).await.unwrap();
        // Set permissions 0700 on Unix
        Self { spool_dir, max_bytes }
    }

    pub async fn write(&self, rep_id: &RepresentationId, bytes: &[u8]) -> anyhow::Result<SpoolEntry> {
        let file_path = self.spool_dir.join(rep_id.to_string());
        fs::write(&file_path, bytes).await?;
        // Set permissions 0600 on Unix
        Ok(SpoolEntry {
            representation_id: rep_id.clone(),
            file_path,
            size: bytes.len(),
        })
    }

    pub async fn read(&self, rep_id: &RepresentationId) -> anyhow::Result<Option<Vec<u8>>> {
        let file_path = self.spool_dir.join(rep_id.to_string());
        match fs::read(&file_path).await {
            Ok(bytes) => Ok(Some(bytes)),
            Err(_) => Ok(None),
        }
    }

    pub async fn delete(&self, rep_id: &RepresentationId) -> anyhow::Result<()> {
        let file_path = self.spool_dir.join(rep_id.to_string());
        fs::remove_file(file_path).await.ok();
        Ok(())
    }
}
```

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-infra/src/clipboard/spool_manager.rs
git commit -m "feat(infra): add SpoolManager with secure permissions"
```

---

### Task 7: Implement SpoolerTask

**Files:**

- Create: `src-tauri/crates/uc-infra/src/clipboard/spooler_task.rs`

**Step 1: Write test**

```rust
#[tokio::test]
async fn test_spooler_task_writes_to_disk() {
    // SpoolerTask should write incoming requests to disk spool
}

#[tokio::test]
async fn test_spooler_task_backpressure() {
    // try_send should fail when queue is full
}
```

**Step 2: Implement**

```rust
use tokio::sync::mpsc;
use uc_core::ids::RepresentationId;

pub struct SpoolRequest {
    pub rep_id: RepresentationId,
    pub bytes: Vec<u8>,
}

pub struct SpoolerTask {
    spool_rx: mpsc::Receiver<SpoolRequest>,
    spool_manager: std::sync::Arc<SpoolManager>,
}

impl SpoolerTask {
    pub fn new(
        spool_rx: mpsc::Receiver<SpoolRequest>,
        spool_manager: std::sync::Arc<SpoolManager>,
    ) -> Self {
        Self {
            spool_rx,
            spool_manager,
        }
    }

    pub async fn run(mut self) {
        while let Some(req) = self.spool_rx.recv().await {
            if let Err(e) = self.spool_manager.write(&req.rep_id, &req.bytes).await {
                tracing::error!("Failed to write spool: {}", e);
            }
        }
    }
}
```

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-infra/src/clipboard/spooler_task.rs
git commit -m "feat(infra): add SpoolerTask for async disk writes"
```

---

### Task 8: Implement BackgroundBlobWorker

**Files:**

- Create: `src-tauri/crates/uc-infra/src/clipboard/background_blob_worker.rs`

**Step 1: Write test**

```rust
#[tokio::test]
async fn test_worker_processes_staged_representations() {
    // Should read from cache, write blob, update state to BlobReady
}

#[tokio::test]
async fn test_worker_falls_back_to_spool() {
    // Should read from spool if cache miss
}

#[tokio::test]
async fn test_worker_retries_on_transient_error() {
    // Should retry with backoff
}
```

**Step 2: Implement**

```rust
use tokio::sync::mpsc;
use uc_core::ids::RepresentationId;
use uc_core::clipboard::PayloadAvailability;
use std::sync::Arc;
use std::time::Duration;

pub struct BackgroundBlobWorker {
    worker_rx: mpsc::Receiver<RepresentationId>,
    cache: Arc<RepresentationCache>,
    spool: Arc<SpoolManager>,
    repo: Arc<dyn ClipboardRepresentationRepositoryPort>,
    blob_writer: Arc<dyn BlobWriterPort>,
    encryption: Arc<dyn EncryptionPort>,
    hasher: Arc<dyn ContentHashPort>,
    retry_max_attempts: u32,
    retry_backoff: Duration,
}

impl BackgroundBlobWorker {
    pub fn new(/* deps */) -> Self {
        Self { /* ... */ }
    }

    pub async fn run(mut self) {
        while let Some(rep_id) = self.worker_rx.recv().await {
            if let Err(e) = self.process_representation(rep_id).await {
                tracing::error!("Failed to process representation: {}", e);
            }
        }
    }

    async fn process_representation(&self, rep_id: RepresentationId) -> anyhow::Result<()> {
        // 1. Optional CAS: Staged -> Processing
        // 2. Get bytes: cache.get -> spool.read
        // 3. Hash with keyed hash
        // 4. Encrypt with AAD
        // 5. blob_writer.write_if_absent
        // 6. repo CAS update to BlobReady
        // 7. Delete spool, mark cache completed
        Ok(())
    }
}
```

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-infra/src/clipboard/background_blob_worker.rs
git commit -m "feat(infra): add BackgroundBlobWorker"
```

---

## Phase 2: Integration (Wiring + Capture Flow)

**Estimated:** 4-5 hours
**Goal:** Wire components and update capture flow

### Task 9: Update Normalizer to Use Staged State

**Files:**

- Modify: `src-tauri/crates/uc-infra/src/clipboard/normalizer.rs`

**Step 1: Write test**

```rust
#[tokio::test]
async fn test_normalizer_creates_stated_for_large_content() {
    // Large content should have payload_state=Staged, inline_data=None, blob_id=None
}

#[tokio::test]
async fn test_normalizer_creates_inline_for_small_content() {
    // Small content should have payload_state=Inline, inline_data=Some
}
```

**Step 2: Update normalizer**

Change from `Some(vec![])` placeholder to `new_staged()`:

```rust
async fn normalize(&self, observed: &ObservedClipboardRepresentation) -> Result<PersistedClipboardRepresentation> {
    let size_bytes = observed.bytes.len() as i64;

    if size_bytes <= self.config.inline_threshold_bytes {
        Ok(PersistedClipboardRepresentation::new(
            observed.id.clone(),
            observed.format_id.clone(),
            observed.mime.clone(),
            size_bytes,
            Some(observed.bytes.clone()),
            None,
        ))
    } else {
        // Large content: create staged representation
        Ok(PersistedClipboardRepresentation::new_staged(
            observed.id.clone(),
            observed.format_id.clone(),
            observed.mime.clone(),
            size_bytes,
        ))
    }
}
```

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-infra/src/clipboard/normalizer.rs
git commit -m "refactor(normalizer): use Staged state for large content"
```

---

### Task 10: Update Capture UseCase with Cache + Spool

**Files:**

- Modify: `src-tauri/crates/uc-app/src/usecases/internal/capture_clipboard.rs`

**Step 1: Update CaptureClipboardUseCase constructor**

Add cache and spool channels:

```rust
pub struct CaptureClipboardUseCase {
    // ... existing fields ...
    representation_cache: Arc<RepresentationCache>,
    spool_tx: mpsc::Sender<SpoolRequest>,
    worker_tx: mpsc::Sender<RepresentationId>,
}
```

**Step 2: Update execute_with_snapshot**

After `insert_event`, for large representations:

```rust
async fn execute_with_snapshot(&self, snapshot: SystemClipboardSnapshot) -> Result<EventId> {
    // ... existing code ...

    let normalized_reps = try_join_all(normalized_futures).await?;
    self.event_writer.insert_event(&new_event, &normalized_reps).await?;

    // NEW: Queue large representations for background processing
    for rep in &normalized_reps {
        if rep.payload_state() == PayloadAvailability::Staged {
            // Find original bytes from snapshot
            if let Some(observed) = snapshot.representations.iter().find(|o| o.id == rep.id) {
                // Put in cache
                self.representation_cache.put(&rep.id, observed.bytes.clone()).await;

                // Queue spool write (try_send, don't await)
                let _ = self.spool_tx.try_send(SpoolRequest {
                    rep_id: rep.id.clone(),
                    bytes: observed.bytes.clone(),
                });

                // Queue blob worker (try_send, don't await)
                let _ = self.worker_tx.try_send(rep.id.clone());
            }
        }
    }

    // ... rest of existing code ...
}
```

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/internal/capture_clipboard.rs
git commit -m "feat(capture): add cache+spool+worker queuing"
```

---

### Task 11: Update Resolver to Read-Only

**Files:**

- Find and modify payload resolver implementation

**Step 1: Update resolver logic**

```rust
async fn resolve(&self, representation: &PersistedClipboardRepresentation) -> anyhow::Result<ResolvedClipboardPayload> {
    match representation.payload_state() {
        PayloadAvailability::Inline => {
            // Return inline_data directly
        }
        PayloadAvailability::BlobReady => {
            // Read blob, decrypt, return
        }
        PayloadAvailability::Staged | PayloadAvailability::Processing | PayloadAvailability::Failed => {
            // Try cache.get -> spool.read
            // If found: return bytes + re-queue (try_send)
            // If not found: return error
        }
        PayloadAvailability::Lost => {
            // Return unrecoverable error
        }
    }
}
```

**Step 2: Commit**

```bash
git add <resolver-file>
git commit -m "refactor(resolver): read-only with re-queue"
```

---

### Task 12: Wire Components in Bootstrap

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs` (or equivalent)

**Step 1: Create channels**

```rust
let (spool_tx, spool_rx) = mpsc::channel(100);
let (worker_tx, worker_rx) = mpsc::channel(100);
```

**Step 2: Create shared components**

```rust
let cache = Arc::new(RepresentationCache::new(
    config.cache_max_entries,
    config.cache_max_bytes,
));

let spool_manager = Arc::new(SpoolManager::new(
    app_dirs.cache_dir.join("spool"),
    config.spool_max_bytes,
));
```

**Step 3: Spawn background tasks**

```rust
let spooler = SpoolerTask::new(spool_rx, spool_manager.clone());
tokio::spawn(async move {
    spooler.run().await;
});

let worker = BackgroundBlobWorker::new(
    worker_rx,
    cache.clone(),
    spool_manager.clone(),
    // ... other deps
);
tokio::spawn(async move {
    worker.run().await;
});
```

**Step 4: Register with Tauri**

```rust
app.manage(cache);
app.manage(spool_tx);
app.manage(worker_tx);
```

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/
git commit -m "feat(bootstrap): wire cache+spool+worker tasks"
```

---

## Phase 3: Testing + Recovery

**Estimated:** 3-4 hours
**Goal:** Ensure reliability and restart recovery

### Task 13: Add Integration Tests

**Files:**

- Create: `src-tauri/crates/uc-app/tests/snapshot_cache_integration_test.rs`

**Step 1: Test capture non-blocking**

```rust
#[tokio::test]
async fn test_capture_does_not_block_on_large_image() {
    // Capture 100 large images
    // Measure time - should be < 100ms for all captures (no disk I/O)
}
```

**Step 2: Test worker materializes blobs**

```rust
#[tokio::test]
async fn test_worker_materializes_all_blobs() {
    // Capture large content
    // Wait for worker
    // Assert all blob_ids are set
}
```

**Step 3: Test restart recovery**

```rust
#[tokio::test]
async fn test_spool_recovers_after_restart() {
    // Capture large content
    // Kill before worker finishes
    // Restart
    // Spool scanner should re-queue pending items
}
```

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-app/tests/
git commit -m "test: add snapshot cache integration tests"
```

---

### Task 14: Implement Spool Scanner for Recovery

**Files:**

- Create: `src-tauri/crates/uc-infra/src/clipboard/spool_scanner.rs`

**Step 1: Implement**

```rust
pub struct SpoolScanner {
    spool_dir: PathBuf,
    repo: Arc<dyn ClipboardRepresentationRepositoryPort>,
    worker_tx: mpsc::Sender<RepresentationId>,
}

impl SpoolScanner {
    pub async fn scan_and_recover(&self) -> anyhow::Result<usize> {
        // List all files in spool_dir
        // For each file:
        //   - Check if representation.state is Staged/Processing
        //   - If yes, re-queue to worker
        //   - If no (BlobReady), delete stale spool file
    }
}
```

**Step 2: Call from bootstrap**

```rust
// After wiring, before spawning tasks
let scanner = SpoolScanner::new(/* ... */);
let recovered = scanner.scan_and_recover().await?;
info!("Recovered {} representations from spool", recovered);
```

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-infra/src/clipboard/spool_scanner.rs
git commit -m "feat(infra): add SpoolScanner for restart recovery"
```

---

### Task 15: Stress Test

**Files:**

- Create: `src-tauri/crates/uc-app/tests/stress_test.rs`

**Step 1: Implement**

```rust
#[tokio::test]
#[ignore] // Run with cargo test -- --ignored
async fn stress_test_100_large_images() {
    // Capture 100 x 5MB images
    // Assert:
    //   - Capture time < 500ms total
    //   - No data loss
    //   - All blobs eventually materialized
    //   - Cache stays within bounds
}
```

**Step 2: Commit**

```bash
git add src-tauri/crates/uc-app/tests/stress_test.rs
git commit -m "test: add stress test for burst capture"
```

---

## Summary

**Total Estimated Time:** 16-21 hours

**Key Deliverables:**

1. Explicit state machine (PayloadAvailability)
2. Atomic port contracts (write_if_absent, update_processing_result)
3. Bounded memory cache (eviction by entries + bytes)
4. Disk spool with secure permissions
5. Non-blocking capture (try_send only)
6. Background blob worker with cache→spool fallback
7. Restart recovery via spool scanner

**Success Criteria:**

- Capture of 100 large images < 500ms (no disk I/O in capture path)
- Zero data loss (spool provides crash recovery)
- Idempotent blob writes (content-addressed storage)
- Bounded memory usage (cache eviction)
- Graceful degradation (cache-only if queues full)
