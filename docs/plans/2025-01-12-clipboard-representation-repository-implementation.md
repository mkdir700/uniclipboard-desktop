# ClipboardRepresentationRepositoryPort Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement `ClipboardRepresentationRepositoryPort` for querying and updating clipboard snapshot representations in SQLite via Diesel.

**Architecture:** Hexagonal Architecture (Ports & Adapters). The repository implements a port defined in `uc-core` and uses Diesel ORM with mapper pattern for domain/row conversion.

**Tech Stack:** Rust, Diesel ORM, SQLite, async-trait, anyhow

---

## Task 1: Update Port Return Type

**Files:**

- Modify: `src-tauri/crates/uc-core/src/ports/clipboard/representation_repository.rs:9-13`

**Step 1: Update get_signature return type**

Change from `Result<PersistedClipboardRepresentation>` to `Result<Option<PersistedClipboardRepresentation>>` to align with existing query patterns like `BlobRepositoryPort::find_by_hash`.

```rust
async fn get_representation(
    &self,
    event_id: &EventId,
    representation_id: &RepresentationId,
) -> Result<Option<PersistedClipboardRepresentation>>;
```

**Step 2: Run cargo check**

Run: `cargo check -p uc-core`
Expected: Success (type change is backward compatible for callers using `?`)

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-core/src/ports/clipboard/representation_repository.rs
git commit -m "refactor(uc-core): change get_representation return type to Option

Aligns with BlobRepositoryPort::find_by_hash pattern for query operations.
Allows graceful handling of missing records."
```

---

## Task 2: Add RowMapper to Snapshot Representation Mapper

**Files:**

- Modify: `src-tauri/crates/uc-infra/src/db/mappers/snapshot_representation_mapper.rs`

**Step 1: Add RowMapper import**

Add `RowMapper` to the imports (line 2):

```rust
use crate::db::ports::{InsertMapper, RowMapper};
```

**Step 2: Add SnapshotRepresentationRow import**

Add to imports (line 1):

```rust
use crate::db::models::snapshot_representation::{NewSnapshotRepresentationRow, SnapshotRepresentationRow};
```

**Step 3: Implement RowMapper trait**

Add after the existing `InsertMapper` impl (after line 26):

```rust
impl RowMapper<SnapshotRepresentationRow, uc_core::clipboard::PersistedClipboardRepresentation>
    for RepresentationRowMapper
{
    fn to_domain(
        &self,
        row: &SnapshotRepresentationRow,
    ) -> Result<uc_core::clipboard::PersistedClipboardRepresentation> {
        use uc_core::{ids::RepresentationId, BlobId, MimeType};

        Ok(uc_core::clipboard::PersistedClipboardRepresentation::new(
            RepresentationId::from(row.id.clone()),
            uc_core::FormatId::from(row.format_id.clone()),
            row.mime_type.as_ref().map(|s| MimeType(s.clone())),
            row.size_bytes,
            row.inline_data.clone(),
            row.blob_id.as_ref().map(|s| BlobId::from(s.clone())),
        ))
    }
}
```

**Step 4: Run cargo check**

Run: `cargo check -p uc-infra`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-infra/src/db/mappers/snapshot_representation_mapper.rs
git commit -m "feat(uc-infra): add RowMapper for SnapshotRepresentationRow

Enables conversion from database rows to domain models for queries."
```

---

## Task 3: Create Representation Repository

**Files:**

- Create: `src-tauri/crates/uc-infra/src/db/repositories/representation_repo.rs`

**Step 1: Create repository file**

Create the file with this content:

```rust
use crate::db::models::snapshot_representation::SnapshotRepresentationRow;
use crate::db::mappers::snapshot_representation_mapper::RepresentationRowMapper;
use crate::db::ports::{DbExecutor, RowMapper};
use crate::db::schema::clipboard_snapshot_representation;
use anyhow::Result;
use diesel::{OptionalExtension, QueryDsl, RunQueryDsl};
use uc_core::clipboard::PersistedClipboardRepresentation;
use uc_core::ids::{EventId, RepresentationId};
use uc_core::ports::clipboard::ClipboardRepresentationRepositoryPort;
use uc_core::BlobId;

pub struct DieselClipboardRepresentationRepository<E>
where
    E: DbExecutor,
{
    executor: E,
}

impl<E> DieselClipboardRepresentationRepository<E>
where
    E: DbExecutor,
{
    pub fn new(executor: E) -> Self {
        Self { executor }
    }
}

#[async_trait::async_trait]
impl<E> ClipboardRepresentationRepositoryPort for DieselClipboardRepresentationRepository<E>
where
    E: DbExecutor,
{
    async fn get_representation(
        &self,
        event_id: &EventId,
        representation_id: &RepresentationId,
    ) -> Result<Option<PersistedClipboardRepresentation>> {
        let event_id_str = event_id.to_string();
        let rep_id_str = representation_id.to_string();

        let row: Option<SnapshotRepresentationRow> = self.executor.run(|conn| {
            let result: Result<Option<SnapshotRepresentationRow>, diesel::result::Error> =
                clipboard_snapshot_representation::table
                    .filter(
                        clipboard_snapshot_representation::event_id
                            .eq(&event_id_str)
                            .and(clipboard_snapshot_representation::id.eq(&rep_id_str)),
                    )
                    .first::<SnapshotRepresentationRow>(conn)
                    .optional();
            result.map_err(|e| anyhow::anyhow!("Database error: {}", e))
        })?;

        match row {
            Some(r) => {
                let mapper = RepresentationRowMapper;
                let rep = mapper.to_domain(&r)?;
                Ok(Some(rep))
            }
            None => Ok(None),
        }
    }

    async fn update_blob_id(
        &self,
        representation_id: &RepresentationId,
        blob_id: &BlobId,
    ) -> Result<()> {
        let rep_id_str = representation_id.to_string();
        let blob_id_str = blob_id.to_string();

        self.executor.run(|conn| {
            diesel::update(
                clipboard_snapshot_representation::table
                    .filter(clipboard_snapshot_representation::id.eq(&rep_id_str)),
            )
            .set(clipboard_snapshot_representation::blob_id.eq(&blob_id_str))
            .execute(conn)?;
            Ok(())
        })
    }
}
```

**Step 2: Run cargo check**

Run: `cargo check -p uc-infra`
Expected: PASS

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-infra/src/db/repositories/representation_repo.rs
git commit -m "feat(uc-infra): implement ClipboardRepresentationRepositoryPort

- Add DieselClipboardRepresentationRepository with get_representation method
- Add update_blob_id method for lazy materialization support
- Uses RepresentationRowMapper for domain conversion"
```

---

## Task 4: Export Repository Module

**Files:**

- Modify: `src-tauri/crates/uc-infra/src/db/repositories/mod.rs`

**Step 1: Add module declaration and export**

Add to the file (after line 4):

```rust
mod representation_repo;

pub use representation_repo::*;
```

**Step 2: Run cargo check**

Run: `cargo check -p uc-infra`
Expected: PASS

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-infra/src/db/repositories/mod.rs
git commit -m "feat(uc-infra): export representation_repo module"
```

---

## Task 5: Add Unit Tests for Mapper

**Files:**

- Create: `src-tauri/crates/uc-infra/src/db/mappers/snapshot_representation_mapper_test.rs`

**Step 1: Create mapper tests**

Create the test file:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::snapshot_representation::SnapshotRepresentationRow;
    use uc_core::{clipboard::PersistedClipboardRepresentation, ids::RepresentationId, FormatId, BlobId, MimeType};

    #[test]
    fn test_row_mapper_all_fields() {
        let mapper = RepresentationRowMapper;
        let row = SnapshotRepresentationRow {
            id: "test-rep-id".to_string(),
            event_id: "test-event-id".to_string(),
            format_id: "public.utf8-plain-text".to_string(),
            mime_type: Some("text/plain".to_string()),
            size_bytes: 42,
            inline_data: Some(vec![1, 2, 3]),
            blob_id: None,
        };

        let result = mapper.to_domain(&row).unwrap();

        assert_eq!(result.id.to_string(), "test-rep-id");
        assert_eq!(result.format_id.to_string(), "public.utf8-plain-text");
        assert_eq!(result.mime_type, Some(MimeType("text/plain".to_string())));
        assert_eq!(result.size_bytes, 42);
        assert_eq!(result.inline_data, Some(vec![1, 2, 3]));
        assert_eq!(result.blob_id, None);
    }

    #[test]
    fn test_row_mapper_optional_fields_none() {
        let mapper = RepresentationRowMapper;
        let row = SnapshotRepresentationRow {
            id: "test-rep-id-2".to_string(),
            event_id: "test-event-id-2".to_string(),
            format_id: "public.png".to_string(),
            mime_type: None,
            size_bytes: 1024,
            inline_data: None,
            blob_id: Some("blob-123".to_string()),
        };

        let result = mapper.to_domain(&row).unwrap();

        assert_eq!(result.id.to_string(), "test-rep-id-2");
        assert_eq!(result.mime_type, None);
        assert_eq!(result.inline_data, None);
        assert_eq!(result.blob_id, Some(BlobId::from("blob-123".to_string())));
    }

    #[test]
    fn test_insert_mapper() {
        let mapper = RepresentationRowMapper;
        let rep = PersistedClipboardRepresentation::new(
            RepresentationId::from("rep-456".to_string()),
            FormatId::from("public.html".to_string()),
            Some(MimeType("text/html".to_string())),
            100,
            Some(vec![10, 20, 30]),
            None,
        );
        let event_id = EventId::from("event-789".to_string());

        let row = mapper.to_row(&(rep.clone(), event_id)).unwrap();

        assert_eq!(row.id, "rep-456");
        assert_eq!(row.event_id, "event-789");
        assert_eq!(row.format_id, "public.html");
        assert_eq!(row.mime_type, Some("text/html".to_string()));
        assert_eq!(row.size_bytes, 100);
        assert_eq!(row.inline_data, Some(vec![10, 20, 30]));
        assert_eq!(row.blob_id, None);
    }
}
```

Add this to the bottom of `snapshot_representation_mapper.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Test content from above...
}
```

**Step 2: Run tests**

Run: `cargo test -p uc-infra snapshot_representation_mapper`
Expected: All tests PASS

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-infra/src/db/mappers/snapshot_representation_mapper.rs
git commit -m "test(uc-infra): add unit tests for RepresentationRowMapper

Covers:
- All fields present conversion
- Optional fields as None
- InsertMapper round-trip"
```

---

## Task 6: Add Integration Tests for Repository

**Files:**

- Create: `src-tauri/crates/uc-infra/src/db/repositories/representation_repo_test.rs`

**Step 1: Create integration test file**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::snapshot_representation::NewSnapshotRepresentationRow;
    use crate::db::schema::clipboard_snapshot_representation;
    use diesel::prelude::*;
    use uc_core::{clipboard::PersistedClipboardRepresentation, ids::{RepresentationId, EventId, FormatId}, MimeType};

    // Note: This requires a test database setup.
    // For now, we provide the test structure that can be run with proper test DB.
    // Actual execution requires test container or in-memory SQLite setup.

    #[tokio::test]
    async fn test_get_representation_found() {
        // TODO: Set up test database connection
        // This test requires DbExecutor implementation for testing

        // let executor = TestDbExecutor::new();
        // let repo = DieselClipboardRepresentationRepository::new(executor);

        // // Insert test data
        // executor.run(|conn| {
        //     diesel::insert_into(clipboard_snapshot_representation::table)
        //         .values(&NewSnapshotRepresentationRow {
        //             id: "test-rep-1".to_string(),
        //             event_id: "test-event-1".to_string(),
        //             format_id: "public.text".to_string(),
        //             mime_type: Some("text/plain".to_string()),
        //             size_bytes: 10,
        //             inline_data: Some(vec![1, 2, 3]),
        //             blob_id: None,
        //         })
        //         .execute(conn)
        //         .unwrap();
        // });

        // let result = repo
        //     .get_representation(
        //         &EventId::from("test-event-1".to_string()),
        //         &RepresentationId::from("test-rep-1".to_string()),
        //     )
        //     .await
        //     .unwrap();

        // assert!(result.is_some());
        // let rep = result.unwrap();
        // assert_eq!(rep.format_id.to_string(), "public.text");
    }

    #[tokio::test]
    async fn test_get_representation_not_found() {
        // TODO: Set up test database
        // Test that Ok(None) is returned for non-existent representation
    }

    #[tokio::test]
    async fn test_update_blob_id() {
        // TODO: Set up test database
        // Test that blob_id is correctly updated
    }
}
```

Add to bottom of `representation_repo.rs`:

```rust
#[cfg(test)]
mod tests {
    // Test content from above
}
```

**Step 2: Run cargo check**

Run: `cargo check -p uc-infra`
Expected: PASS (tests are compiled but may not run without DB setup)

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-infra/src/db/repositories/representation_repo.rs
git commit -m "test(uc-infra): add integration test structure for representation repository

Tests are placeholders pending test database setup.
Test structure covers: found, not_found, and update_blob_id scenarios."
```

---

## Task 7: Verify Full Build

**Files:**

- None (verification)

**Step 1: Run cargo check**

Run: `cargo check -p uc-infra`
Expected: PASS - All modules compile successfully

**Step 2: Run cargo check for core**

Run: `cargo check -p uc-core`
Expected: PASS - Port definition changes are valid

**Step 3: Run full workspace check**

Run: `cargo check`
Expected: PASS - No errors in workspace

**Step 4: Verify exports**

Check that the repository is accessible:

```bash
cargo doc -p uc-infra --no-deps --open
```

Expected: Documentation includes `DieselClipboardRepresentationRepository`

**Step 5: Commit (if any fixes needed)**

```bash
# Only if changes were made during verification
git add -A
git commit -m "fix(uc-infra): resolve verification issues"
```

---

## Task 8: Final Verification and Documentation

**Files:**

- Update: `src-tauri/crates/uc-infra/README.md` (if exists)

**Step 1: Create or update module documentation**

Add to `src-tauri/crates/uc-infra/src/db/repositories/representation_repo.rs` at the top:

````rust
//! Clipboard Representation Repository
//!
//! Implements [`ClipboardRepresentationRepositoryPort`] for querying and updating
//! clipboard snapshot representations stored in SQLite.
//!
//! # Usage
//!
//! ```rust
//! use uc_infra::db::repositories::DieselClipboardRepresentationRepository;
//!
//! let repo = DieselClipboardRepresentationRepository::new(executor);
//!
//! // Query a representation
//! let rep = repo.get_representation(&event_id, &rep_id).await?;
//!
//! // Update blob_id after materialization
//! repo.update_blob_id(&rep_id, &blob_id).await?;
//! ```
````

**Step 2: Final build verification**

Run: `cargo build -p uc-infra`
Expected: SUCCESS

**Step 3: Run clippy**

Run: `cargo clippy -p uc-infra`
Expected: No warnings

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-infra/src/db/repositories/representation_repo.rs
git commit -m "docs(uc-infra): add module documentation for representation repository

Includes usage examples and overview of functionality."
```

---

## Summary

This plan implements `ClipboardRepresentationRepositoryPort` with:

1. **Port definition update** - Return type changed to `Result<Option<...>>`
2. **Mapper implementation** - `RowMapper` for query operations
3. **Repository implementation** - Full Diesel-based repository
4. **Tests** - Unit tests for mapper, structure for integration tests
5. **Documentation** - Module-level docs with usage examples

**Total estimated time**: 1-2 hours

**Dependencies**: None (uses existing infrastructure)

**Testing approach**:

- Unit tests run immediately
- Integration tests require test database setup (documented as TODO)
