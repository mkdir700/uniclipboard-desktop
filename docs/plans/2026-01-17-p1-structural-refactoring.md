# P1 Structural Refactoring Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Eliminate dual entry points and architectural boundary violations by (1) unifying watcher startup through Port/use case, and (2) moving cross-repo aggregation logic from commands to uc-app use case layer.

**Architecture:** Follow Hexagonal Architecture principles - composition root uses Port abstractions consistently, business logic lives in use cases, commands only do DTO mapping.

**Tech Stack:** Rust, Tauri 2, async/await with tokio, thiserror for error handling

---

## Overview

This plan addresses two P1 structural issues:

1. **P1-1: Watcher Startup Dual Entry Point**
   - Current: `main.rs:268-269` sends `PlatformCommand::StartClipboardWatcher` directly via channel
   - Fix: Use `runtime.usecases().start_clipboard_watcher().execute()` instead

2. **P1-2: Command Layer Doing Business Logic**
   - Current: `commands/clipboard.rs:get_clipboard_entries` directly accesses repos to aggregate data
   - Fix: Create `ListClipboardEntryProjections` use case that handles cross-repo aggregation

---

## Task 1: P1-1 - Unify Watcher Startup Entry Point

**Files:**

- Modify: `src-tauri/src/main.rs:265-295`

**Step 1: Read the current implementation**

First, examine lines 265-295 of `src-tauri/src/main.rs` to understand the current watcher startup logic.

Run: `bat -n src-tauri/src/main.rs -l 265 -H 295`

**Step 2: Replace direct channel send with use case call**

**Current code (lines 268-279):**

```rust
// 3. Start watcher if encryption is ready
if should_start_watcher {
    match platform_cmd_tx_for_spawn
        .send(PlatformCommand::StartClipboardWatcher)
        .await
    {
        Ok(_) => log::info!("StartClipboardWatcher command sent"),
        Err(e) => {
            log::error!("Failed to send StartClipboardWatcher command: {}", e);
            // Emit error event to frontend for user notification
            let app_handle_guard = runtime_for_unlock.app_handle();
            if let Some(app) = app_handle_guard.as_ref() {
                if let Err(emit_err) =
                    app.emit("clipboard-watcher-start-failed", format!("{}", e))
```

**Replace with:**

```rust
// 3. Start watcher if encryption is ready
if should_start_watcher {
    match runtime_for_unlock
        .usecases()
        .start_clipboard_watcher()
        .execute()
        .await
    {
        Ok(_) => log::info!("Clipboard watcher started successfully"),
        Err(e) => {
            log::error!("Failed to start clipboard watcher: {}", e);
            // Emit error event to frontend for user notification
            let app_handle_guard = runtime_for_unlock.app_handle();
            if let Some(app) = app_handle_guard.as_ref() {
                if let Err(emit_err) =
                    app.emit("clipboard-watcher-start-failed", format!("{}", e))
```

**Step 3: Remove unused platform_cmd_tx_for_spawn variable**

Since we're no longer using `platform_cmd_tx_for_spawn` after the watcher startup, check if it's used elsewhere. If not, it can be removed from the spawn block, but keep it for now as other platform commands may still need it.

**Step 4: Test the change**

Run: `bun tauri dev`

Expected: Application starts successfully, watcher starts when encryption is ready.

Verify in logs: Look for "Clipboard watcher started successfully" instead of "StartClipboardWatcher command sent".

**Step 5: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "refactor(platform): use use case to start clipboard watcher instead of direct channel send

- Replace platform_cmd_tx.send(StartClipboardWatcher) with runtime.usecases().start_clipboard_watcher().execute()
- Maintains consistency with Hexagonal Architecture - composition root uses Port abstraction
- Error handling remains the same, just through use case layer

Related: P1-1 structural refactoring"
```

---

## Task 2: P1-2 - Create ListClipboardEntryProjections Use Case

**Files:**

- Create: `src-tauri/crates/uc-app/src/usecases/clipboard/list_entry_projections/mod.rs`
- Create: `src-tauri/crates/uc-app/src/usecases/clipboard/list_entry_projections/list_entry_projections.rs`
- Modify: `src-tauri/crates/uc-app/src/usecases/clipboard/mod.rs`
- Modify: `src-tauri/crates/uc-app/src/usecases/mod.rs`

### Step 2.1: Create module directory structure

Run: `mkdir -p src-tauri/crates/uc-app/src/usecases/clipboard/list_entry_projections`

### Step 2.2: Create the mod.rs file

**File:** `src-tauri/crates/uc-app/src/usecases/clipboard/list_entry_projections/mod.rs`

```rust
//! Use case for listing clipboard entry projections with cross-repo aggregation
//! 跨仓库聚合的剪贴板条目投影列表用例

mod list_entry_projections;

pub use list_entry_projections::{
    EntryProjectionDto, ListClipboardEntryProjections, ListProjectionsError,
};
```

Run: `cargo check --manifest-path src-tauri/Cargo.toml -p uc-app`

Expected: Compilation error (module doesn't have the implementation yet)

### Step 2.3: Write the failing test first

**File:** `src-tauri/crates/uc-app/src/usecases/clipboard/list_entry_projections/list_entry_projections.rs`

First, write the tests to understand the desired behavior:

```rust
//! Use case for listing clipboard entry projections
//! 列出剪贴板条目投影的用例

use anyhow::Result;
use std::sync::Arc;
use uc_core::clipboard::ClipboardEntry;
use uc_core::ids::{EntryId, EventId};
use uc_core::ports::{
    ClipboardEntryRepositoryPort, ClipboardRepresentationRepositoryPort,
    ClipboardSelectionRepositoryPort,
};

/// DTO for clipboard entry projection (returned to command layer)
/// 剪贴板条目投影 DTO（返回给命令层）
#[derive(Debug, Clone, PartialEq)]
pub struct EntryProjectionDto {
    pub id: String,
    pub preview: String,
    pub has_detail: bool,
    pub size_bytes: i64,
    pub captured_at: i64,
    pub content_type: String,
    // TODO: is_encrypted, is_favorited to be implemented later
    pub is_encrypted: bool,
    pub is_favorited: bool,
    pub updated_at: i64,
    pub active_time: i64,
}

/// Error type for list projections use case
#[derive(Debug, thiserror::Error)]
pub enum ListProjectionsError {
    #[error("Invalid limit: {0}")]
    InvalidLimit(String),

    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error("Selection not found for entry {0}")]
    SelectionNotFound(String),

    #[error("Representation not found: {0}")]
    RepresentationNotFound(String),
}

/// Use case for listing clipboard entry projections
pub struct ListClipboardEntryProjections {
    entry_repo: Arc<dyn ClipboardEntryRepositoryPort>,
    selection_repo: Arc<dyn ClipboardSelectionRepositoryPort>,
    representation_repo: Arc<dyn ClipboardRepresentationRepositoryPort>,
    max_limit: usize,
}

impl ListClipboardEntryProjections {
    /// Create a new use case instance
    pub fn new(
        entry_repo: Arc<dyn ClipboardEntryRepositoryPort>,
        selection_repo: Arc<dyn ClipboardSelectionRepositoryPort>,
        representation_repo: Arc<dyn ClipboardRepresentationRepositoryPort>,
    ) -> Self {
        Self {
            entry_repo,
            selection_repo,
            representation_repo,
            max_limit: 1000,
        }
    }

    /// Execute the use case
    pub async fn execute(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<EntryProjectionDto>, ListProjectionsError> {
        // TODO: Implement this in the next step
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uc_core::ClipboardSelectionDecision;
    use uc_core::ClipboardSnapshotRepresentation;

    // Mock repositories for testing
    struct MockEntryRepository {
        entries: Vec<ClipboardEntry>,
    }

    struct MockSelectionRepository {
        selections: std::collections::HashMap<String, uc_core::ClipboardSelectionDecision>,
    }

    struct MockRepresentationRepository {
        representations: std::collections::HashMap<(String, String), uc_core::ClipboardSnapshotRepresentation>,
    }

    #[async_trait::async_trait]
    impl ClipboardEntryRepositoryPort for MockEntryRepository {
        async fn save_entry_and_selection(
            &self,
            _entry: &ClipboardEntry,
            _selection: &ClipboardSelectionDecision,
        ) -> Result<()> {
            unimplemented!()
        }

        async fn get_entry(&self, _entry_id: &EntryId) -> Result<Option<ClipboardEntry>> {
            unimplemented!()
        }

        async fn list_entries(&self, limit: usize, offset: usize) -> Result<Vec<ClipboardEntry>> {
            Ok(self.entries.iter().skip(offset).take(limit).cloned().collect())
        }

        async fn delete_entry(&self, _entry_id: &EntryId) -> Result<()> {
            unimplemented!()
        }
    }

    #[async_trait::async_trait]
    impl ClipboardSelectionRepositoryPort for MockSelectionRepository {
        async fn save_selection(&self, _selection: &uc_core::ClipboardSelectionDecision) -> Result<()> {
            unimplemented!()
        }

        async fn get_selection(
            &self,
            entry_id: &EntryId,
        ) -> Result<Option<uc_core::ClipboardSelectionDecision>> {
            Ok(self.selections.get(entry_id.inner()).cloned())
        }

        async fn delete_selection(&self, _entry_id: &EntryId) -> Result<()> {
            unimplemented!()
        }
    }

    #[async_trait::async_trait]
    impl ClipboardRepresentationRepositoryPort for MockRepresentationRepository {
        async fn save_representation(
            &self,
            _event_id: &EventId,
            _representation: &ClipboardSnapshotRepresentation,
        ) -> Result<()> {
            unimplemented!()
        }

        async fn get_representation(
            &self,
            event_id: &EventId,
            rep_id: &uc_core::RepresentationId,
        ) -> Result<Option<ClipboardSnapshotRepresentation>> {
            Ok(self.representations.get(&(event_id.inner().clone(), rep_id.inner().clone())).cloned())
        }

        async fn delete_representations(&self, _event_id: &EventId) -> Result<()> {
            unimplemented!()
        }
    }

    fn create_test_entry(id: &str, timestamp: i64) -> ClipboardEntry {
        ClipboardEntry::new(
            EntryId::from_str(id),
            EventId::from_str(id),
            timestamp,
            Some(format!("Entry {}", id)),
            100 * id.len() as i64,
        )
    }

    #[tokio::test]
    async fn test_validates_limit_zero() {
        let entry_repo = Arc::new(MockEntryRepository { entries: vec![] });
        let selection_repo = Arc::new(MockSelectionRepository {
            selections: std::collections::HashMap::new(),
        });
        let representation_repo = Arc::new(MockRepresentationRepository {
            representations: std::collections::HashMap::new(),
        });

        let use_case = ListClipboardEntryProjections::new(
            entry_repo,
            selection_repo,
            representation_repo,
        );

        let result = use_case.execute(0, 0).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ListProjectionsError::InvalidLimit(_)));
    }

    #[tokio::test]
    async fn test_validates_limit_exceeds_max() {
        let entry_repo = Arc::new(MockEntryRepository { entries: vec![] });
        let selection_repo = Arc::new(MockSelectionRepository {
            selections: std::collections::HashMap::new(),
        });
        let representation_repo = Arc::new(MockRepresentationRepository {
            representations: std::collections::HashMap::new(),
        });

        let use_case = ListClipboardEntryProjections::new(
            entry_repo,
            selection_repo,
            representation_repo,
        );

        let result = use_case.execute(2000, 0).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ListProjectionsError::InvalidLimit(_)));
    }
}
```

Run: `cargo test --manifest-path src-tauri/Cargo.toml -p uc-app list_entry_projections`

Expected: Tests fail with "not implemented" or compile errors

### Step 2.4: Implement the execute method

Replace the `execute` method in `list_entry_projections.rs` with the full implementation:

```rust
    /// Execute the use case
    pub async fn execute(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<EntryProjectionDto>, ListProjectionsError> {
        // Validate limit
        if limit == 0 {
            return Err(ListProjectionsError::InvalidLimit(format!(
                "Must be at least 1, got {}",
                limit
            )));
        }

        if limit > self.max_limit {
            return Err(ListProjectionsError::InvalidLimit(format!(
                "Must be at most {}, got {}",
                self.max_limit, limit
            )));
        }

        // Query entries from repository
        let entries = self
            .entry_repo
            .list_entries(limit, offset)
            .await
            .map_err(|e| ListProjectionsError::RepositoryError(e.to_string()))?;

        let mut projections = Vec::with_capacity(entries.len());

        for entry in entries {
            let entry_id_str = entry.entry_id().inner().clone();
            let event_id_str = entry.event_id().inner().clone();
            let captured_at = entry.created_at_ms();

            // Get selection for this entry
            let selection = self
                .selection_repo
                .get_selection(&entry.entry_id())
                .await
                .map_err(|e| {
                    ListProjectionsError::RepositoryError(format!(
                        "Failed to get selection for {}: {}",
                        entry_id_str, e
                    ))
                })?
                .ok_or_else(|| ListProjectionsError::SelectionNotFound(entry_id_str.clone()))?;

            // Get preview representation
            let preview_rep_id = selection.preview_rep_id().inner().clone();
            let representation = self
                .representation_repo
                .get_representation(&entry.event_id(), selection.preview_rep_id())
                .await
                .map_err(|e| {
                    ListProjectionsError::RepositoryError(format!(
                        "Failed to get representation for {}/{}: {}",
                        event_id_str, preview_rep_id, e
                    ))
                })?
                .ok_or_else(|| {
                    ListProjectionsError::RepresentationNotFound(format!(
                        "{}/{}",
                        event_id_str, preview_rep_id
                    ))
                })?;

            // Build preview text
            let preview = if let Some(data) = representation.inline_data() {
                String::from_utf8_lossy(data).trim().to_string()
            } else {
                format!("Image ({} bytes)", representation.size_bytes())
            };

            // Get content type from representation
            let content_type = representation
                .mime_type()
                .as_ref()
                .map(|mt| mt.as_str().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            // Check if has detail (blob exists)
            let has_detail = representation.blob_id().is_some();

            projections.push(EntryProjectionDto {
                id: entry_id_str,
                preview,
                has_detail,
                size_bytes: entry.total_size(),
                captured_at,
                content_type,
                is_encrypted: false,  // TODO: implement later
                is_favorited: false,  // TODO: implement later
                updated_at: captured_at,
                active_time: captured_at,
            });
        }

        Ok(projections)
    }
```

Run: `cargo test --manifest-path src-tauri/Cargo.toml -p uc-app list_entry_projections`

Expected: Basic validation tests pass

### Step 2.5: Add integration test with full data

Add this test to the `tests` module:

```rust
    #[tokio::test]
    async fn test_aggregates_data_from_multiple_repos() {
        use uc_core::ids::{EntryId, EventId, RepresentationId};
        use uc_core::ClipboardSelectionDecision;

        // Setup test data
        let entry = create_test_entry("test-entry-1", 1234567890);
        let entry_id = EntryId::from_str("test-entry-1");
        let event_id = EventId::from_str("test-entry-1");
        let rep_id = RepresentationId::from_str("preview-rep-1");

        let selection = ClipboardSelectionDecision::new(
            entry_id.clone(),
            rep_id.clone(),
            rep_id.clone(),  // full_rep_id same as preview for this test
        );

        let mut representation = ClipboardSnapshotRepresentation::new(
            rep_id.clone(),
            "text/plain".to_string(),
            Some(b"Hello, World!".to_vec()),
            None,
            13,
        );

        let mut entries = vec![entry];
        let mut selections = std::collections::HashMap::new();
        selections.insert("test-entry-1".to_string(), selection);

        let mut representations = std::collections::HashMap::new();
        representations.insert(
            ("test-entry-1".to_string(), "preview-rep-1".to_string()),
            representation,
        );

        let entry_repo = Arc::new(MockEntryRepository { entries });
        let selection_repo = Arc::new(MockSelectionRepository { selections });
        let representation_repo = Arc::new(MockRepresentationRepository { representations });

        let use_case = ListClipboardEntryProjections::new(
            entry_repo,
            selection_repo,
            representation_repo,
        );

        let result = use_case.execute(10, 0).await.unwrap();

        assert_eq!(result.len(), 1);
        let proj = &result[0];
        assert_eq!(proj.id, "test-entry-1");
        assert_eq!(proj.preview, "Hello, World!");
        assert_eq!(proj.content_type, "text/plain");
        assert_eq!(proj.size_bytes, 130); // 10 * 13 (length of "test-entry-1")
        assert_eq!(proj.captured_at, 1234567890);
        assert!(!proj.has_detail); // No blob_id set
    }
```

Run: `cargo test --manifest-path src-tauri/Cargo.toml -p uc-app list_entry_projections`

Expected: All tests pass

### Step 2.6: Export from clipboard mod.rs

**File:** `src-tauri/crates/uc-app/src/usecases/clipboard/mod.rs`

Add:

```rust
pub mod list_entry_projections;

pub use list_entry_projections::{EntryProjectionDto, ListClipboardEntryProjections, ListProjectionsError};
```

Run: `cargo check --manifest-path src-tauri/Cargo.toml -p uc-app`

Expected: Compiles successfully

### Step 2.7: Export from usecases mod.rs

**File:** `src-tauri/crates/uc-app/src/usecases/mod.rs`

Add to the existing exports:

```rust
pub use clipboard::list_entry_projections::{
    EntryProjectionDto, ListClipboardEntryProjections, ListProjectionsError,
};
```

Run: `cargo check --manifest-path src-tauri/Cargo.toml -p uc-app`

Expected: Compiles successfully

### Step 2.8: Commit use case implementation

```bash
git add src-tauri/crates/uc-app/src/usecases/clipboard/
git add src-tauri/crates/uc-app/src/usecases/mod.rs
git commit -m "feat(uc-app): add ListClipboardEntryProjections use case

- New use case for cross-repo aggregation of clipboard entry projections
- Aggregates data from entry_repo, selection_repo, and representation_repo
- Returns EntryProjectionDto with all fields needed by command layer
- Includes comprehensive unit tests

Related: P1-2 structural refactoring"
```

---

## Task 3: Update UseCases Accessor

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`

### Step 3.1: Add accessor method to UseCases

**File:** `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`

Add this method to the `UseCases` impl block (after `start_clipboard_watcher` around line 411):

````rust
    /// List clipboard entry projections (with cross-repo aggregation)
    ///
    /// ## Example / 示例
    ///
    /// ```rust,no_run
    /// # use uc_tauri::bootstrap::AppRuntime;
    /// # use tauri::State;
    /// # async fn example(runtime: State<'_, AppRuntime>) -> Result<Vec<uc_app::EntryProjectionDto>, String> {
    /// let uc = runtime.usecases().list_entry_projections();
    /// let projections = uc.execute(50, 0).await.map_err(|e| e.to_string())?;
    /// # Ok(projections)
    /// # }
    /// ```
    pub fn list_entry_projections(&self) -> uc_app::ListClipboardEntryProjections {
        uc_app::ListClipboardEntryProjections::new(
            self.runtime.deps.clipboard_entry_repo.clone(),
            self.runtime.deps.selection_repo.clone(),
            self.runtime.deps.representation_repo.clone(),
        )
    }
````

Run: `cargo check --manifest-path src-tauri/Cargo.toml -p uc-tauri`

Expected: Compiles successfully

### Step 3.2: Commit accessor update

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs
git commit -m "feat(tauri): add list_entry_projections to UseCases accessor

- Commands can now call runtime.usecases().list_entry_projections()
- Returns use case wired with all required repo dependencies

Related: P1-2 structural refactoring"
```

---

## Task 4: Update Command to Use New Use Case

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/clipboard.rs`

### Step 4.1: Replace get_clipboard_entries implementation

**File:** `src-tauri/crates/uc-tauri/src/commands/clipboard.rs`

Replace the entire `get_clipboard_entries` function (lines 10-201) with:

```rust
/// Get clipboard history entries (preview only)
/// 获取剪贴板历史条目（仅预览）
#[tauri::command]
pub async fn get_clipboard_entries(
    runtime: State<'_, Arc<AppRuntime>>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<ClipboardEntryProjection>, String> {
    let resolved_limit = limit.unwrap_or(50);
    let resolved_offset = offset.unwrap_or(0);
    let device_id = runtime.deps.device_identity.current_device_id();

    let span = info_span!(
        "command.clipboard.get_entries",
        device_id = %device_id,
        limit = resolved_limit,
        offset = resolved_offset,
    );

    async move {
        let uc = runtime.usecases().list_entry_projections();
        let dtos = uc
            .execute(resolved_limit, resolved_offset)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to get clipboard entry projections");
                e.to_string()
            })?;

        // Map DTOs to command layer models
        let projections = dtos
            .into_iter()
            .map(|dto| ClipboardEntryProjection {
                id: dto.id,
                preview: dto.preview,
                has_detail: dto.has_detail,
                size_bytes: dto.size_bytes,
                captured_at: dto.captured_at,
                content_type: dto.content_type,
                is_encrypted: dto.is_encrypted,
                is_favorited: dto.is_favorited,
                updated_at: dto.updated_at,
                active_time: dto.active_time,
            })
            .collect();

        tracing::info!(count = projections.len(), "Retrieved clipboard entries");
        Ok(projections)
    }
    .instrument(span)
    .await
}
```

Run: `cargo check --manifest-path src-tauri/Cargo.toml -p uc-tauri`

Expected: Compiles successfully

### Step 4.2: Run integration tests

Run: `cargo test --manifest-path src-tauri/Cargo.toml -p uc-tauri`

Expected: All tests pass

### Step 4.3: Manual testing

Run: `bun tauri dev`

Expected:

- Application starts successfully
- Clipboard history list displays correctly
- Each entry shows preview, content type, and size

### Step 4.4: Commit command update

```bash
git add src-tauri/crates/uc-tauri/src/commands/clipboard.rs
git commit -m "refactor(commands): use ListClipboardEntryProjections use case

- Replace direct repo access with use case call
- Command now only does simple DTO mapping
- All cross-repo aggregation logic moved to uc-app layer
- Maintains same API contract with frontend

Related: P1-2 structural refactoring"
```

---

## Task 5: Final Verification

### Step 5.1: Run all tests

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: All tests pass

### Step 5.2: Check for unused code

Run: `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`

Expected: No warnings about the changes

### Step 5.3: Build release mode

Run: `cargo build --manifest-path src-tauri/Cargo.toml --release`

Expected: Builds successfully

### Step 5.4: Final commit summary

Create a summary commit if needed:

```bash
git commit --allow-empty -m "docs: summarize P1 structural refactoring completion

P1-1: Unified watcher startup entry point
- Composition root now uses runtime.usecases().start_clipboard_watcher()
- Consistent with Hexagonal Architecture principles

P1-2: Moved cross-repo aggregation to use case layer
- Created ListClipboardEntryProjections use case
- Commands now only do DTO mapping
- Clean separation of concerns maintained"
```

---

## Verification Checklist

- [ ] P1-1: Watcher starts via use case, not direct channel send
- [ ] P1-2: Commands no longer directly access selection_repo/representation_repo
- [ ] P1-2: All cross-repo aggregation logic in uc-app use case
- [ ] All tests pass
- [ ] No clippy warnings
- [ ] Release build succeeds
- [ ] Manual testing confirms clipboard history displays correctly

---

## Architecture Decision Records

### ADR-001: Strict Error Handling in Use Cases

**Decision:** Use case returns errors when selection or representation is not found, rather than returning default values.

**Rationale:**

- Data integrity is a business concern
- Forces frontend to handle error states explicitly
- Makes monitoring and debugging easier
- Command layer can still provide fallback UX if desired

**Alternatives Considered:**

- Return default/fallback values with `is_partial: bool` flag
  - Rejected: Adds complexity, hides real problems
- Return `Option<EntryProjectionDto>` for each entry
  - Rejected: Changes return type significantly, harder to work with

### ADR-002: DTO Definition Location

**Decision:** Define `EntryProjectionDto` in use case module, not in shared models.

**Rationale:**

- Use case owns its output contract
- Command layer can do simple type mapping if needed
- Keeps uc-app independent of uc-tauri (command layer)

**Future Consideration:**

- If multiple commands need this DTO, consider moving to `uc-app/src/models/` or `uc-core/src/models/`
