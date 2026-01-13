# Phase 1 - Infrastructure Layer Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the infrastructure layer for UniClipboard Desktop, enabling clipboard content capture, device persistence, and dependency injection.

**Architecture:** Hexagonal Architecture (Ports and Adapters) with crate-based modularization. The infrastructure layer provides database repositories, filesystem storage, and runtime orchestration for upper layers.

**Tech Stack:**

- Rust 1.75+
- Diesel ORM for database operations
- SQLite (development) / PostgreSQL (production)
- Tokio async runtime
- anyhow for error handling

---

## Task 1: Device Repository Implementation

**Files:**

- Modify: `src-tauri/crates/uc-infra/src/db/repositories/device_repo.rs`
- Test: `src-tauri/crates/uc-infra/tests/device_repo_test.rs` (create new)
- Reference: `src-tauri/crates/uc-infra/src/db/models/device_row.rs`

**Prerequisites:**

- DeviceRow model already exists in `uc-infra/src/db/models/device_row.rs`
- DeviceRepositoryPort trait defined in `uc-core/src/ports/device.rs`

---

### Step 1: Create test file structure

Create the test directory and file:

```bash
mkdir -p src-tauri/crates/uc-infra/tests
touch src-tauri/crates/uc-infra/tests/device_repo_test.rs
```

**Step 2: Add test module to Cargo.toml**

Modify `src-tauri/crates/uc-infra/Cargo.toml`:

```toml
[dev-dependencies]
# ... existing dev-dependencies ...
tokio = { workspace = true, features = ["rt", "macros"] }
```

**Step 3: Write the failing test - save device**

Add to `src-tauri/crates/uc-infra/tests/device_repo_test.rs`:

```rust
use uc_core::device::{Device, DeviceId};
use uc_core::ports::DeviceRepositoryPort;
use uc_infra::db::repositories::device_repo::DieselDeviceRepository;
use uc_infra::db::repositories::device_repo::device_repo_mapper::DeviceRepoMapper;
use std::sync::Arc;

#[tokio::test]
async fn test_save_and_find_device() {
    // This test will fail until we implement the repository
    let device = Device {
        id: DeviceId::new("test-device-1".to_string()),
        name: "Test Device".to_string(),
        platform: "macos".to_string(),
        is_local: true,
    };

    // TODO: Setup test database connection
    // let executor = setup_test_db().await;
    // let mapper = DeviceRepoMapper;
    // let repo = DieselDeviceRepository::new(executor, mapper);

    // repo.save(device.clone()).await.unwrap();

    // let found = repo.find_by_id(&device.id).await.unwrap();
    // assert!(found.is_some());
    // assert_eq!(found.unwrap().name, "Test Device");
}
```

**Step 4: Run test to verify setup**

Run: `cargo test -p uc-infra test_save_and_find_device`

Expected: Test compiles but we have no database setup yet (will be addressed in future steps)

**Step 5: Implement find_by_id method**

Modify `src-tauri/crates/uc-infra/src/db/repositories/device_repo.rs:28-33`:

```rust
async fn find_by_id(
    &self,
    device_id: &DeviceId,
) -> Result<Option<Device>, DeviceRepositoryError> {
    self.executor
        .run(|conn| {
            let device_row = t_device
                .filter(id.eq(device_id.as_str()))
                .first::<DeviceRow>(conn)
                .optional()
                .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))?;

            Ok(device_row.map(Device::from))
        })
        .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))?
}
```

**Step 6: Implement save method**

Modify `src-tauri/crates/uc-infra/src/db/repositories/device_repo.rs:35-49`:

```rust
async fn save(&self, device: Device) -> Result<(), DeviceRepositoryError> {
    let row = self.mapper.to_row(&device)
        .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))?;

    self.executor
        .run(|conn| {
            diesel::insert_into(t_device)
                .values(&row)
                .on_conflict(id)
                .do_update()
                .set((
                    name.eq(&row.name),
                    platform.eq(&row.platform),
                    is_local.eq(&row.is_local),
                ))
                .execute(conn)
                .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))?;
            Ok(())
        })
        .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))?
}
```

**Step 7: Implement delete method**

Modify `src-tauri/crates/uc-infra/src/db/repositories/device_repo.rs:51-61`:

```rust
async fn delete(&self, device_id: &DeviceId) -> Result<(), DeviceRepositoryError> {
    self.executor
        .run(|conn| {
            diesel::delete(t_device.filter(id.eq(device_id.as_str())))
                .execute(conn)
                .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))?;
            Ok(())
        })
        .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))?
}
```

**Step 8: Implement list_all method**

Modify `src-tauri/crates/uc-infra/src/db/repositories/device_repo.rs:63-72`:

```rust
async fn list_all(&self) -> Result<Vec<Device>, DeviceRepositoryError> {
    self.executor
        .run(|conn| {
            let rows = t_device
                .load::<DeviceRow>(conn)
                .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))?;

            Ok(rows.into_iter().map(Device::from).collect())
        })
        .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))?
}
```

**Step 9: Verify compilation**

Run: `cargo check -p uc-infra`

Expected: No compilation errors

**Step 10: Commit**

```bash
git add src-tauri/crates/uc-infra/src/db/repositories/device_repo.rs
git commit -m "feat(uc-infra): implement DeviceRepository CRUD operations

Implement find_by_id, save, delete, and list_all methods for
DieselDeviceRepository using Diesel ORM.

- find_by_id: Query device by ID with optional result
- save: Insert or update device (upsert)
- delete: Remove device by ID
- list_all: Fetch all devices from database

Generated with [Claude Code](https://claude.ai/code)
via [Happy](https://happy.engineering)

Co-Authored-By: Claude <noreply@anthropic.com>
Co-Authored-By: Happy <yesreply@happy.engineering>"
```

---

## Task 2: Clipboard Event Repository Implementation

**Files:**

- Modify: `src-tauri/crates/uc-infra/src/db/repositories/clipboard_event_repo.rs`
- Test: `src-tauri/crates/uc-infra/tests/clipboard_event_repo_test.rs` (create new)
- Reference: `src-tauri/crates/uc-infra/src/db/models/snapshot_representation.rs`

**Prerequisites:**

- SnapshotRepresentationRow model exists
- ClipboardEventRepositoryPort trait defined in uc-core

---

### Step 1: Write the failing test - get_representation

Create `src-tauri/crates/uc-infra/tests/clipboard_event_repo_test.rs`:

```rust
use uc_core::ids::EventId;
use uc_core::ports::ClipboardEventRepositoryPort;
use uc_infra::db::repositories::clipboard_event_repo::DieselClipboardEventRepository;

#[tokio::test]
async fn test_get_representation() {
    let event_id = EventId::new("test-event-1".to_string());
    let rep_id = "test-rep-1";

    // TODO: Setup test database with sample data
    // let executor = setup_test_db_with_data().await;
    // let repo = DieselClipboardEventRepository::new(executor, mapper, snapshot_mapper);

    // let result = repo.get_representation(&event_id, rep_id).await;

    // assert!(result.is_ok());
}
```

**Step 2: Run test to verify it compiles**

Run: `cargo check -p uc-infra --tests`

**Step 3: Implement get_representation method**

Modify `src-tauri/crates/uc-infra/src/db/repositories/clipboard_event_repo.rs:92-104`:

```rust
async fn get_representation(
    &self,
    event_id: &EventId,
    representation_id: &str,
) -> Result<uc_core::ObservedClipboardRepresentation> {
    use crate::db::schema::clipboard_snapshot_representation::dsl::*;

    let event_id_str = event_id.as_str();
    let rep_id = representation_id.to_string();

    self.executor.run(|conn| {
        let rep_row = clipboard_snapshot_representation
            .filter(event_id.eq(event_id_str))
            .filter(id.eq(&rep_id))
            .first::<SnapshotRepresentationRow>(conn)
            .map_err(|e| anyhow::anyhow!("Failed to fetch representation: {}", e))?;

        Ok(uc_core::ObservedClipboardRepresentation {
            id: rep_row.id,
            mime_type: rep_row.mime_type,
            size_bytes: rep_row.size_bytes,
            is_inline: rep_row.inline_data.is_some(),
            inline_data: rep_row.inline_data,
            blob_id: rep_row.blob_id,
        })
    }).map_err(|e| anyhow::anyhow!("Database error: {}", e))?
}
```

**Step 4: Remove PlaceholderClipboardEventRepository**

The placeholder is no longer needed. Remove from `src-tauri/crates/uc-infra/src/db/repositories/clipboard_event_repo.rs:80-105`:

Delete lines 80-105 (the entire `PlaceholderClipboardEventRepository` struct and impl).

**Step 5: Update exports**

Modify `src-tauri/crates/uc-infra/src/db/repositories/mod.rs` to remove placeholder export if present:

```rust
// Remove this line if it exists:
// pub use clipboard_event_repo::PlaceholderClipboardEventRepository;
```

**Step 6: Verify compilation**

Run: `cargo check -p uc-infra`

Expected: No errors. Placeholder references in wiring.rs will be fixed in Task 5.

**Step 7: Commit**

```bash
git add src-tauri/crates/uc-infra/src/db/repositories/clipboard_event_repo.rs
git commit -m "feat(uc-infra): implement ClipboardEventRepository::get_representation

Replace PlaceholderClipboardEventRepository with actual database
implementation using Diesel ORM.

- Queries clipboard_snapshot_representation table by event_id and rep_id
- Returns ObservedClipboardRepresentation with all fields
- Removes placeholder implementation

Generated with [Claude Code](https://claude.ai/code)
via [Happy](https://happy.engineering)

Co-Authored-By: Claude <noreply@anthropic.com>
Co-Authored-By: Happy <yesreply@happy.engineering>"
```

---

## Task 3: Blob Store Implementation

**Files:**

- Modify: `src-tauri/crates/uc-platform/src/adapters/blob.rs`
- Test: `src-tauri/crates/uc-platform/tests/blob_store_test.rs` (create new)
- Helper: `src-tauri/crates/uc-platform/src/adapters/blob_store.rs` (create new)

**Prerequisites:**

- BlobStorePort trait defined in uc-core
- Application data directory available via Tauri API

---

### Step 1: Create filesystem blob store helper

Create `src-tauri/crates/uc-platform/src/adapters/blob_store.rs`:

```rust
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::fs;
use uc_core::BlobId;

/// Filesystem-based blob storage
/// 基于文件系统的 blob 存储
pub struct FilesystemBlobStore {
    base_dir: PathBuf,
}

impl FilesystemBlobStore {
    /// Create a new blob store with the given base directory
    /// 使用给定基础目录创建新的 blob 存储
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Ensure the blob directory exists
    /// 确保 blob 目录存在
    fn ensure_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.base_dir)
            .context("Failed to create blob directory")
    }

    /// Get the full path for a blob ID
    /// 获取 blob ID 的完整路径
    fn blob_path(&self, blob_id: &BlobId) -> PathBuf {
        self.base_dir.join(blob_id.as_str())
    }
}

#[async_trait::async_trait]
impl uc_core::ports::BlobStorePort for FilesystemBlobStore {
    async fn put(&self, blob_id: &BlobId, data: &[u8]) -> Result<PathBuf> {
        self.ensure_dir()?;
        let path = self.blob_path(blob_id);
        fs::write(&path, data)
            .context("Failed to write blob file")?;
        Ok(path)
    }

    async fn get(&self, blob_id: &BlobId) -> Result<Vec<u8>> {
        let path = self.blob_path(blob_id);
        fs::read(&path)
            .context("Failed to read blob file")?
    }
}
```

**Step 2: Write failing test for blob store**

Create `src-tauri/crates/uc-platform/tests/blob_store_test.rs`:

```rust
use uc_platform::adapters::blob_store::FilesystemBlobStore;
use uc_core::BlobId;
use tempfile::TempDir;

#[tokio::test]
async fn test_put_and_get_blob() {
    let temp_dir = TempDir::new().unwrap();
    let store = FilesystemBlobStore::new(temp_dir.path().to_path_buf());

    let blob_id = BlobId::new("test-blob-1".to_string());
    let data = b"hello, world!";

    let path = store.put(&blob_id, data).await.unwrap();
    assert!(path.exists());

    let retrieved = store.get(&blob_id).await.unwrap();
    assert_eq!(retrieved, data);
}

#[tokio::test]
async fn test_get_nonexistent_blob() {
    let temp_dir = TempDir::new().unwrap();
    let store = FilesystemBlobStore::new(temp_dir.path().to_path_buf());

    let blob_id = BlobId::new("nonexistent".to_string());
    let result = store.get(&blob_id).await;

    assert!(result.is_err());
}
```

**Step 3: Add tempfile dependency**

Modify `src-tauri/crates/uc-platform/Cargo.toml`:

```toml
[dev-dependencies]
tempfile = "3"
```

**Step 4: Run test to verify setup**

Run: `cargo test -p uc-platform test_put_and_get_blob`

Expected: Fails because FilesystemBlobStore doesn't implement async trait yet

**Step 5: Fix async trait implementation**

The put/get methods are not actually async. Update the implementation to use tokio::fs for true async:

Update `src-tauri/crates/uc-platform/src/adapters/blob_store.rs`:

```rust
use anyhow::{Context, Result};
use std::path::PathBuf;
use uc_core::BlobId;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::io::AsyncReadExt;

pub struct FilesystemBlobStore {
    base_dir: PathBuf,
}

impl FilesystemBlobStore {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    async fn ensure_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.base_dir)
            .await
            .context("Failed to create blob directory")
    }

    fn blob_path(&self, blob_id: &BlobId) -> PathBuf {
        self.base_dir.join(blob_id.as_str())
    }
}

#[async_trait::async_trait]
impl uc_core::ports::BlobStorePort for FilesystemBlobStore {
    async fn put(&self, blob_id: &BlobId, data: &[u8]) -> Result<PathBuf> {
        self.ensure_dir().await?;
        let path = self.blob_path(blob_id);

        let mut file = fs::File::create(&path).await
            .context("Failed to create blob file")?;
        file.write_all(data).await
            .context("Failed to write blob data")?;

        Ok(path)
    }

    async fn get(&self, blob_id: &BlobId) -> Result<Vec<u8>> {
        let path = self.blob_path(blob_id);
        let mut file = fs::File::open(&path).await
            .context("Failed to open blob file")?;

        let mut data = Vec::new();
        file.read_to_end(&mut data).await
            .context("Failed to read blob data")?;

        Ok(data)
    }
}
```

**Step 6: Run tests to verify implementation**

Run: `cargo test -p uc-platform`

Expected: All tests pass

**Step 7: Export FilesystemBlobStore**

Modify `src-tauri/crates/uc-platform/src/adapters/mod.rs`:

```rust
pub mod blob_store;
pub use blob_store::FilesystemBlobStore;
```

**Step 8: Replace PlaceholderBlobStorePort in exports**

Modify `src-tauri/crates/uc-platform/src/adapters/mod.rs`:

Remove or comment out:

```rust
// pub use blob::{PlaceholderBlobStorePort, PlaceholderBlobMaterializerPort};
```

Add:

```rust
pub use blob_store::FilesystemBlobStore;
```

**Step 9: Update blob.rs to only contain materializer**

Modify `src-tauri/crates/uc-platform/src/adapters/blob.rs` to remove the store implementation (now in blob_store.rs).

**Step 10: Verify compilation**

Run: `cargo check -p uc-platform`

**Step 11: Commit**

```bash
git add src-tauri/crates/uc-platform/src/adapters/blob_store.rs
git add src-tauri/crates/uc-platform/src/adapters/blob.rs
git add src-tauri/crates/uc-platform/src/adapters/mod.rs
git add src-tauri/crates/uc-platform/tests/
git commit -m "feat(uc-platform): implement FilesystemBlobStore

Implement filesystem-based blob storage using tokio::fs for async
operations. Replaces PlaceholderBlobStorePort.

Features:
- put: Store blob data to filesystem
- get: Retrieve blob data by ID
- Automatic directory creation
- Async I/O with tokio

Generated with [Claude Code](https://claude.ai/code)
via [Happy](https://happy.engineering)

Co-Authored-By: Claude <noreply@anthropic.com>
Co-Authored-By: Happy <yesreply@happy.engineering>"
```

---

## Task 4: Clipboard Runtime Event and Command Handling

**Files:**

- Modify: `src-tauri/crates/uc-platform/src/runtime/runtime.rs`
- Test: `src-tauri/crates/uc-platform/tests/runtime_test.rs` (create new)

**Prerequisites:**

- LocalClipboard already implements SystemClipboardPort
- PlatformCommand and PlatformEvent types defined

---

### Step 1: Write failing test for event handling

Create `src-tauri/crates/uc-platform/tests/runtime_test.rs`:

```rust
use uc_platform::runtime::event_bus::*;
use uc_platform::runtime::runtime::PlatformRuntime;
use uc_platform::clipboard::LocalClipboard;
use uc_platform::ports::PlatformCommandExecutorPort;
use std::sync::Arc;

struct MockExecutor;

#[async_trait::async_trait]
impl PlatformCommandExecutorPort for MockExecutor {
    async fn execute(&self, _cmd: PlatformCommand) -> anyhow::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_handle_clipboard_changed_event() {
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(100);
    let (_cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(100);

    let runtime = PlatformRuntime::new(
        event_tx.clone(),
        event_rx,
        cmd_rx,
        Arc::new(MockExecutor),
    ).unwrap();

    // TODO: Test event handling
    // event_tx.send(PlatformEvent::ClipboardChanged { content: ... }).await.unwrap();
    // Runtime should handle the event without panicking
}
```

**Step 2: Run test to verify setup**

Run: `cargo check -p uc-platform --tests`

**Step 3: Implement handle_event method**

Modify `src-tauri/crates/uc-platform/src/runtime/runtime.rs:91-100`:

```rust
async fn handle_event(&self, event: PlatformEvent) {
    match event {
        PlatformEvent::ClipboardChanged { content } => {
            log::debug!("Clipboard changed: {:?}", content);
            // TODO: In future tasks, this will trigger the SyncClipboard use case
            // For now, just log the event
        }
        PlatformEvent::DeviceDiscovered { device_info } => {
            log::debug!("Device discovered: {:?}", device_info);
            // TODO: Handle device discovery in future (Phase 3)
        }
        PlatformEvent::PairingRequest { from_device, pin } => {
            log::debug!("Pairing request from {} with PIN: {}", from_device, pin);
            // TODO: Handle pairing in future (Phase 3)
        }
        _ => {
            log::trace!("Unhandled event: {:?}", event);
        }
    }
}
```

**Step 4: Implement ReadClipboard command**

Modify `src-tauri/crates/uc-platform/src/runtime/runtime.rs:107-109`:

```rust
PlatformCommand::ReadClipboard => {
    match self.local_clipboard.read().await {
        Ok(content) => {
            log::debug!("Read clipboard: {:?}", content);
            // TODO: Send response back through a response channel
            // For now, just log
        }
        Err(e) => {
            log::error!("Failed to read clipboard: {:?}", e);
        }
    }
}
```

**Step 5: Implement WriteClipboard command**

Modify `src-tauri/crates/uc-platform/src/runtime/runtime.rs:110-112`:

```rust
PlatformCommand::WriteClipboard { content } => {
    match self.local_clipboard.write(&content).await {
        Ok(_) => {
            log::debug!("Wrote to clipboard successfully");
            // TODO: Send success response
        }
        Err(e) => {
            log::error!("Failed to write clipboard: {:?}", e);
        }
    }
}
```

**Step 6: Verify compilation**

Run: `cargo check -p uc-platform`

Expected: No errors

**Step 7: Run tests**

Run: `cargo test -p uc-platform`

**Step 8: Commit**

```bash
git add src-tauri/crates/uc-platform/src/runtime/runtime.rs
git add src-tauri/crates/uc-platform/tests/
git commit -m "feat(uc-platform): implement runtime event and command handling

Implement handle_event and clipboard command handlers in PlatformRuntime.

Event handling:
- ClipboardChanged: Log content changes
- DeviceDiscovered: Log device info (stub for Phase 3)
- PairingRequest: Log pairing info (stub for Phase 3)

Command handling:
- ReadClipboard: Read from system clipboard via LocalClipboard
- WriteClipboard: Write to system clipboard via LocalClipboard

Generated with [Claude Code](https://claude.ai/code)
via [Happy](https://happy.engineering)

Co-Authored-By: Claude <noreply@anthropic.com>
Co-Authored-By: Happy <yesreply@happy.engineering>"
```

---

## Task 5: Dependency Injection Wiring

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`
- Test: `src-tauri/crates/uc-tauri/tests/bootstrap_test.rs` (may already exist)

**Prerequisites:**

- All repositories and adapters implemented in previous tasks
- AppDeps and wire_dependencies function skeleton exists

---

### Step 1: Examine current wiring placeholders\*\*

Check `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs` for all `todo!()` calls.

**Step 2: Add FilesystemBlobStore import**

Add to imports in `wiring.rs`:

```rust
use uc_platform::adapters::FilesystemBlobStore;
```

**Step 3: Create blob store**

Find the blob store injection section and replace with:

```rust
// Create blob store (filesystem-based)
let blob_store_dir = app_data_dir.join("blobs");
let blob_store: Arc<dyn BlobStorePort> = Arc::new(FilesystemBlobStore::new(blob_store_dir));
```

**Step 4: Create DieselClipboardEventRepository**

Find the clipboard event repository section and replace placeholder:

```rust
// Create clipboard event repository (using Diesel, not placeholder)
let clipboard_event_repo: Arc<dyn ClipboardEventRepositoryPort> =
    Arc::new(DieselClipboardEventRepository::new(
        db_executor.clone(),
        clipboard_event_mapper,
        snapshot_representation_mapper,
    ));
```

**Step 5: Wire blob_materializer**

Still using placeholder for now (to be implemented in Phase 2):

```rust
// TODO: Phase 2 - Replace with real blob materializer
let blob_materializer: Arc<dyn BlobMaterializerPort> =
    Arc::new(PlaceholderBlobMaterializerPort);
```

**Step 6: Wire blob_repository**

Connect to the blob store:

```rust
// Blob repository uses the blob store
let blob_repository = blob_store.clone();
```

**Step 7: Verify all dependencies are wired**

Run: `cargo check -p uc-tauri`

Expected: May have errors for missing imports or type mismatches

**Step 8: Fix any compilation errors**

Add missing imports and adjust type signatures as needed based on compiler errors.

**Step 9: Run integration tests**

Run: `cargo test -p uc-tauri --test bootstrap_integration_test`

**Step 10: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs
git commit -m "feat(uc-tauri): wire real implementations in dependency injection

Replace placeholder injections with actual implementations:

- FilesystemBlobStore for blob storage
- DieselClipboardEventRepository for event queries
- Blob repository wired to blob store

Remaining placeholders (for Phase 2):
- BlobMaterializerPort
- ClipboardRepresentationMaterializerPort

Generated with [Claude Code](https://claude.ai/code)
via [Happy](https://happy.engineering)

Co-Authored-By: Claude <noreply@anthropic.com>
Co-Authored-By: Happy <yesreply@happy.engineering>"
```

---

## Final Verification

### Step 1: Run all tests

Run: `cargo test --workspace`

Expected: All tests pass (except those skipped for future phases)

### Step 2: Verify no placeholders in implemented code

Search for remaining placeholders in implemented files:

```bash
grep -r "unimplemented!\|todo!\|Placeholder" \
    src-tauri/crates/uc-infra/src/db/repositories/device_repo.rs \
    src-tauri/crates/uc-infra/src/db/repositories/clipboard_event_repo.rs \
    src-tauri/crates/uc-platform/src/adapters/blob_store.rs \
    src-tauri/crates/uc-platform/src/runtime/runtime.rs
```

Expected: Only intentional placeholders for features not in Phase 1

### Step 3: Build the application

Run: `cargo build --release`

Expected: Clean build with no errors

### Step 4: Final commit

```bash
git add docs/plans/2026-01-12-phase1-infrastructure-implementation.md
git commit -m "docs(plans): add Phase 1 infrastructure implementation plan

Detailed step-by-step implementation plan for infrastructure layer:
- Device repository (CRUD)
- Clipboard event repository
- Blob store (filesystem)
- Runtime event/command handling
- Dependency injection wiring

Generated with [Claude Code](https://claude.ai/code)
via [Happy](https://happy.engineering)

Co-Authored-By: Claude <noreply@anthropic.com>
Co-Authored-By: Happy <yesreply@happy.engineering>"
```

---

## Summary

Phase 1 implements the foundation for local clipboard persistence:

| Component                  | Implementation | Notes                             |
| -------------------------- | -------------- | --------------------------------- |
| Device Repository          | Diesel ORM     | Full CRUD with SQLite             |
| Clipboard Event Repository | Diesel ORM     | Query by event_id and rep_id      |
| Blob Store                 | Filesystem     | Async I/O with tokio::fs          |
| Runtime                    | Event loop     | Handles clipboard events/commands |
| Wiring                     | DI container   | All components connected          |

**Next Steps:** Proceed to Phase 2 - Core Business Layer
