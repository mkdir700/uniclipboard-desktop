# ClipboardRepresentationRepositoryPort Implementation Design

## Overview

Design for implementing `ClipboardRepresentationRepositoryPort` in the Hexagonal Architecture.

## Requirements

Support two primary use cases:

1. **Lazy Blob Materialization** - Update `blob_id` after async content materialization
2. **On-Demand Loading** - Query specific representations for history display

## Architecture

### Component Structure

```
uc-infra/src/db/
├── mapper/
│   └── representation_mapper.rs    # Mapper trait implementations
├── repositories/
│   └── representation_repo.rs      # Repository implementation
└── mod.rs                          # Module exports
```

### Core Components

| Component                                        | Location                              | Responsibility                            |
| ------------------------------------------------ | ------------------------------------- | ----------------------------------------- |
| `DieselClipboardRepresentationRepository<E, RM>` | `repositories/representation_repo.rs` | Implements port, encapsulates DB queries  |
| `SnapshotRepresentationRowMapper`                | `mapper/representation_mapper.rs`     | Row → Domain model conversion             |
| `SnapshotRepresentationInsertMapper`             | `mapper/representation_mapper.rs`     | Domain → Insert row conversion (optional) |

### Database Table

Uses existing `clipboard_snapshot_representation` table:

| Column        | Type              | Maps To            |
| ------------- | ----------------- | ------------------ |
| `id`          | Text (PK)         | `RepresentationId` |
| `event_id`    | Text (FK)         | `EventId`          |
| `format_id`   | Text              | `FormatId`         |
| `mime_type`   | Text (nullable)   | `Option<MimeType>` |
| `size_bytes`  | BigInt            | `i64`              |
| `inline_data` | Binary (nullable) | `Option<Vec<u8>>`  |
| `blob_id`     | Text (nullable)   | `Option<BlobId>`   |

## Method Implementations

### `get_representation`

**Signature** (proposed change):

```rust
async fn get_representation(
    &self,
    event_id: &EventId,
    representation_id: &RepresentationId,
) -> Result<Option<PersistedClipboardRepresentation>>
```

**Logic**:

1. Query with composite key filter: `event_id` + `id`
2. Use `.optional()` for graceful handling of missing records
3. Convert `SnapshotRepresentationRow` → `PersistedClipboardRepresentation` via `RowMapper`
4. Return `Ok(None)` if not found (caller decides error handling)

**Note**: Current port definition returns `Result<PersistedClipboardRepresentation>`. Recommend changing to `Result<Option<...>>` for consistency with query patterns like `BlobRepositoryPort::find_by_hash`.

### `update_blob_id`

**Signature**:

```rust
async fn update_blob_id(
    &self,
    representation_id: &RepresentationId,
    blob_id: &BlobId,
) -> Result<()>
```

**Logic**:

1. Filter by `id.eq(representation_id)`
2. Set `blob_id` to new value
3. Execute update
4. Return `Ok(())` regardless of affected rows (simple update pattern)

**Rationale**: Independent update following single responsibility principle. Blob existence verified at use case layer or via foreign key constraint.

## Error Handling

Follow project error handling standards:

1. **Diesel errors** → Wrap with context using `anyhow!`

   ```rust
   .map_err(|e| anyhow::anyhow!("Failed to query representation: event={}, id={}: {}", event_id, representation_id, e))
   ```

2. **ID parsing errors** → Convert to domain errors

   ```rust
   RepresentationId::new(row.id).map_err(|e| anyhow::anyhow!("Invalid representation_id: {}", e))?
   ```

3. **NotFound** → Return `Ok(None)`, caller determines if error

## Testing Strategy

### Unit Tests

1. **Mapper Tests**
   - Valid conversion (all fields present)
   - Optional fields as `None`
   - Invalid ID format error handling

2. **Repository Query Tests** (test container or in-memory SQLite)
   - Existing record → `Ok(Some(model))`
   - Non-existent record → `Ok(None)`
   - DB connection error → `Err`

3. **Repository Update Tests**
   - Existing record → `blob_id` updated correctly
   - Non-existent record → `Ok(())` (no-op)

## Future Extensions

Potential additions (out of scope for initial implementation):

1. **Batch query** - `get_representations_by_event(event_id)`
2. **Delete method** - `delete_representation(id)` for cleanup
3. **Insert method** - `insert_representation(rep)` for independent operations

## Performance Considerations

- Index on `event_id` may be beneficial if frequently querying all representations for an event
- Batch update interface could reduce transaction overhead for bulk materialization

## Integration Points

### Module Exports

Add to `uc-infra/src/db/repositories/mod.rs`:

```rust
mod representation_repo;
pub use representation_repo::*;
```

Create `uc-infra/src/db/mapper/mod.rs`:

```rust
mod representation_mapper;
pub use representation_mapper::*;
```

### Dependencies

- `uc-core` - Domain models and port definitions
- `uc-infra::db::ports` - `DbExecutor`, `RowMapper`, `InsertMapper` traits
- `diesel` - Query DSL and execution
- `anyhow` - Error handling
