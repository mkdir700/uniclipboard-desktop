# Clipboard Materialization Refactor Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Rename and restructure the clipboard materialization components to achieve semantic clarity by separating three overloaded meanings of "materialize" into focused terms: `normalize`, `resolve`, and `write`.

**Architecture:** This refactor follows Hexagonal Architecture principles, maintaining strict separation between Port definitions (uc-core), Infrastructure implementations (uc-infra), and UseCase orchestration (uc-app). The changes preserve the existing read/write path separation while making each component's responsibility explicit through precise naming.

**Tech Stack:** Rust, async-trait, anyhow, Tauri 2, Diesel ORM, SQLite, Blake3 hashing

---

## Phase 1: Port Layer Refactor (Breaking Changes)

This phase creates the new Port interfaces and deprecates the old ones. All changes are breaking - code using these ports will fail to compile until updated.

### Task 1: Create ClipboardRepresentationNormalizerPort

**Files:**

- Create: `src-tauri/crates/uc-core/src/ports/clipboard/representation_normalizer.rs`

**Step 1: Write the failing test**

First, let's verify the file doesn't exist by trying to compile:

```rust
// In a test file, this should fail because the port doesn't exist yet
use uc_core::ports::clipboard::ClipboardRepresentationNormalizerPort;

#[test]
fn test_normalizer_port_exists() {
    // This test verifies the port trait exists
    // Will fail initially because trait doesn't exist
}
```

Run: `cargo check -p uc-core`
Expected: FAIL with "cannot find ClipboardRepresentationNormalizerPort"

**Step 2: Create the port file**

Create: `src-tauri/crates/uc-core/src/ports/clipboard/representation_normalizer.rs`

```rust
//! Clipboard Representation Normalizer Port
//!
//! This port converts platform-layer `ObservedClipboardRepresentation` to
//! domain-layer `PersistedClipboardRepresentation`.
//!
//! **Semantic:** "normalize" = type conversion / normalization from platform format to domain format

use crate::clipboard::{ObservedClipboardRepresentation, PersistedClipboardRepresentation};

#[async_trait::async_trait]
pub trait ClipboardRepresentationNormalizerPort: Send + Sync {
    /// Normalize an observed clipboard representation into a persisted representation.
    ///
    /// # Post-conditions
    /// - Returned `PersistedClipboardRepresentation` contains valid metadata (mime, size)
    /// - `inline_data` is populated by strategy:
    ///   - Small data (< threshold): full storage
    ///   - Large data: preview/placeholder
    /// - `blob_id` is initially `None` (will be set later during resolve phase)
    async fn normalize(
        &self,
        observed: &ObservedClipboardRepresentation,
    ) -> anyhow::Result<PersistedClipboardRepresentation>;
}
```

**Step 3: Export the new port**

Modify: `src-tauri/crates/uc-core/src/ports/clipboard/mod.rs`

```rust
// Add to the existing file
mod representation_normalizer;

pub use representation_normalizer::ClipboardRepresentationNormalizerPort;
```

**Step 4: Run test to verify it passes**

Run: `cargo check -p uc-core`
Expected: PASS (the port now exists)

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-core/src/ports/clipboard/representation_normalizer.rs
git add src-tauri/crates/uc-core/src/ports/clipboard/mod.rs
git commit -m "feat(core): add ClipboardRepresentationNormalizerPort

Adds new port trait for normalizing observed clipboard representations
to persisted representations. Replaces the semantic ambiguity of
'materialize' with 'normalize' for type conversion.

Part of clipboard materialization refactor."
```

---

### Task 2: Create BlobWriterPort

**Files:**

- Create: `src-tauri/crates/uc-core/src/ports/blob_writer.rs`

**Step 1: Write the failing test**

```rust
// Verify port doesn't exist
use uc_core::ports::BlobWriterPort;

#[test]
fn test_blob_writer_port_exists() {
    // Should fail initially
}
```

Run: `cargo check -p uc-core`
Expected: FAIL with "cannot find BlobWriterPort"

**Step 2: Create the port file**

Create: `src-tauri/crates/uc-core/src/ports/blob_writer.rs`

```rust
//! Blob Writer Port
//!
//! This port writes raw bytes to blob store with deduplication.
//!
//! **Semantic:** "write" = persistence with deduplication

use crate::{Blob, ContentHash};

#[async_trait::async_trait]
pub trait BlobWriterPort: Send + Sync {
    /// Write bytes to blob store with deduplication by content hash.
    ///
    /// # Idempotence guarantee
    /// - Data with identical `ContentHash` is written only once
    /// - Returns existing `Blob` or newly created one
    ///
    /// # Concurrency safety
    /// - Uses `ContentHash` as content-addressed key
    /// - `find_by_hash()` + `insert_blob()` combination provides natural deduplication
    async fn write(&self, data: &[u8], content_hash: &ContentHash) -> anyhow::Result<Blob>;
}
```

**Step 3: Export the new port**

Modify: `src-tauri/crates/uc-core/src/ports/mod.rs`

```rust
// Add near blob_materializer (which will be removed later)
mod blob_writer;

pub use blob_writer::BlobWriterPort;
```

**Step 4: Run test to verify it passes**

Run: `cargo check -p uc-core`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-core/src/ports/blob_writer.rs
git add src-tauri/crates/uc-core/src/ports/mod.rs
git commit -m "feat(core): add BlobWriterPort

Adds new port trait for writing blobs with deduplication.
Replaces 'BlobMaterializerPort' with semantically clear 'write'
terminology for persistence operations.

Part of clipboard materialization refactor."
```

---

### Task 3: Create ClipboardPayloadResolverPort

**Files:**

- Create: `src-tauri/crates/uc-core/src/ports/clipboard/payload_resolver.rs`

**Step 1: Create the enum for resolved payload**

First, let's check if `ResolvedClipboardPayload` exists:

Run: `cargo check -p uc-core 2>&1 | grep -i "ResolvedClipboardPayload"`
Expected: Not found (we need to create it)

**Step 2: Create the port file with enum**

Create: `src-tauri/crates/uc-core/src/ports/clipboard/payload_resolver.rs`

```rust
//! Clipboard Payload Resolver Port
//!
//! This port resolves persisted representations into directly usable payloads.
//!
//! **Semantic:** "resolve" = on-demand loading with lazy blob write

use crate::clipboard::PersistedClipboardRepresentation;
use crate::BlobId;

/// Result of resolving a clipboard representation into a usable payload
#[derive(Debug, Clone)]
pub enum ResolvedClipboardPayload {
    /// Inline data available (small content or preview)
    Inline { mime: String, bytes: Vec<u8> },

    /// Reference to blob storage (large content)
    BlobRef { mime: String, blob_id: BlobId },
}

#[async_trait::async_trait]
pub trait ClipboardPayloadResolverPort: Send + Sync {
    /// Resolve a persisted clipboard representation into a usable payload.
    ///
    /// # Resolution rules
    /// 1. **Prefer inline**: If `inline_data` available and complete → return `Inline`
    /// 2. **Has blob**: If `blob_id` exists → return `BlobRef`
    /// 3. **Lazy write**: Otherwise:
    ///    - Load raw bytes (from inline_data or temp storage)
    ///    - Calculate `ContentHash`
    ///    - Call `BlobWriterPort::write()` to persist
    ///    - Write back `representation.blob_id` (idempotent)
    ///    - Return `BlobRef`
    ///
    /// # Idempotence guarantee
    /// - Multiple resolves of same rep yield identical `blob_id`
    /// - Concurrent resolve: `update_blob_id` only takes effect when `None`
    async fn resolve(
        &self,
        representation: &PersistedClipboardRepresentation,
    ) -> anyhow::Result<ResolvedClipboardPayload>;
}
```

**Step 3: Export the new port and type**

Modify: `src-tauri/crates/uc-core/src/ports/clipboard/mod.rs`

```rust
// Add to exports
mod payload_resolver;

pub use payload_resolver::{ClipboardPayloadResolverPort, ResolvedClipboardPayload};
```

**Step 4: Run test to verify it compiles**

Run: `cargo check -p uc-core`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-core/src/ports/clipboard/payload_resolver.rs
git add src-tauri/crates/uc-core/src/ports/clipboard/mod.rs
git commit -m "feat(core): add ClipboardPayloadResolverPort and ResolvedClipboardPayload

Adds new port for resolving persisted clipboard representations into
usable payloads. Supports inline data, blob references, and lazy
blob writing for on-demand materialization.

Part of clipboard materialization refactor."
```

---

### Task 4: Create SelectionResolverPort

**Files:**

- Create: `src-tauri/crates/uc-core/src/ports/clipboard/selection_resolver.rs`

**Step 1: Create the port file**

Create: `src-tauri/crates/uc-core/src/ports/clipboard/selection_resolver.rs`

```rust
//! Selection Resolver Port
//!
//! This port loads the complete selection context for an entry.
//!
//! **Semantic:** "resolve" = loading related entities

use crate::clipboard::{ClipboardEntry, PersistedClipboardRepresentation};
use crate::ids::EntryId;

#[async_trait::async_trait]
pub trait SelectionResolverPort: Send + Sync {
    /// Resolve the complete selection context for an entry.
    ///
    /// # Returns
    /// - Tuple of (ClipboardEntry, PersistedClipboardRepresentation)
    ///
    /// # Loading flow
    /// 1. Load `ClipboardEntry` from `EntryRepository` (get `event_id`)
    /// 2. Load `SelectionDecision` from `SelectionRepository` (get `primary_rep_id`)
    /// 3. Load target `PersistedClipboardRepresentation` from `RepresentationRepository`
    async fn resolve_selection(
        &self,
        entry_id: &EntryId,
    ) -> anyhow::Result<(ClipboardEntry, PersistedClipboardRepresentation)>;
}
```

**Step 2: Export the new port**

Modify: `src-tauri/crates/uc-core/src/ports/clipboard/mod.rs`

```rust
// Add to exports
mod selection_resolver;

pub use selection_resolver::SelectionResolverPort;
```

**Step 3: Run test to verify it compiles**

Run: `cargo check -p uc-core`
Expected: PASS

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-core/src/ports/clipboard/selection_resolver.rs
git add src-tauri/crates/uc-core/src/ports/clipboard/mod.rs
git commit -m "feat(core): add SelectionResolverPort

Adds new port for resolving complete selection context including
entry, selection decision, and target representation.

Part of clipboard materialization refactor."
```

---

### Task 5: Add update_blob_id_if_none method to RepresentationRepositoryPort

**Files:**

- Modify: `src-tauri/crates/uc-core/src/ports/clipboard/representation_repository.rs`

**Step 1: Read the existing port**

Read: `src-tauri/crates/uc-core/src/ports/clipboard/representation_repository.rs`

**Step 2: Add the new method**

Add this method to the trait:

````rust
/// Update blob_id for a representation, but only if it's currently None.
///
/// # Returns
/// - `true` if the update was applied (blob_id was None)
/// - `false` if blob_id was already set (no-op)
///
/// # Concurrency safety
/// This should use compare-and-set semantics at the database level:
/// ```sql
/// UPDATE clipboard_snapshots_representations
/// SET blob_id = ?
/// WHERE id = ? AND blob_id IS NULL
/// ```
async fn update_blob_id_if_none(
    &self,
    rep_id: &crate::ids::RepresentationId,
    blob_id: &crate::BlobId,
) -> anyhow::Result<bool>;
````

**Step 3: Run test to verify signature compiles**

Run: `cargo check -p uc-core`
Expected: FAIL (implementation doesn't exist yet - that's expected)

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-core/src/ports/clipboard/representation_repository.rs
git commit -m "feat(core): add update_blob_id_if_none to RepresentationRepositoryPort

Adds conditional update method for safe concurrent blob_id updates.
Implements compare-and-set semantics for thread-safe lazy materialization.

Part of clipboard materialization refactor."
```

---

## Phase 2: Infrastructure Layer Adaptation

This phase adapts infrastructure implementations to use the new port interfaces.

### Task 6: Rename ClipboardRepresentationMaterializer to ClipboardRepresentationNormalizer

**Files:**

- Rename: `src-tauri/crates/uc-infra/src/clipboard/materializer.rs` → `src-tauri/crates/uc-infra/src/clipboard/normalizer.rs`

**Step 1: Copy the file to new location**

```bash
cp src-tauri/crates/uc-infra/src/clipboard/materializer.rs \
   src-tauri/crates/uc-infra/src/clipboard/normalizer.rs
```

**Step 2: Update the file contents**

Modify: `src-tauri/crates/uc-infra/src/clipboard/normalizer.rs`

Changes:

1. Update struct name: `ClipboardRepresentationMaterializer` → `ClipboardRepresentationNormalizer`
2. Update impl trait: `ClipboardRepresentationMaterializerPort` → `ClipboardRepresentationNormalizerPort`
3. Update method name: `materialize` → `normalize`
4. Update file-level documentation
5. Update log messages: "Materializing" → "Normalizing"

```rust
//! Clipboard representation normalizer with owned config
//! 带有拥有所有权的配置的剪贴板表示规范化器

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::debug;

use crate::config::clipboard_storage_config::ClipboardStorageConfig;
use uc_core::clipboard::{
    MimeType, ObservedClipboardRepresentation, PersistedClipboardRepresentation,
};
use uc_core::ports::clipboard::ClipboardRepresentationNormalizerPort;

const PREVIEW_LENGTH_CHARS: usize = 500;

/// Check if MIME type is text-based
/// 检查 MIME 类型是否为文本类型
pub(crate) fn is_text_mime_type(mime_type: &Option<MimeType>) -> bool {
    // ... (unchanged)
}

/// UTF-8 safe truncation to first N characters
/// UTF-8 安全截断到前 N 个字符
pub(crate) fn truncate_to_preview(bytes: &[u8]) -> Vec<u8> {
    // ... (unchanged)
}

/// Clipboard representation normalizer with owned config
/// 带有拥有所有权的配置的剪贴板表示规范化器
pub struct ClipboardRepresentationNormalizer {
    config: Arc<ClipboardStorageConfig>,
}

impl ClipboardRepresentationNormalizer {
    /// Create a new normalizer with the given config
    /// 使用给定配置创建新规范化器
    pub fn new(config: Arc<ClipboardStorageConfig>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ClipboardRepresentationNormalizerPort for ClipboardRepresentationNormalizer {
    async fn normalize(
        &self,
        observed: &ObservedClipboardRepresentation,
    ) -> Result<PersistedClipboardRepresentation> {
        let inline_threshold_bytes = self.config.inline_threshold_bytes;
        let size_bytes = observed.bytes.len() as i64;

        // Decision: inline or blob, with preview for large text
        // 决策：内联还是 blob，大文本生成预览
        let inline_data = if size_bytes <= inline_threshold_bytes {
            // Small content: store full data inline
            debug!(
                representation_id = %observed.id,
                format_id = %observed.format_id,
                size_bytes,
                threshold = inline_threshold_bytes,
                strategy = "inline",
                "Normalizing small content inline"
            );
            Some(observed.bytes.clone())
        } else {
            // Large content: decide based on type
            if is_text_mime_type(&observed.mime) {
                // Text type: generate truncated preview
                debug!(
                    representation_id = %observed.id,
                    format_id = %observed.format_id,
                    size_bytes,
                    threshold = inline_threshold_bytes,
                    preview_length_chars = PREVIEW_LENGTH_CHARS,
                    strategy = "preview",
                    "Normalizing large text as preview"
                );
                Some(truncate_to_preview(&observed.bytes))
            } else {
                // Non-text (images, etc.): use empty array to satisfy CHECK constraint
                debug!(
                    representation_id = %observed.id,
                    format_id = %observed.format_id,
                    size_bytes,
                    threshold = inline_threshold_bytes,
                    strategy = "placeholder",
                    "Normalizing large non-text as placeholder (blob storage pending)"
                );
                Some(vec![])
            }
        };

        Ok(PersistedClipboardRepresentation::new(
            observed.id.clone(),
            observed.format_id.clone(),
            observed.mime.clone(),
            size_bytes,
            inline_data,
            None, // blob_id will be set later by resolver
        ))
    }
}

// Tests remain unchanged
```

**Step 3: Update module exports**

Modify: `src-tauri/crates/uc-infra/src/clipboard/mod.rs`

```rust
// Add/replace the export
pub mod normalizer;

pub use normalizer::{ClipboardRepresentationNormalizer, is_text_mime_type, truncate_to_preview};
```

**Step 4: Run tests**

Run: `cargo test -p uc-infra clipboard::normalizer`
Expected: PASS (with warnings that the old file still exists)

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-infra/src/clipboard/normalizer.rs
git add src-tauri/crates/uc-infra/src/clipboard/mod.rs
git commit -m "feat(infra): add ClipboardRepresentationNormalizer

Adds new normalizer component that implements the semantic
'normalize' operation for converting observed representations
to persisted representations. Replaces 'materializer' terminology.

The old materializer.rs file will be removed in Phase 4 after
all references are migrated.

Part of clipboard materialization refactor."
```

---

### Task 7: Rename BlobMaterializer to BlobWriter

**Files:**

- Rename: `src-tauri/crates/uc-infra/src/blob/blob_materializer.rs` → `src-tauri/crates/uc-infra/src/blob/blob_writer.rs`

**Step 1: Copy the file to new location**

```bash
cp src-tauri/crates/uc-infra/src/blob/blob_materializer.rs \
   src-tauri/crates/uc-infra/src/blob/blob_writer.rs
```

**Step 2: Update the file contents**

Modify: `src-tauri/crates/uc-infra/src/blob/blob_writer.rs`

Changes:

1. Update struct name: `BlobMaterializer` → `BlobWriter`
2. Update impl trait: `BlobMaterializerPort` → `BlobWriterPort`
3. Update method name: `materialize` → `write`
4. Update log messages

```rust
use anyhow::{Ok, Result};
use async_trait::async_trait;
use tracing::{debug_span, Instrument};
use uc_core::blob::BlobStorageLocator;
use uc_core::ports::ClockPort;
use uc_core::ports::{BlobRepositoryPort, BlobStorePort, BlobWriterPort};
use uc_core::ContentHash;
use uc_core::{Blob, BlobId};

pub struct BlobWriter<B, BR, C>
where
    B: BlobStorePort,
    BR: BlobRepositoryPort,
    C: ClockPort,
{
    blob_store: B,
    blob_repo: BR,
    clock: C,
}

impl<B, BR, C> BlobWriter<B, BR, C>
where
    B: BlobStorePort,
    BR: BlobRepositoryPort,
    C: ClockPort,
{
    pub fn new(blob_store: B, blob_repo: BR, clock: C) -> Self {
        BlobWriter {
            blob_store,
            blob_repo,
            clock,
        }
    }
}

#[async_trait]
impl<B, BR, C> BlobWriterPort for BlobWriter<B, BR, C>
where
    B: BlobStorePort,
    BR: BlobRepositoryPort,
    C: ClockPort,
{
    async fn write(&self, data: &[u8], content_hash: &ContentHash) -> Result<Blob> {
        let span = debug_span!(
            "infra.blob.write",
            size_bytes = data.len(),
            content_hash = %content_hash,
        );
        async {
            if let Some(blob) = self.blob_repo.find_by_hash(content_hash).await? {
                return Ok(blob);
            }

            let blob_id = BlobId::new();

            // TODO: Implement encryption for blob data
            let storage_path = self.blob_store.put(&blob_id, data).await?;

            let created_at_ms = self.clock.now_ms();
            let blob_storage_locator = BlobStorageLocator::new_local_fs(storage_path);
            let result = Blob::new(
                blob_id,
                blob_storage_locator,
                data.len() as i64,
                content_hash.clone(),
                created_at_ms,
            );

            self.blob_repo.insert_blob(&result).await?;
            Ok(result)
        }
        .instrument(span)
        .await
    }
}
```

**Step 3: Update module exports**

Modify: `src-tauri/crates/uc-infra/src/blob/mod.rs`

```rust
pub mod blob_writer;

pub use blob_writer::BlobWriter;
```

**Step 4: Run tests**

Run: `cargo test -p uc-infra blob`
Expected: PASS (warnings about old file)

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-infra/src/blob/blob_writer.rs
git add src-tauri/crates/uc-infra/src/blob/mod.rs
git commit -m "feat(infra): add BlobWriter

Adds new blob writer component that implements the semantic
'write' operation for persisting blobs with deduplication.
Replaces 'materializer' terminology with clearer intent.

The old blob_materializer.rs file will be removed in Phase 4
after all references are migrated.

Part of clipboard materialization refactor."
```

---

### Task 8: Implement ClipboardPayloadResolver

**Files:**

- Create: `src-tauri/crates/uc-infra/src/clipboard/payload_resolver.rs`

**Step 1: Create the implementation file**

Create: `src-tauri/crates/uc-infra/src/clipboard/payload_resolver.rs`

```rust
//! Clipboard Payload Resolver Implementation
//!
//! Resolves persisted clipboard representations into usable payloads.
//! Supports inline data, blob references, and lazy blob writing.

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info_span, Instrument};

use uc_core::clipboard::PersistedClipboardRepresentation;
use uc_core::ports::{
    BlobRepositoryPort, BlobStorePort, BlobWriterPort, ClipboardPayloadResolverPort,
    ClipboardRepresentationRepositoryPort, ContentHashPort,
};
use uc_core::ports::clipboard::ResolvedClipboardPayload;

/// Clipboard payload resolver implementation
pub struct ClipboardPayloadResolver<R, B, BR, H>
where
    R: ClipboardRepresentationRepositoryPort,
    B: BlobWriterPort,
    BR: BlobRepositoryPort,
    H: ContentHashPort,
{
    representation_repo: R,
    blob_writer: B,
    blob_repo: BR,
    hasher: H,
}

impl<R, B, BR, H> ClipboardPayloadResolver<R, B, BR, H>
where
    R: ClipboardRepresentationRepositoryPort,
    B: BlobWriterPort,
    BR: BlobRepositoryPort,
    H: ContentHashPort,
{
    pub fn new(
        representation_repo: R,
        blob_writer: B,
        blob_repo: BR,
        hasher: H,
    ) -> Self {
        Self {
            representation_repo,
            blob_writer,
            blob_repo,
            hasher,
        }
    }
}

#[async_trait]
impl<R, B, BR, H> ClipboardPayloadResolverPort for ClipboardPayloadResolver<R, B, BR, H>
where
    R: ClipboardRepresentationRepositoryPort,
    B: BlobWriterPort,
    BR: BlobRepositoryPort,
    H: ContentHashPort,
{
    async fn resolve(
        &self,
        representation: &PersistedClipboardRepresentation,
    ) -> Result<ResolvedClipboardPayload> {
        let span = info_span!(
            "infra.payload.resolve",
            representation_id = %representation.id,
            format_id = %representation.format_id,
        );
        async move {
            // Rule 1: Prefer inline data
            if let Some(inline_data) = &representation.inline_data {
                // Check if it's a placeholder (empty) or actual data
                if !inline_data.is_empty() {
                    debug!("Resolving from inline data");
                    return Ok(ResolvedClipboardPayload::Inline {
                        mime: representation.mime_type.clone().unwrap_or_default(),
                        bytes: inline_data.clone(),
                    });
                }
                // Empty inline_data means placeholder - continue to blob logic
            }

            // Rule 2: Has blob_id
            if let Some(blob_id) = &representation.blob_id {
                debug!("Resolving from existing blob reference");
                return Ok(ResolvedClipboardPayload::BlobRef {
                    mime: representation.mime_type.clone().unwrap_or_default(),
                    blob_id: blob_id.clone(),
                });
            }

            // Rule 3: Lazy write - load bytes and persist to blob
            debug!("Lazy writing blob for representation");

            // Load raw bytes (from inline placeholder or other source)
            let raw_bytes = self.load_raw_bytes(representation).await?;

            // Calculate content hash
            let content_hash = self.hasher.hash_bytes(&raw_bytes)?;

            // Write to blob store (deduplicated)
            let blob = self.blob_writer.write(&raw_bytes, &content_hash).await?;

            // Update representation.blob_id (idempotent)
            let updated = self
                .representation_repo
                .update_blob_id_if_none(&representation.id, &blob.blob_id)
                .await?;

            if updated {
                debug!("Updated representation with new blob_id");
            } else {
                debug!("Representation already had blob_id (concurrent update)");
            }

            Ok(ResolvedClipboardPayload::BlobRef {
                mime: representation.mime_type.clone().unwrap_or_default(),
                blob_id: blob.blob_id,
            })
        }
        .instrument(span)
        .await
    }
}

impl<R, B, BR, H> ClipboardPayloadResolver<R, B, BR, H>
where
    R: ClipboardRepresentationRepositoryPort,
    B: BlobWriterPort,
    BR: BlobRepositoryPort,
    H: ContentHashPort,
{
    /// Load raw bytes for a representation.
    ///
    /// This is a helper for the lazy write case when we need to materialize
    /// the original data before writing to blob storage.
    async fn load_raw_bytes(
        &self,
        representation: &PersistedClipboardRepresentation,
    ) -> Result<Vec<u8>> {
        // For placeholder representations (empty inline_data, no blob_id),
        // we need to reload from the original source.
        //
        // TODO: This requires access to the original snapshot data.
        // For now, return an error as this needs to be implemented
        // with proper temp storage or snapshot caching.
        Err(anyhow::anyhow!(
            "Lazy blob writing not yet implemented for placeholders. \
             Representation {} has no retrievable data.",
            representation.id
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Add tests for inline data, blob reference, and error cases
}
```

**Step 2: Export the new implementation**

Modify: `src-tauri/crates/uc-infra/src/clipboard/mod.rs`

```rust
pub mod payload_resolver;

pub use payload_resolver::ClipboardPayloadResolver;
```

**Step 3: Run tests**

Run: `cargo check -p uc-infra`
Expected: PASS (with TODO noted)

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-infra/src/clipboard/payload_resolver.rs
git add src-tauri/crates/uc-infra/src/clipboard/mod.rs
git commit -m "feat(infra): add ClipboardPayloadResolver implementation

Implements the payload resolver port with support for:
- Inline data resolution
- Existing blob reference resolution
- Lazy blob writing (TODO: placeholder handling)

Part of clipboard materialization refactor."
```

---

### Task 9: Implement SelectionResolver

**Files:**

- Create: `src-tauri/crates/uc-infra/src/clipboard/selection_resolver.rs`

**Step 1: Create the implementation file**

Create: `src-tauri/crates/uc-infra/src/clipboard/selection_resolver.rs`

```rust
//! Selection Resolver Implementation
//!
//! Loads complete selection context for an entry.

use anyhow::Result;
use async_trait::async_trait;
use uc_core::clipboard::{ClipboardEntry, PersistedClipboardRepresentation};
use uc_core::ids::{EntryId, EventId, RepresentationId};
use uc_core::ports::{
    ClipboardEntryRepositoryPort, ClipboardRepresentationRepositoryPort,
    ClipboardSelectionRepositoryPort, SelectionResolverPort,
};

/// Selection resolver implementation
pub struct SelectionResolver<E, S, R>
where
    E: ClipboardEntryRepositoryPort,
    S: ClipboardSelectionRepositoryPort,
    R: ClipboardRepresentationRepositoryPort,
{
    entry_repo: E,
    selection_repo: S,
    representation_repo: R,
}

impl<E, S, R> SelectionResolver<E, S, R>
where
    E: ClipboardEntryRepositoryPort,
    S: ClipboardSelectionRepositoryPort,
    R: ClipboardRepresentationRepositoryPort,
{
    pub fn new(entry_repo: E, selection_repo: S, representation_repo: R) -> Self {
        Self {
            entry_repo,
            selection_repo,
            representation_repo,
        }
    }
}

#[async_trait]
impl<E, S, R> SelectionResolverPort for SelectionResolver<E, S, R>
where
    E: ClipboardEntryRepositoryPort,
    S: ClipboardSelectionRepositoryPort,
    R: ClipboardRepresentationRepositoryPort,
{
    async fn resolve_selection(
        &self,
        entry_id: &EntryId,
    ) -> Result<(ClipboardEntry, PersistedClipboardRepresentation)> {
        // 1. Load ClipboardEntry
        let entry = self
            .entry_repo
            .get_entry(entry_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Entry {} not found", entry_id))?;

        // 2. Load SelectionDecision
        let selection_decision = self
            .selection_repo
            .get_selection(entry_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Selection for entry {} not found", entry_id))?;

        // 3. Load target PersistedClipboardRepresentation
        let primary_rep_id = selection_decision.selection.primary_rep_id;
        let representation = self
            .representation_repo
            .get_representation(&entry.event_id, &primary_rep_id)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Representation {} not found for event {}",
                    primary_rep_id,
                    entry.event_id
                )
            })?;

        Ok((entry, representation))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Add unit tests with mock repositories
}
```

**Step 2: Export the new implementation**

Modify: `src-tauri/crates/uc-infra/src/clipboard/mod.rs`

```rust
pub mod selection_resolver;

pub use selection_resolver::SelectionResolver;
```

**Step 3: Run tests**

Run: `cargo check -p uc-infra`
Expected: PASS

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-infra/src/clipboard/selection_resolver.rs
git add src-tauri/crates/uc-infra/src/clipboard/mod.rs
git commit -m "feat(infra): add SelectionResolver implementation

Implements the selection resolver port for loading complete
selection context including entry, selection decision, and
target representation.

Part of clipboard materialization refactor."
```

---

### Task 10: Implement update_blob_id_if_none in Repository

**Files:**

- Modify: `src-tauri/crates/uc-infra/src/db/repositories/diesel_clipboard_representation_repository.rs`

**Step 1: Read the existing repository**

Read: `src-tauri/crates/uc-infra/src/db/repositories/diesel_clipboard_representation_repository.rs`

**Step 2: Add the new method**

Add to the implementation:

```rust
async fn update_blob_id_if_none(
    &self,
    rep_id: &RepresentationId,
    blob_id: &BlobId,
) -> Result<bool> {
    use diesel::prelude::*;
    use uc_infra::db::schema::clipboard_snapshots_representations::dsl::*;

    let rep_id_str = rep_id.to_string();
    let blob_id_str = blob_id.to_string();

    let updated_rows = self
        .executor
        .execute(move |conn| {
            diesel::update(
                clipboard_snapshots_representations
                    .filter(id.eq(rep_id_str).and(blob_id.is_null())),
            )
            .set(blob_id.eq(blob_id_str))
            .execute(conn)
        })
        .await?;

    Ok(updated_rows > 0)
}
```

**Step 3: Run tests**

Run: `cargo test -p uc-infra diesel_clipboard_representation_repository`
Expected: PASS

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-infra/src/db/repositories/diesel_clipboard_representation_repository.rs
git commit -m "feat(infra): implement update_blob_id_if_none in representation repository

Implements compare-and-set update for thread-safe blob_id assignment.
Uses SQL WHERE clause to ensure only NULL blob_id values are updated.

Part of clipboard materialization refactor."
```

---

## Phase 3: UseCase Layer Migration

This phase updates use cases to use the new ports.

### Task 11: Update CaptureClipboardUseCase

**Files:**

- Modify: `src-tauri/crates/uc-app/src/usecases/internal/capture_clipboard.rs`

**Step 1: Update imports**

```rust
// Change
use uc_core::ports::ClipboardRepresentationMaterializerPort;
// To
use uc_core::ports::clipboard::ClipboardRepresentationNormalizerPort;
```

**Step 2: Update struct field**

```rust
pub struct CaptureClipboardUseCase {
    platform_clipboard_port: Arc<dyn PlatformClipboardPort>,
    entry_repo: Arc<dyn ClipboardEntryRepositoryPort>,
    event_writer: Arc<dyn ClipboardEventWriterPort>,
    representation_policy: Arc<dyn SelectRepresentationPolicyPort>,
    representation_normalizer: Arc<dyn ClipboardRepresentationNormalizerPort>, // Changed
    device_identity: Arc<dyn DeviceIdentityPort>,
}
```

**Step 3: Update constructor**

```rust
pub fn new(
    platform_clipboard_port: Arc<dyn PlatformClipboardPort>,
    entry_repo: Arc<dyn ClipboardEntryRepositoryPort>,
    event_writer: Arc<dyn ClipboardEventWriterPort>,
    representation_policy: Arc<dyn SelectRepresentationPolicyPort>,
    representation_normalizer: Arc<dyn ClipboardRepresentationNormalizerPort>, // Changed
    device_identity: Arc<dyn DeviceIdentityPort>,
) -> Self {
    Self {
        platform_clipboard_port,
        entry_repo,
        event_writer,
        representation_policy,
        representation_normalizer, // Changed
        device_identity,
    }
}
```

**Step 4: Update method calls**

In both `execute()` and `execute_with_snapshot()`:

```rust
// Change from
let materialized_futures: Vec<_> = snapshot
    .representations
    .iter()
    .map(|rep| self.representation_materializer.materialize(rep))
    .collect();
let materialized_reps = try_join_all(materialized_futures).await?;

// To
let normalized_futures: Vec<_> = snapshot
    .representations
    .iter()
    .map(|rep| self.representation_normalizer.normalize(rep))
    .collect();
let normalized_reps = try_join_all(normalized_futures).await?;
```

**Step 5: Update doc comments**

Change all references from "materialize" to "normalize".

**Step 6: Run tests**

Run: `cargo check -p uc-app`
Expected: FAIL (dependencies not updated yet)

**Step 7: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/internal/capture_clipboard.rs
git commit -m "refactor(app): update CaptureClipboardUseCase to use normalizer

Renames representation_materializer to representation_normalizer
and method from materialize to normalize. Updates all documentation
to reflect semantic change.

Part of clipboard materialization refactor."
```

---

### Task 12: Create ResolveClipboardSelectionPayloadUseCase

**Files:**

- Create: `src-tauri/crates/uc-app/src/usecases/internal/resolve_clipboard_selection.rs`

**Step 1: Create the new use case**

Create: `src-tauri/crates/uc-app/src/usecases/internal/resolve_clipboard_selection.rs`

```rust
use std::sync::Arc;
use anyhow::Result;
use tracing::{info, info_span, Instrument};

use uc_core::ids::EntryId;
use uc_core::ports::clipboard::{
    ClipboardPayloadResolverPort, ResolvedClipboardPayload, SelectionResolverPort,
};

/// Resolve clipboard selection payload use case.
///
/// This use case orchestrates the resolution of a clipboard entry's
/// selected representation into a usable payload (inline or blob reference).
///
/// # Behavior
/// 1. Resolve the complete selection context (entry + representation)
/// 2. Resolve the representation into a payload (inline or blob)
/// 3. Return the payload for use by the UI or paste operations
pub struct ResolveClipboardSelectionPayloadUseCase {
    selection_resolver: Arc<dyn SelectionResolverPort>,
    payload_resolver: Arc<dyn ClipboardPayloadResolverPort>,
}

impl ResolveClipboardSelectionPayloadUseCase {
    pub fn new(
        selection_resolver: Arc<dyn SelectionResolverPort>,
        payload_resolver: Arc<dyn ClipboardPayloadResolverPort>,
    ) -> Self {
        Self {
            selection_resolver,
            payload_resolver,
        }
    }

    /// Execute the resolution workflow.
    ///
    /// # Returns
    /// - `ResolvedClipboardPayload` containing either inline data or blob reference
    pub async fn execute(&self, entry_id: &EntryId) -> Result<ResolvedClipboardPayload> {
        let span = info_span!(
            "usecase.resolve_clipboard_selection.execute",
            entry_id = %entry_id,
        );
        async move {
            info!("Resolving clipboard selection for entry {}", entry_id);

            // 1. Resolve selection context
            let (_entry, rep) = self.selection_resolver.resolve_selection(entry_id).await?;

            // 2. Resolve payload
            let payload = self.payload_resolver.resolve(&rep).await?;

            info!(entry_id = %entry_id, "Clipboard selection resolved");
            Ok(payload)
        }
        .instrument(span)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Add unit tests with mock ports
}
```

**Step 2: Export the new use case**

Modify: `src-tauri/crates/uc-app/src/usecases/mod.rs`

```rust
pub mod internal;
```

**Step 3: Run tests**

Run: `cargo check -p uc-app`
Expected: PASS

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/internal/resolve_clipboard_selection.rs
git add src-tauri/crates/uc-app/src/usecases/mod.rs
git commit -m "feat(app): add ResolveClipboardSelectionPayloadUseCase

Creates new use case for resolving clipboard selections into
usable payloads. Replaces MaterializeClipboardSelectionUseCase
with clearer semantic intent.

Part of clipboard materialization refactor."
```

---

## Phase 4: Dependency Injection Updates

This phase updates all dependency injection to use the new components.

### Task 13: Update AppDeps struct

**Files:**

- Modify: `src-tauri/crates/uc-app/src/deps.rs`

**Step 1: Update the struct fields**

```rust
pub struct AppDeps {
    // Clipboard dependencies
    pub clipboard: Arc<dyn PlatformClipboardPort>,
    pub clipboard_entry_repo: Arc<dyn ClipboardEntryRepositoryPort>,
    pub clipboard_event_repo: Arc<dyn ClipboardEventWriterPort>,
    pub representation_repo: Arc<dyn ClipboardRepresentationRepositoryPort>,
    pub representation_normalizer: Arc<dyn ClipboardRepresentationNormalizerPort>, // Changed
    pub selection_repo: Arc<dyn ClipboardSelectionRepositoryPort>,
    pub representation_policy: Arc<dyn SelectRepresentationPolicyPort>,

    // ... other fields unchanged ...

    // Storage dependencies
    pub blob_store: Arc<dyn BlobStorePort>,
    pub blob_repository: Arc<dyn BlobRepositoryPort>,
    pub blob_writer: Arc<dyn BlobWriterPort>, // Changed

    // ... rest unchanged ...
}
```

**Step 2: Run tests**

Run: `cargo check -p uc-app`
Expected: FAIL (implementation not updated)

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-app/src/deps.rs
git commit -m "refactor(app): update AppDeps field names

Updates dependency fields to match new naming:
- representation_materializer → representation_normalizer
- blob_materializer → blob_writer

Part of clipboard materialization refactor."
```

---

### Task 14: Update wiring.rs

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`

**Step 1: Update imports**

```rust
use uc_infra::clipboard::ClipboardRepresentationNormalizer; // Changed
use uc_infra::blob::BlobWriter; // Changed
use uc_core::ports::clipboard::{
    ClipboardRepresentationNormalizerPort, // Changed
    ClipboardPayloadResolverPort, // New
    SelectionResolverPort, // New
    ResolvedClipboardPayload, // New
};
use uc_core::ports::BlobWriterPort; // Changed
```

**Step 2: Update PlatformLayer struct**

```rust
struct PlatformLayer {
    // ... other fields ...
    representation_normalizer: Arc<dyn ClipboardRepresentationNormalizerPort>, // Changed
    blob_writer: Arc<dyn BlobWriterPort>, // Changed
    // ... other fields ...
}
```

**Step 3: Update create_platform_layer function**

```rust
// Change
let representation_materializer: Arc<dyn ClipboardRepresentationMaterializerPort> =
    Arc::new(ClipboardRepresentationMaterializer::new(storage_config));
// To
let representation_normalizer: Arc<dyn ClipboardRepresentationNormalizerPort> =
    Arc::new(ClipboardRepresentationNormalizer::new(storage_config));
```

```rust
// Change
let blob_materializer: Arc<dyn BlobMaterializerPort> =
    Arc::new(PlaceholderBlobMaterializerPort);
// To
let blob_writer: Arc<dyn BlobWriterPort> = Arc::new(PlaceholderBlobWriterPort);
```

**Step 4: Update PlatformLayer construction**

```rust
Ok(PlatformLayer {
    // ...
    representation_normalizer, // Changed
    blob_writer, // Changed
    // ...
})
```

**Step 5: Update AppDeps construction**

```rust
let deps = AppDeps {
    // ...
    representation_normalizer: platform.representation_normalizer, // Changed
    // ...
    blob_writer: platform.blob_writer, // Changed
    // ...
};
```

**Step 6: Run tests**

Run: `cargo test -p uc-tauri bootstrap`
Expected: FAIL (placeholder port doesn't exist yet)

**Step 7: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs
git commit -m "refactor(tauri): update wiring to use new ports

Updates dependency injection to use:
- ClipboardRepresentationNormalizer
- BlobWriter
- SelectionResolverPort
- ClipboardPayloadResolverPort

Part of clipboard materialization refactor."
```

---

### Task 15: Create PlaceholderBlobWriterPort

**Files:**

- Modify: `src-tauri/crates/uc-platform/src/adapters/mod.rs` or similar placeholder location

**Step 1: Add the placeholder implementation**

```rust
use uc_core::{Blob, ContentHash};
use uc_core::ports::BlobWriterPort;

pub struct PlaceholderBlobWriterPort;

#[async_trait::async_trait]
impl BlobWriterPort for PlaceholderBlobWriterPort {
    async fn write(&self, _data: &[u8], _content_hash: &ContentHash) -> anyhow::Result<Blob> {
        Err(anyhow::anyhow!("BlobWriter not implemented"))
    }
}
```

**Step 2: Export the placeholder**

**Step 3: Run tests**

Run: `cargo check -p uc-tauri`
Expected: PASS

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-platform/src/adapters/mod.rs
git commit -m "feat(platform): add PlaceholderBlobWriterPort

Adds placeholder implementation for BlobWriterPort for
use during development and testing.

Part of clipboard materialization refactor."
```

---

### Task 16: Update UseCases accessor

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`

**Step 1: Add the new use case accessor**

```rust
pub fn resolve_clipboard_selection(&self) -> ResolveClipboardSelectionPayloadUseCase {
    ResolveClipboardSelectionPayloadUseCase::new(
        self.deps.selection_resolver.clone(),
        self.deps.payload_resolver.clone(),
    )
}
```

**Step 2: Run tests**

Run: `cargo check -p uc-tauri`
Expected: PASS

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs
git commit -m "feat(tauri): add resolve_clipboard_selection accessor

Adds accessor for the new ResolveClipboardSelectionPayloadUseCase
in the UseCases grouping.

Part of clipboard materialization refactor."
```

---

## Phase 5: Cleanup and Validation

This phase removes deprecated code and validates everything works.

### Task 17: Remove old Port files

**Files:**

- Delete: `src-tauri/crates/uc-core/src/ports/clipboard/representation_materializer.rs`
- Delete: `src-tauri/crates/uc-core/src/ports/blob_materializer.rs`

**Step 1: Verify no references exist**

```bash
grep -r "ClipboardRepresentationMaterializerPort" src-tauri/crates/ --exclude-dir=target
grep -r "BlobMaterializerPort" src-tauri/crates/ --exclude-dir=target
```

Expected: No results (only in old files)

**Step 2: Delete the files**

```bash
rm src-tauri/crates/uc-core/src/ports/clipboard/representation_materializer.rs
rm src-tauri/crates/uc-core/src/ports/blob_materializer.rs
```

**Step 3: Update mod.rs exports**

Remove exports for deleted ports.

**Step 4: Run tests**

Run: `cargo check -p uc-core`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-core/src/ports/
git commit -m "refactor(core): remove deprecated materializer ports

Removes ClipboardRepresentationMaterializerPort and
BlobMaterializerPort, replaced by Normalizer and Writer
variants respectively.

Part of clipboard materialization refactor."
```

---

### Task 18: Remove old Infrastructure files

**Files:**

- Delete: `src-tauri/crates/uc-infra/src/clipboard/materializer.rs`
- Delete: `src-tauri/crates/uc-infra/src/blob/blob_materializer.rs`

**Step 1: Verify no references exist**

```bash
grep -r "ClipboardRepresentationMaterializer" src-tauri/crates/ --exclude-dir=target
grep -r "BlobMaterializer" src-tauri/crates/ --exclude-dir=target
```

Expected: No results

**Step 2: Delete the files**

```bash
rm src-tauri/crates/uc-infra/src/clipboard/materializer.rs
rm src-tauri/crates/uc-infra/src/blob/blob_materializer.rs
```

**Step 3: Update module exports**

Remove old exports.

**Step 4: Run tests**

Run: `cargo test -p uc-infra`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-infra/
git commit -m "refactor(infra): remove old materializer implementations

Removes ClipboardRepresentationMaterializer and BlobMaterializer,
replaced by Normalizer and Writer variants.

Part of clipboard materialization refactor."
```

---

### Task 19: Update Tauri Commands

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/clipboard.rs` or wherever clipboard commands are defined

**Step 1: Find commands using the old use case**

```bash
grep -r "MaterializeClipboardSelectionUseCase" src-tauri/crates/uc-tauri/src/commands/
```

**Step 2: Update to use new use case**

```rust
// Old code
let uc = runtime.usecases().materialize_clipboard_selection();
let payload = uc.execute(&entry_id).await?;

// New code
let uc = runtime.usecases().resolve_clipboard_selection();
let payload = uc.execute(&entry_id).await?;
```

**Step 3: Update response handling if needed**

The `ResolvedClipboardPayload` enum matches the old `MaterializedPayload` structure, so this should be transparent.

**Step 4: Run tests**

Run: `cargo check -p uc-tauri`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/
git commit -m "refactor(tauri): update clipboard commands to use new use case

Updates clipboard-related Tauri commands to use
ResolveClipboardSelectionPayloadUseCase instead of
MaterializeClipboardSelectionUseCase.

Part of clipboard materialization refactor."
```

---

### Task 20: Remove old UseCase file

**Files:**

- Delete: `src-tauri/crates/uc-app/src/usecases/internal/materialize_clipboard_selection.rs`

**Step 1: Verify no references exist**

```bash
grep -r "MaterializeClipboardSelectionUseCase" src-tauri/crates/ --exclude-dir=target
```

Expected: No results

**Step 2: Delete the file**

```bash
rm src-tauri/crates/uc-app/src/usecases/internal/materialize_clipboard_selection.rs
```

**Step 3: Run tests**

Run: `cargo check -p uc-app`
Expected: PASS

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/
git commit -m "refactor(app): remove MaterializeClipboardSelectionUseCase

Removes the old use case, replaced by
ResolveClipboardSelectionPayloadUseCase.

Part of clipboard materialization refactor."
```

---

### Task 21: Final Integration Tests

**Step 1: Run full test suite**

```bash
cd src-tauri && cargo test --workspace
```

Expected: All tests PASS

**Step 2: Build application**

```bash
cd src-tauri && cargo build
```

Expected: Successful build

**Step 3: Run application smoke test**

```bash
bun run tauri dev
```

Manual verification:

- Clipboard capture works
- Viewing clipboard entries works
- Copy/paste operations work

**Step 4: Commit final cleanup**

```bash
git commit --allow-empty -m "test: verify clipboard materialization refactor

All tests passing, application builds and runs successfully.
Clipboard capture and resolution working as expected."
```

---

### Task 22: Update Documentation

**Files:**

- Modify: `docs/architecture/clipboard-materialization-refactor-design.md`
- Modify: Any other relevant documentation

**Step 1: Update design document status**

Change: `Status: Design Draft` → `Status: Implemented`

**Step 2: Add implementation notes**

Add section:

```markdown
## Implementation Notes

This refactor was completed in 2025-01-18 with the following changes:

- Total commits: 22
- Files modified: ~15
- New ports created: 4 (Normalizer, Writer, PayloadResolver, SelectionResolver)
- Deprecated ports removed: 2 (RepresentationMaterializer, BlobMaterializer)
```

**Step 3: Run docs build if applicable**

**Step 4: Commit**

```bash
git add docs/
git commit -m "docs: update clipboard materialization refactor status

Marks the refactor as complete and adds implementation notes."
```

---

## Summary

This refactoring achieves semantic clarity by:

1. **`normalize`** (was `materialize` in capture): Type conversion from platform to domain
2. **`resolve`** (was `materialize` in selection): On-demand loading with lazy blob write
3. **`write`** (was `materialize` for blob): Persistence with deduplication

The refactor maintains backward compatibility in behavior while making code intent explicit through naming.

**Total estimated tasks:** 22
**Estimated time:** 4-6 hours for an engineer familiar with the codebase
**Risk level:** Medium (breaking changes confined to internal architecture)
