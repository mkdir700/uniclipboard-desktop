# Clipboard Restore Single Representation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Restore clipboard entries by writing only the primary representation to avoid platform write failures.

**Architecture:** Keep the multi-representation selection in storage, but downgrade restore output in `uc-app` to a single representation until platform adapters support multi-format atomic writes. This preserves hexagonal boundaries: the decision lives in the use case layer without touching `uc-platform`.

**Tech Stack:** Rust, Tauri, uc-app use cases, uc-core ports, clipboard-rs (indirect).

---

### Task 1: Add failing test for single-representation restore behavior

**Files:**

- Modify: `src-tauri/crates/uc-app/src/usecases/clipboard/restore_clipboard_selection.rs`

**Step 1: Write the failing test**

Add a `#[cfg(test)]` module that builds a use case with in-memory mocks for:

- `ClipboardEntryRepositoryPort`
- `ClipboardSelectionRepositoryPort`
- `ClipboardRepresentationRepositoryPort`
- `BlobStorePort`
- `SystemClipboardPort`

Test behavior: when `selection.selection` contains both `paste_rep_id` and `secondary_rep_ids`, `build_snapshot` should return exactly **one** representation matching `paste_rep_id`.

```rust
#[tokio::test]
async fn build_snapshot_returns_only_paste_representation() {
    // Arrange: entry + selection with secondary rep id
    // Arrange: representation repo returns inline data for both reps
    // Act: build_snapshot
    // Assert: snapshot.representations.len() == 1
    // Assert: snapshot.representations[0].id == paste_rep_id
}
```

**Step 2: Run test to verify it fails**

Run (from `src-tauri/`):

```bash
cargo test -p uc-app build_snapshot_returns_only_paste_representation
```

Expected: FAIL because the current implementation returns both primary and secondary representations.

### Task 2: Implement minimal change to restore only the primary representation

**Files:**

- Modify: `src-tauri/crates/uc-app/src/usecases/clipboard/restore_clipboard_selection.rs`

**Step 1: Write minimal implementation**

Update `build_snapshot` to only include the primary `paste_rep_id` in the `rep_ids` list.

```rust
let rep_ids = vec![selection.selection.paste_rep_id];
```

**Step 2: Run test to verify it passes**

Run (from `src-tauri/`):

```bash
cargo test -p uc-app build_snapshot_returns_only_paste_representation
```

Expected: PASS.

### Task 3: Add ADR for single-representation restore decision

**Files:**

- Create: `docs/architecture/adr-004-restore-single-representation.md`

**Step 1: Write ADR**

Use the existing ADR format (`docs/architecture/adr-003-thumbnail-resource-protocol.md`). Include:

- Context: multi-representation restore fails because platform write only supports one representation.
- Decision: downgrade restore to primary representation only in `RestoreClipboardSelectionUseCase`.
- Consequences: restore fidelity reduced; multi-format restore deferred to platform-specific atomic write support.

### Task 4: Create GitHub Issue tracking multi-representation restore

**Files:**

- None (GitHub only)

**Step 1: Create issue**

Run:

```bash
gh issue create --title "Clipboard restore fails with multiple representations" --body "## Problem\nRestoring a clipboard entry that includes multiple representations (e.g. text + html) fails with `platform::write expects exactly ONE representation`. The platform adapter currently uses clipboard-rs high-level APIs which overwrite clipboard content on each call, so multi-format atomic writes are unsupported.\n\n## Current Behavior\n- `RestoreClipboardSelectionUseCase::build_snapshot` returns primary + secondary representations.\n- `CommonClipboardImpl::write_snapshot` enforces a single representation and returns an error.\n\n## Impact\nRestoring entries that include multiple representations fails, and the restore command returns an error to the UI.\n\n## Next Steps\n- Implement platform-specific atomic multi-format write paths (macOS NSPasteboardItem, Windows CF_* formats, Linux targets).\n- Update uc-platform write path to accept multi-representation snapshots.\n\n## Related\n- TODO in `src-tauri/crates/uc-platform/src/clipboard/common.rs` (multi-representation restore is lossy)\n- ADR-004: Restore single representation (temporary workaround)"
```

### Task 5: Run diagnostics

**Files:**

- Check: `src-tauri/crates/uc-app/src/usecases/clipboard/restore_clipboard_selection.rs`

**Step 1: Run LSP diagnostics**

Use `lsp_diagnostics` on the modified Rust file and confirm no errors or warnings.
