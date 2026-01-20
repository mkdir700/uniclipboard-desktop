# Clipboard Materialization Refactor Design

**Date:** 2025-01-18
**Author:** Refactoring Team
**Status:** Design Draft

## Problem Statement

The term "materialize/物化" is currently used in three distinct contexts within the clipboard capture and selection flow, creating significant cognitive overhead:

1. **`ClipboardRepresentationMaterializerPort`** (in `CaptureClipboardUseCase`)
   - Converts `ObservedClipboardRepresentation` → `PersistedClipboardRepresentation`
   - Actual semantic: **type conversion / normalization**

2. **`BlobMaterializerPort`** (in `MaterializeClipboardSelectionUseCase`)
   - Writes `&[u8]` → blob store, returns `blob_id`
   - Actual semantic: **persistence with deduplication**

3. **`MaterializeClipboardSelectionUseCase`**
   - Orchestrates the above two steps plus entry/selection loading
   - Actual semantic: **coordination**

When reading code, developers must trace into implementations to understand which "materialize" is being used, violating the principle that names should be self-descriptive.

## Design Principles

### 1. Semantic Singularization

Each term expresses exactly one action:

| Old Term                  | New Term            | Responsibility                          |
| ------------------------- | ------------------- | --------------------------------------- |
| `materialize` (capture)   | `normalize`         | Structure conversion: platform → domain |
| `materialize` (selection) | `resolve`           | On-demand loading: write blob if needed |
| `materialize` (blob)      | `write` / `persist` | Persistence: bytes → blob store         |

### 2. Read/Write Path Separation

- **Write Path** - Record facts only, defer blob writing
- **Read Path** - Resolve on-demand, write blob only when necessary

### 3. Idempotence & Concurrency Safety

- `Resolve` operations must be idempotent: multiple resolves of the same representation should not write duplicate blobs
- Concurrent safety strategies (choose one):
  - **Option A**: Blob store uses content-addressed dedupe (hash as key), eventual consistency on write-back
  - **Option B**: `update_blob_id` uses compare-and-set semantics ("only update when blob_id is None")

## Architecture Design

### Component Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         Write Path                              │
│                                                                 │
│  SystemClipboardSnapshot                                        │
│           │                                                     │
│           ▼                                                     │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ CaptureClipboardUseCase                                   │  │
│  │                                                            │  │
│  │ 1. representation_normalizer.normalize(observed)         │  │
│  │    → PersistedClipboardRepresentation {                  │  │
│  │         inline_data: Some(full/preview/placeholder),      │  │
│  │         blob_id: None                                    │  │
│  │       }                                                   │  │
│  │                                                            │  │
│  │ 2. event_writer.insert_event(event, representations)     │  │
│  │                                                            │  │
│  │ 3. policy.select(snapshot) → SelectionDecision            │  │
│  │                                                            │  │
│  │ 4. entry_repo.save_entry_and_selection(entry, selection) │  │
│  └──────────────────────────────────────────────────────────┘  │
│           │                                                     │
│           ▼                                                     │
│  Event + Representations + Entry + Selection                   │
│  (persisted, blob_id=None)                                     │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                          Read Path                              │
│                                                                 │
│  EntryId                                                        │
│     │                                                           │
│     ▼                                                           │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ ResolveClipboardSelectionPayloadUseCase                   │  │
│  │                                                            │  │
│  │ 1. selection_resolver.resolve_selection(entry_id)        │  │
│  │    → (entry, representation)                              │  │
│  │                                                            │  │
│  │ 2. payload_resolver.resolve(representation)              │  │
│  │    ├─ inline available? → Inline { mime, bytes }         │  │
│  │    ├─ blob_id exists? → BlobRef { mime, blob_id }        │  │
│  │    └─ else:                                              │  │
│  │       ├─ load raw bytes                                  │  │
│  │       ├─ calculate ContentHash                           │  │
│  │       ├─ blob_writer.write(bytes, hash) → Blob           │  │
│  │       ├─ rep_repo.update_blob_id_if_none(rep_id, blob_id)│  │
│  │       └─ BlobRef { mime, blob_id }                       │  │
│  └──────────────────────────────────────────────────────────┘  │
│           │                                                     │
│           ▼                                                     │
│  ResolvedClipboardPayload (UI/paste-ready)                     │
└─────────────────────────────────────────────────────────────────┘
```

## Port Definitions

### 1. ClipboardRepresentationNormalizerPort

**Responsibility:** Convert platform-layer `ObservedClipboardRepresentation` to domain-layer `PersistedClipboardRepresentation`

**Input:** `ObservedClipboardRepresentation`
**Output:** `PersistedClipboardRepresentation`

**Post-conditions:**

- Returned `PersistedClipboardRepresentation` contains valid metadata (mime, size)
- `inline_data` populated by strategy:
  - Small data (< threshold): full storage
  - Large data: preview/placeholder
- `blob_id` initially `None`

```rust
#[async_trait::async_trait]
pub trait ClipboardRepresentationNormalizerPort: Send + Sync {
    async fn normalize(
        &self,
        observed: &ObservedClipboardRepresentation,
    ) -> anyhow::Result<PersistedClipboardRepresentation>;
}
```

### 2. BlobWriterPort (replaces BlobMaterializerPort)

**Responsibility:** Write raw bytes to blob store, return blob reference

**Input:** `&[u8]` + `ContentHash`
**Output:** `Blob` (containing `blob_id`)

**Idempotence guarantee:**

- Data with identical `ContentHash` is written only once
- Returns existing `Blob` or newly created one

**Concurrency safety:**

- Uses `ContentHash` as content-addressed key
- `find_by_hash()` + `insert_blob()` combination provides natural deduplication

```rust
#[async_trait::async_trait]
pub trait BlobWriterPort: Send + Sync {
    /// Write bytes to blob store with deduplication by content hash
    async fn write(&self, data: &[u8], content_hash: &ContentHash) -> anyhow::Result<Blob>;
}
```

### 3. ClipboardPayloadResolverPort

**Responsibility:** Given a `PersistedClipboardRepresentation`, return directly usable payload

**Input:** `PersistedClipboardRepresentation`
**Output:** `ResolvedClipboardPayload`

**Resolution rules:**

1. **Prefer inline**: If `inline_data` available and complete → return `Inline`
2. **Has blob**: If `blob_id` exists → return `BlobRef`
3. **Lazy write**: Otherwise:
   - Load raw bytes (from inline_data or temp storage)
   - Calculate `ContentHash`
   - Call `BlobWriterPort::write()` to persist
   - Write back `representation.blob_id` (idempotent)
   - Return `BlobRef`

**Idempotence guarantee:**

- Multiple resolves of same rep yield identical `blob_id`
- Concurrent resolve: `update_blob_id` only takes effect when `None`

```rust
#[async_trait::async_trait]
pub trait ClipboardPayloadResolverPort: Send + Sync {
    async fn resolve(
        &self,
        representation: &PersistedClipboardRepresentation,
    ) -> anyhow::Result<ResolvedClipboardPayload>;
}

pub enum ResolvedClipboardPayload {
    Inline { mime: String, bytes: Vec<u8> },
    BlobRef { mime: String, blob_id: BlobId },
}
```

**Concurrency safety constraint:**

- `RepresentationRepository::update_blob_id()` must support conditional update:

```rust
async fn update_blob_id_if_none(
    &self,
    rep_id: &RepresentationId,
    blob_id: &BlobId,
) -> anyhow::Result<bool>;
```

### 4. SelectionResolverPort

**Responsibility:** Given `EntryId`, load complete selection context

**Input:** `EntryId`
**Output:** `(ClipboardEntry, PersistedClipboardRepresentation)`

**Loading flow:**

1. Load `ClipboardEntry` from `EntryRepository` (get `event_id`)
2. Load `SelectionDecision` from `SelectionRepository` (get `primary_rep_id`)
3. Load target `PersistedClipboardRepresentation` from `RepresentationRepository`

```rust
#[async_trait::async_trait]
pub trait SelectionResolverPort: Send + Sync {
    async fn resolve_selection(
        &self,
        entry_id: &EntryId,
    ) -> anyhow::Result<(ClipboardEntry, PersistedClipboardRepresentation)>;
}
```

### 5. BlobStorePort (unchanged)

Low-level storage interface: read/write byte streams by `blob_id`

```rust
#[async_trait::async_trait]
pub trait BlobStorePort: Send + Sync {
    async fn put(&self, blob_id: &BlobId, data: &[u8]) -> anyhow::Result<String>;
    async fn get(&self, blob_id: &BlobId) -> anyhow::Result<Vec<u8>>;
}
```

## Migration Plan

### Phase 1: Port Layer Refactor (Breaking Changes)

**Delete old Ports, create new Ports:**

```
uc-core/src/ports/clipboard/
├── representation_materializer.rs      [DELETE]
├── representation_normalizer.rs        [NEW]
├── payload_resolver.rs                 [NEW]
└── selection_resolver.rs               [NEW]

uc-core/src/ports/
├── blob_materializer.rs                [DELETE]
└── blob_writer.rs                      [NEW, replacement]
```

**Actions:**

1. Delete old files
2. Create new Port files
3. Global search-and-replace all references (use IDE refactoring tools)

### Phase 2: Infrastructure Layer Adaptation

**Rename + new implementations:**

```
uc-infra/src/
├── blob/
│   └── blob_materializer.rs → blob_writer.rs
├── clipboard/
│   └── representation_normalizer.rs   [NEW, refactored from materializer logic]
└── clipboard/
    └── payload_resolver.rs            [NEW]
```

### Phase 3: UseCase Layer Migration

**CaptureClipboardUseCase (minor adjustment):**

```rust
// Old code
let materialized_reps = try_join_all(
    snapshot.representations
        .iter()
        .map(|rep| self.representation_materializer.materialize(rep))
).await?;

// New code
let normalized_reps = try_join_all(
    snapshot.representations
        .iter()
        .map(|rep| self.representation_normalizer.normalize(rep))
).await?;
```

**MaterializeClipboardSelectionUseCase → ResolveClipboardSelectionPayloadUseCase (refactor):**

```rust
// Old use case: mixed responsibilities
pub struct MaterializeClipboardSelectionUseCase { ... }

// New use case: clear responsibilities
pub struct ResolveClipboardSelectionPayloadUseCase {
    selection_resolver: Arc<dyn SelectionResolverPort>,
    payload_resolver: Arc<dyn ClipboardPayloadResolverPort>,
}

pub async fn execute(&self, entry_id: &EntryId) -> Result<ResolvedClipboardPayload> {
    let (_entry, rep) = self.selection_resolver.resolve_selection(entry_id).await?;
    self.payload_resolver.resolve(&rep).await
}
```

### Phase 4: Dependency Injection Updates

**`src-tauri/crates/uc-app/src/deps.rs`:**

```rust
// Old code
representation_materializer: Arc<dyn ClipboardRepresentationMaterializerPort>,
blob_materializer: Arc<dyn BlobMaterializerPort>,

// New code
representation_normalizer: Arc<dyn ClipboardRepresentationNormalizerPort>,
blob_writer: Arc<dyn BlobWriterPort>,
payload_resolver: Arc<dyn ClipboardPayloadResolverPort>,
selection_resolver: Arc<dyn SelectionResolverPort>,
```

### Phase 5: Cleanup

1. Remove any remaining references to old Ports
2. Update documentation and comments
3. Run test suite to verify

## Impact Assessment

**File Change Count:**

| Layer          | Delete   | New | Refactor |
| -------------- | -------- | --- | -------- |
| Port           | 2        | 4   | -        |
| Infrastructure | 1 rename | 2   | 1        |
| UseCase        | 1        | 1   | 1        |
| DI Config      | -        | -   | 1        |

**Total:** ~10 files

**Benefits:**

- ✅ Semantic clarity: each term has one meaning
- ✅ Read/write path separation enables lazy blob writing
- ✅ Better testability: smaller, focused components
- ✅ Concurrency safety built into design

**Risks:**

- ⚠️ Breaking change requires coordinated updates across layers
- ⚠️ Existing tests may need updates

## Post-Conditions

### Write Path (Capture)

- ✅ event, representations, entry, selection all persisted
- ✅ primary_rep_id available
- ❌ blob_id not guaranteed (lazy write)

### Read Path (Resolve)

- ✅ Return value always consumable
- ✅ If blob path taken, blob written and `representation.blob_id` updated
- ✅ Idempotent: multiple resolves produce consistent result
