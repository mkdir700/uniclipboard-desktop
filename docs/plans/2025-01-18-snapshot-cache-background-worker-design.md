# Snapshot Cache and Background Blob Worker Design

**Date:** 2025-01-18
**Status:** Design Proposal
**Author:** Design Session with User

## Problem Statement

The current clipboard materialization refactor has an issue: when capturing large content (like images), the original data is lost during the `normalize` step because:

1. `ClipboardRepresentationNormalizer` returns `inline_data: Some(vec![])` as a placeholder
2. The original `snapshot.representations[].bytes` is not persisted
3. When users later access the content, `ClipboardPayloadResolver.load_raw_bytes()` fails

**User Requirements:**

- Don't write blobs during capture (should be independent)
- Use a background worker for async blob writing
- Cache recent snapshots (10 entries) for fast access
- Use FIFO eviction with dynamic expansion to prevent data loss

## Solution Overview

```
Capture → normalize → save to DB (blob_id=NULL)
         ↓
    save to SnapshotCache
         ↓
    notify BackgroundWorker
         ↓
Worker encrypts + writes blob → updates blob_id
```

## Architecture Components

### 1. SnapshotCache

**Location:** `uc-infra/src/clipboard/snapshot_cache.rs`

```rust
pub struct SnapshotCache {
    entries: VecDeque<CacheEntry>,
    initial_capacity: usize,  // 10
    current_capacity: usize,  // dynamically grows
}

struct CacheEntry {
    event_id: EventId,
    snapshot: SystemClipboardSnapshot,
    status: CacheEntryStatus,  // Pending | Processing | Completed
}
```

**Key Features:**

- FIFO eviction (oldest first)
- Status tracking: `Pending` → `Processing` → `Completed`
- Dynamic expansion when full:
  1. Try to remove `Completed` entries
  2. If none available, expand capacity
- Thread-safe: `Arc<Mutex<>>`

**Methods:**

- `put(event_id, snapshot)` - May block/explore when full
- `get(event_id)` - Returns snapshot if cached
- `mark_processing(event_id)` - Mark as being processed
- `mark_completed(event_id)` - Mark as done
- `remove(event_id)` - Remove after processing

### 2. BackgroundBlobWorker

**Location:** `uc-infra/src/clipboard/background_blob_worker.rs`

```rust
pub struct BackgroundBlobWorker {
    write_rx: mpsc::Receiver<EventId>,
    snapshot_cache: Arc<SnapshotCache>,
    blob_writer: Arc<dyn BlobWriterPort>,
    representation_repo: Arc<dyn ClipboardRepresentationRepositoryPort>,
    encryption: Arc<dyn EncryptionPort>,
    hasher: Arc<dyn ContentHashPort>,
    inline_threshold: i64,
}
```

**Worker Loop:**

```rust
while let Some(event_id) = write_rx.recv().await {
    snapshot = cache.get(&event_id);
    for rep in snapshot.representations {
        if rep.size > inline_threshold {
            hash = hasher.hash(&rep.bytes);
            encrypted = encryption.encrypt(&rep.bytes, &aad);
            blob = blob_writer.write(&encrypted, &hash);
            repo.update_blob_id_if_none(&rep.id, &blob.id);
        }
    }
    cache.mark_completed(&event_id);
}
```

**Key Features:**

- Independent tokio task (doesn't block capture)
- Channel buffer: 100 requests
- Processes each representation individually
- Only writes blobs above threshold
- Error isolation: single failure doesn't affect others

### 3. Updated ClipboardPayloadResolver

**Location:** `uc-infra/src/clipboard/payload_resolver.rs`

**New Dependencies:**

- `snapshot_cache: Arc<SnapshotCache>`
- `encryption: Arc<dyn EncryptionPort>`

**Resolve Logic:**

```rust
async fn resolve(representation, entry_id) {
    // 1. Inline data non-empty → return directly
    if let Some(data) = &representation.inline_data {
        if !data.is_empty() { return Inline(data); }
    }

    // 2. blob_id exists → read from blob_store
    if let Some(blob_id) = &representation.blob_id {
        encrypted = blob_repo.get(blob_id);
        decrypted = encryption.decrypt(encrypted);
        return Inline(decrypted);
    }

    // 3. Neither exists → immediate write from cache
    raw_bytes = load_from_snapshot_cache(&representation.id, entry_id);
    blob = write_blob_now(raw_bytes);
    repo.update_blob_id_if_none(&representation.id, &blob.id);
    return Inline(raw_bytes);
}
```

**Fallback Behavior:**

- If blob not written yet, load from cache
- Write immediately (synchronous, bypassing Worker)
- If cache also empty, return error with clear message

## Data Flow

### Capture Flow

```
┌─────────────────────────────────────────────────────────────┐
│ ClipboardWatcher detects change                             │
└─────────────────────────────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────────┐
│ CaptureClipboardUseCase.execute_with_snapshot(snapshot)     │
│                                                             │
│ 1. Normalize all representations                           │
│    → PersistedClipboardRepresentation {                     │
│        inline_data: Some(vec![]),  // placeholder           │
│        blob_id: None,              // to be written          │
│      }                                                         │
│                                                             │
│ 2. event_writer.insert_event(event, reps)                  │
│    → Save to database (blob_id is NULL)                      │
│                                                             │
│ 3. snapshot_cache.put(event_id, snapshot)  ← Cache it!     │
│                                                             │
│ 4. blob_writer_tx.send(event_id)  ← Notify Worker          │
└─────────────────────────────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────────┐
│ SnapshotCache (LruCache<EventId, Snapshot>)                 │
│ - Capacity: 10 (initial)                                     │
│ - Dynamically expands if full                                │
│ - FIFO eviction                                               │
└─────────────────────────────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────────┐
│ BackgroundBlobWorker (tokio task)                           │
│ - Receives event_id from channel                              │
│ - Loads snapshot from cache                                   │
│ - Encrypts + writes blobs                                     │
│ - Updates representation.blob_id                              │
│ - Marks cache entry as completed                              │
└─────────────────────────────────────────────────────────────┘
```

### User Access Flow

```
┌─────────────────────────────────────────────────────────────┐
│ Frontend: get_clipboard_entry_detail(entry_id)              │
└─────────────────────────────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────────┐
│ ResolveClipboardSelectionPayloadUseCase.execute(entry_id)    │
│                                                             │
│ 1. selection_resolver.resolve(entry_id)                     │
│    → Get (entry, representation)                              │
│                                                             │
│ 2. payload_resolver.resolve(representation, entry_id)       │
│                                                             │
│    Branch A: inline_data exists → return                     │
│    Branch B: blob_id exists → decrypt & return               │
│    Branch C: neither exists →                                │
│      - Load from snapshot_cache                              │
│      - Write blob immediately (sync)                         │
│      - Update blob_id                                        │
│      - Return data                                           │
└─────────────────────────────────────────────────────────────┘
```

## Dependency Injection

### Port Updates

```rust
// uc-core/src/ports/clipboard/payload_resolver.rs
#[async_trait::async_trait]
pub trait ClipboardPayloadResolverPort: Send + Sync {
    async fn resolve(
        &self,
        representation: &PersistedClipboardRepresentation,
        entry_id: &EntryId,  // ← New parameter
    ) -> Result<ResolvedClipboardPayload>;
}
```

### AppDeps Updates

```rust
// uc-app/src/deps.rs
pub struct AppDeps {
    // ... existing fields ...

    /// Snapshot cache for blob write worker
    pub snapshot_cache: Arc<SnapshotCache>,

    /// Channel sender for blob write requests
    pub blob_writer_tx: mpsc::Sender<EventId>,
}
```

### Wiring Updates

```rust
// uc-tauri/src/bootstrap/wiring.rs
pub fn create_app_runtime(config: &AppConfig) -> Result<(AppRuntime, JoinHandle<()>)> {
    // Create snapshot cache
    let snapshot_cache = uc_infra::clipboard::SnapshotCache::new();

    // Spawn background worker
    let (worker_tx, worker_handle) = uc_infra::clipboard::BackgroundBlobWorker::spawn(
        snapshot_cache.clone(),
        blob_writer.clone(),
        representation_repo.clone(),
        encryption.clone(),
        hasher.clone(),
        config.storage.inline_threshold_bytes,
    );

    // Update payload resolver with cache
    let payload_resolver = uc_infra::clipboard::ClipboardPayloadResolver::new(
        representation_repo.clone(),
        blob_writer.clone(),
        blob_repository.clone(),
        hasher.clone(),
        encryption.clone(),
        snapshot_cache.clone(),
    );

    // ... rest of wiring ...
}
```

## Error Handling

### Worker Error Handling

```rust
// Single representation failure doesn't stop others
if let Err(e) = self.process_representation(rep).await {
    tracing::error!(
        error = %e,
        representation_id = %rep.id,
        "Failed to write blob, will retry on user access"
    );
    // Don't retry - let user access trigger immediate write
}
```

### Cache Miss Error

```rust
// When user accesses content that's not in cache
Err(anyhow::anyhow!(
    "Snapshot not in cache for event {}. \
     The blob may have been lost due to cache eviction. \
     This is rare and suggests the worker hasn't finished processing.",
     event_id
))
```

## Testing Strategy

### Unit Tests

```rust
// uc-infra/src/clipboard/background_blob_worker_tests.rs

#[tokio::test]
async fn test_worker_processes_pending_snapshots() {
    // 1. Create cache and mock dependencies
    // 2. Put test snapshot in cache
    // 3. Send write request
    // 4. Verify blob_writer called
    // 5. Verify blob_id updated
}

#[tokio::test]
async fn test_cache_expansion_when_full() {
    // 1. Fill cache (10 entries)
    // 2. Add 11th entry
    // 3. Verify capacity expands to 11
}

#[tokio::test]
async fn test_payload_resolver_fallback_to_cache() {
    // 1. Create representation (no blob_id)
    // 2. snapshot_cache has data
    // 3. Call resolve
    // 4. Verify immediate write succeeds
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_end_to_end_capture_and_access() {
    // 1. Capture large image
    // 2. Verify entry saved (blob_id is NULL)
    // 3. Access before worker finishes
    // 4. Verify immediate write from cache
    // 5. Access after worker finishes
    // 6. Verify blob_id present
}
```

## Trade-offs

### Advantages

✅ **Fast capture** - No blocking I/O during clipboard monitoring
✅ **No data loss** - Dynamic expansion prevents eviction of pending data
✅ **Efficient** - Deduplication via content hash
✅ **Graceful degradation** - Immediate write if worker hasn't finished
✅ **Simple cache** - In-memory only, no persistence complexity

### Disadvantages

❌ **Memory usage** - 10 snapshots cached (varies by content size)
❌ **Cold start loss** - Unprocessed blobs lost on app restart
❌ **Complexity** - Additional worker and cache management

## Future Considerations

1. **Persistent cache** - If app restart loss becomes problematic
2. **Priority queue** - Process user-accessed entries first
3. **Progress reporting** - Show user which entries are being processed
4. **Configurable cache size** - Let users adjust based on memory

## Implementation Checklist

- [ ] Create `SnapshotCache` struct
- [ ] Create `BackgroundBlobWorker` struct
- [ ] Update `ClipboardPayloadResolverPort` signature
- [ ] Update `ClipboardPayloadResolver` implementation
- [ ] Update `CaptureClipboardUseCase` to use cache
- [ ] Update `AppDeps` struct
- [ ] Update `wiring.rs` to spawn worker
- [ ] Add unit tests for cache
- [ ] Add unit tests for worker
- [ ] Add integration tests
- [ ] Update documentation
