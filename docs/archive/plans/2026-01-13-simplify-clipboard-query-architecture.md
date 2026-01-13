# Simplify Clipboard Query Architecture Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove redundant projection models and implement proper hexagonal architecture for clipboard queries with use cases.

**Architecture:**

- Delete `uc-app::models::ClipboardEntryProjection` (redundant middle layer)
- Create `ListClipboardEntries` use case in `uc-app/usecases/`
- Tauri commands call use case, use case calls repository port
- DTO conversion happens in Tauri command layer

**Tech Stack:** Rust, Tauri 2, Hexagonal Architecture (Ports and Adapters)

---

## Task 1: Delete redundant projection model file

**Files:**

- Delete: `src-tauri/crates/uc-app/src/models/clipboard_entry_projection.rs`

**Step 1: Verify file exists and check current usage**

Run: `ls -la src-tauri/crates/uc-app/src/models/clipboard_entry_projection.rs`
Expected: File exists

**Step 2: Delete the file**

Run: `rm src-tauri/crates/uc-app/src/models/clipboard_entry_projection.rs`

**Step 3: Verify compilation fails (expected)**

Run: `cd src-tauri && cargo check --message-format=short 2>&1 | head -20`
Expected: Compilation errors in `uc-app/src/models/mod.rs` and `uc-app/src/lib.rs`

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-app/src/models/clipboard_entry_projection.rs
git commit -m "refactor(uc-app): remove redundant ClipboardEntryProjection model"
```

---

## Task 2: Update uc-app models module

**Files:**

- Modify: `src-tauri/crates/uc-app/src/models/mod.rs`

**Step 1: Remove projection-related declarations**

Edit the file to remove these lines:

```rust
mod clipboard_entry_projection;

pub use clipboard_entry_projection::*;
```

The file should now start with the `MaterializedPayload` enum (keep everything else).

**Step 2: Verify the change**

Run: `head -10 src-tauri/crates/uc-app/src/models/mod.rs`
Expected: File starts with `// TODO: 暂时不知道如何分类，先写这里。`

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-app/src/models/mod.rs
git commit -m "refactor(uc-app): remove projection module imports"
```

---

## Task 3: Remove ClipboardEntryProjectionQueryPort trait

**Files:**

- Modify: `src-tauri/crates/uc-app/src/ports.rs`

**Step 1: Read current content**

Run: `cat src-tauri/crates/uc-app/src/ports.rs`
Expected: File contains `ClipboardEntryProjectionQueryPort` trait

**Step 2: Delete the entire file content (port is no longer needed)**

The query will go directly through `ClipboardEntryRepositoryPort` which already exists in `uc-core`.

Run: `rm src-tauri/crates/uc-app/src/ports.rs`

**Step 3: Verify no other files import this port**

Run: `grep -r "ClipboardEntryProjectionQueryPort" src-tauri/crates/`
Expected: No matches (previously checked - only used in deleted projection.rs)

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-app/src/ports.rs
git commit -m "refactor(uc-app): remove ClipboardEntryProjectionQueryPort"
```

---

## Task 4: Update uc-app lib.rs exports

**Files:**

- Modify: `src-tauri/crates/uc-app/src/lib.rs`

**Step 1: Remove the projection export**

Edit line 16 to remove:

```rust
pub use models::ClipboardEntryProjection;
```

**Step 2: Verify the file compiles**

Run: `cd src-tauri && cargo check -p uc-app --message-format=short 2>&1 | head -20`
Expected: May have errors about missing `ports` module, fix in next step

**Step 3: Remove ports module declaration if it causes errors**

Edit to remove this line if present:

```rust
pub mod ports;
```

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-app/src/lib.rs
git commit -m "refactor(uc-app): remove ClipboardEntryProjection export"
```

---

## Task 5: Delete ClipboardProjectionReader

**Files:**

- Delete: `src-tauri/crates/uc-infra/src/clipboard/projection.rs`

**Step 1: Verify file is unused**

Run: `grep -r "ClipboardProjectionReader" src-tauri/crates/ --include="*.rs"`
Expected: Only found in the file itself and already commented out in mod.rs

**Step 2: Delete the file**

Run: `rm src-tauri/crates/uc-infra/src/clipboard/projection.rs`

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-infra/src/clipboard/projection.rs
git rm src-tauri/crates/uc-infra/src/clipboard/projection.rs
git commit -m "refactor(uc-infra): remove unused ClipboardProjectionReader"
```

---

## Task 6: Clean up uc-infra clipboard module

**Files:**

- Modify: `src-tauri/crates/uc-infra/src/clipboard/mod.rs`

**Step 1: Remove commented projection lines**

Edit file to remove these lines:

```rust
// mod projection;
// pub use projection::ClipboardProjectionReader;
```

**Step 2: Verify file only contains materializer**

Run: `cat src-tauri/crates/uc-infra/src/clipboard/mod.rs`
Expected: Only contains `mod materializer;` and `pub use materializer::...`

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-infra/src/clipboard/mod.rs
git commit -m "refactor(uc-infra): clean up commented projection imports"
```

---

## Task 7: Create ListClipboardEntries use case

**Files:**

- Create: `src-tauri/crates/uc-app/src/usecases/list_clipboard_entries.rs`

**Step 1: Create the use case file**

Create file with this content:

```rust
use anyhow::Result;
use uc_core::clipboard::ClipboardEntry;
use uc_core::ports::ClipboardEntryRepositoryPort;

/// Use case for listing clipboard entries with pagination
/// 列出剪贴板条目的用例（分页）
pub struct ListClipboardEntries<R> {
    entry_repo: R,
    max_limit: usize,
}

impl<R: ClipboardEntryRepositoryPort> ListClipboardEntries<R> {
    /// Create a new use case instance
    /// 创建新的用例实例
    pub fn new(entry_repo: R) -> Self {
        Self {
            entry_repo,
            max_limit: 1000, // Business rule: maximum 1000 entries per query
        }
    }

    /// Execute the query
    /// 执行查询
    ///
    /// # Arguments
    /// * `limit` - Maximum number of entries to return (1 to max_limit)
    /// * `offset` - Number of entries to skip
    ///
    /// # Returns
    /// Vector of clipboard entries
    ///
    /// # Errors
    /// Returns error if:
    /// - Limit is 0 or exceeds max_limit
    /// - Repository query fails
    pub async fn execute(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<ClipboardEntry>> {
        // Validate limit
        if limit == 0 {
            return Err(anyhow::anyhow!(
                "Invalid limit: {}. Must be at least 1",
                limit
            ));
        }

        if limit > self.max_limit {
            return Err(anyhow::anyhow!(
                "Invalid limit: {}. Must be at most {}",
                limit,
                self.max_limit
            ));
        }

        // Query repository
        self.entry_repo
            .list_entries(limit, offset)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to query clipboard entries: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uc_core::ids::{EntryId, EventId};
    use std::sync::Arc;

    // Mock repository for testing
    struct MockClipboardEntryRepository {
        entries: Vec<ClipboardEntry>,
        should_fail: bool,
    }

    #[async_trait::async_trait]
    impl ClipboardEntryRepositoryPort for MockClipboardEntryRepository {
        async fn save_entry_and_selection(
            &self,
            _entry: &ClipboardEntry,
            _selection: &uc_core::ClipboardSelectionDecision,
        ) -> Result<()> {
            unimplemented!()
        }

        async fn get_entry(&self, _entry_id: &EntryId) -> Result<Option<ClipboardEntry>> {
            unimplemented!()
        }

        async fn list_entries(
            &self,
            limit: usize,
            offset: usize,
        ) -> Result<Vec<ClipboardEntry>> {
            if self.should_fail {
                return Err(anyhow::anyhow!("Mock repository error"));
            }
            Ok(self
                .entries
                .iter()
                .skip(offset)
                .take(limit)
                .cloned()
                .collect())
        }
    }

    fn create_test_entry(id: u32, timestamp: i64) -> ClipboardEntry {
        ClipboardEntry::new(
            EntryId::from_u32(id),
            EventId::from_u32(id),
            timestamp,
            Some(format!("Entry {}", id)),
            100 * id as i64,
        )
    }

    #[tokio::test]
    async fn test_execute_returns_entries() {
        let entries = vec![
            create_test_entry(1, 1000),
            create_test_entry(2, 2000),
            create_test_entry(3, 3000),
        ];

        let repo = MockClipboardEntryRepository {
            entries,
            should_fail: false,
        };

        let use_case = ListClipboardEntries::new(repo);
        let result = use_case.execute(10, 0).await.unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].entry_id.to_u32(), 1);
    }

    #[tokio::test]
    async fn test_execute_respects_limit() {
        let entries = vec![
            create_test_entry(1, 1000),
            create_test_entry(2, 2000),
            create_test_entry(3, 3000),
        ];

        let repo = MockClipboardEntryRepository {
            entries,
            should_fail: false,
        };

        let use_case = ListClipboardEntries::new(repo);
        let result = use_case.execute(2, 0).await.unwrap();

        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_execute_respects_offset() {
        let entries = vec![
            create_test_entry(1, 1000),
            create_test_entry(2, 2000),
            create_test_entry(3, 3000),
        ];

        let repo = MockClipboardEntryRepository {
            entries,
            should_fail: false,
        };

        let use_case = ListClipboardEntries::new(repo);
        let result = use_case.execute(10, 1).await.unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].entry_id.to_u32(), 2);
    }

    #[tokio::test]
    async fn test_execute_rejects_zero_limit() {
        let repo = MockClipboardEntryRepository {
            entries: vec![],
            should_fail: false,
        };

        let use_case = ListClipboardEntries::new(repo);
        let result = use_case.execute(0, 0).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid limit"));
    }

    #[tokio::test]
    async fn test_execute_rejects_excessive_limit() {
        let repo = MockClipboardEntryRepository {
            entries: vec![],
            should_fail: false,
        };

        let use_case = ListClipboardEntries::new(repo);
        let result = use_case.execute(2000, 0).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Must be at most"));
    }

    #[tokio::test]
    async fn test_execute_propagates_repository_errors() {
        let repo = MockClipboardEntryRepository {
            entries: vec![],
            should_fail: true,
        };

        let use_case = ListClipboardEntries::new(repo);
        let result = use_case.execute(10, 0).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to query"));
    }
}
```

**Note:** This assumes `EntryId` and `EventId` have `from_u32()` and `to_u32()` methods. If not, we'll adjust in a follow-up task.

**Step 2: Run tests to verify they compile and pass**

Run: `cd src-tauri && cargo test -p uc-app list_clipboard_entries --message-format=short`
Expected: Tests compile and pass (may need adjustment based on actual ID types)

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/list_clipboard_entries.rs
git commit -m "feat(uc-app): add ListClipboardEntries use case with tests"
```

---

## Task 8: Export the new use case

**Files:**

- Modify: `src-tauri/crates/uc-app/src/usecases/mod.rs`

**Step 1: Add the use case module and export**

Read the current file to understand the structure, then add:

```rust
pub mod list_clipboard_entries;

pub use list_clipboard_entries::ListClipboardEntries;
```

**Step 2: Verify it compiles**

Run: `cd src-tauri && cargo check -p uc-app --message-format=short 2>&1 | head -20`
Expected: No errors

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/mod.rs
git commit -m "refactor(uc-app): export ListClipboardEntries use case"
```

---

## Task 9: Update Tauri command to use the use case

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/clipboard.rs`

**Step 1: Read current implementation**

Run: `cat src-tauri/crates/uc-tauri/src/commands/clipboard.rs`
Expected: See current implementation that directly calls repository

**Step 2: Replace implementation with use case**

Replace the entire `get_clipboard_entries` function with:

```rust
/// Get clipboard history entries
/// 获取剪贴板历史条目
#[tauri::command]
pub async fn get_clipboard_entries(
    deps: State<'_, AppDeps>,
    limit: Option<usize>,
) -> Result<Vec<ClipboardEntryProjection>, String> {
    use uc_app::usecases::ListClipboardEntries;

    let use_case = ListClipboardEntries::new(deps.clipboard_entry_repo.clone());
    let limit = limit.unwrap_or(50);

    // Query entries through use case
    let entries = use_case
        .execute(limit, 0)
        .await
        .map_err(|e| e.to_string())?;

    // Convert domain models to DTOs
    let projections: Vec<ClipboardEntryProjection> = entries
        .into_iter()
        .map(|entry| ClipboardEntryProjection {
            id: entry.entry_id.to_string(),
            preview: entry.title.unwrap_or_else(|| format!("Entry ({} bytes)", entry.total_size)),
            captured_at: entry.created_at_ms,
            content_type: "clipboard".to_string(),
            is_encrypted: false, // TODO: Determine from actual entry state
        })
        .collect();

    Ok(projections)
}
```

**Step 3: Verify compilation**

Run: `cd src-tauri && cargo check -p uc-tauri --message-format=short 2>&1 | head -30`
Expected: Compiles successfully

**Step 4: Run cargo test to ensure no regressions**

Run: `cd src-tauri && cargo test --message-format=short 2>&1 | tail -20`
Expected: All tests pass

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/clipboard.rs
git commit -m "refactor(uc-tauri): use ListClipboardEntries use case in command"
```

---

## Task 10: Final integration test

**Files:**

- Test: `src-tauri/crates/uc-tauri/tests/`

**Step 1: Run full workspace test**

Run: `cd src-tauri && cargo test --workspace --message-format=short 2>&1 | tail -30`
Expected: All tests pass

**Step 2: Build the project**

Run: `cd src-tauri && cargo build --message-format=short 2>&1 | tail -20`
Expected: Builds successfully

**Step 3: Verify no unused imports or dead code**

Run: `cd src-tauri && cargo clippy --all-targets --message-format=short 2>&1 | grep -A5 "clipboard\|projection" | head -40`
Expected: No warnings related to removed projection code

**Step 4: Final commit**

```bash
git add src-tauri/
git commit -m "test: verify clipboard query refactoring passes all checks"
```

---

## Verification Checklist

After completing all tasks:

- [ ] `ClipboardEntryProjection` removed from `uc-app`
- [ ] `ClipboardProjectionReader` removed from `uc-infra`
- [ ] `ListClipboardEntries` use case created and tested
- [ ] Tauri command uses use case instead of direct repository access
- [ ] All tests pass
- [ ] No compilation warnings
- [ ] Architecture follows hexagonal pattern (Command → UseCase → Port → Repository)

---

## References

- Hexagonal Architecture: Port definitions in `uc-core/src/ports/`
- Existing use cases: `uc-app/src/usecases/initialize_encryption.rs` (for reference)
- Tauri commands: `uc-tauri/src/commands/`
- DTO definitions: `uc-tauri/src/models/mod.rs`
