# Thumbnail UC Protocol Fix Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make image thumbnails resolvable via a dedicated `uc://thumbnail/<representation_id>` path so thumbnails display reliably without requiring a representation lookup by thumbnail blob id.

**Architecture:** Add a thumbnail resolver use case in `uc-app` that reads thumbnail metadata via `ThumbnailRepositoryPort` and bytes via `BlobStorePort`. Extend the Tauri `uc://` protocol handler to route `thumbnail` requests to this use case. Emit `thumbnail_url` as `uc://thumbnail/<preview_rep_id>` in projections.

**Tech Stack:** Rust (uc-core/uc-app/uc-tauri), Tauri custom protocol, TypeScript/Vite frontend, Vitest.

---

### Task 1: Add thumbnail resolver use case (uc-app)

**Files:**

- Create: `src-tauri/crates/uc-app/src/usecases/clipboard/resolve_thumbnail_resource.rs`
- Modify: `src-tauri/crates/uc-app/src/usecases/clipboard/mod.rs`
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`
- Test: `src-tauri/crates/uc-app/src/usecases/clipboard/resolve_thumbnail_resource.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn test_resolve_thumbnail_resource_returns_bytes() {
    let rep_id = RepresentationId::from("rep-1");
    let blob_id = BlobId::from("thumb-1");
    let metadata = ThumbnailMetadata::new(
        rep_id.clone(),
        blob_id.clone(),
        MimeType("image/webp".to_string()),
        120,
        80,
        1024,
        None,
    );

    let uc = ResolveThumbnailResourceUseCase::new(
        Arc::new(MockThumbnailRepo { metadata: Some(metadata) }),
        Arc::new(MockBlobStore { blob_id: blob_id.clone(), bytes: vec![1, 2, 3] }),
    );

    let result = uc.execute(&rep_id).await.unwrap();
    assert_eq!(result.mime_type, Some("image/webp".to_string()));
    assert_eq!(result.bytes, vec![1, 2, 3]);
}
```

**Step 2: Run test to verify it fails**

Run: `RUSTC_WRAPPER= cargo test -p uc-app resolve_thumbnail_resource` (from `src-tauri/`)  
Expected: FAIL with missing module / type not found.

**Step 3: Write minimal implementation**

```rust
pub struct ResolveThumbnailResourceUseCase {
    thumbnail_repo: Arc<dyn ThumbnailRepositoryPort>,
    blob_store: Arc<dyn BlobStorePort>,
}

pub async fn execute(&self, representation_id: &RepresentationId) -> Result<ThumbnailResourceResult> {
    let metadata = self
        .thumbnail_repo
        .get_by_representation_id(representation_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Thumbnail not found"))?;
    let bytes = self.blob_store.get(&metadata.thumbnail_blob_id).await?;
    Ok(ThumbnailResourceResult {
        representation_id: representation_id.clone(),
        mime_type: Some(metadata.thumbnail_mime_type.as_str().to_string()),
        bytes,
    })
}
```

**Step 4: Run test to verify it passes**

Run: `RUSTC_WRAPPER= cargo test -p uc-app resolve_thumbnail_resource`  
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/clipboard/resolve_thumbnail_resource.rs \
        src-tauri/crates/uc-app/src/usecases/clipboard/mod.rs \
        src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs
git commit -m "feat(uc-app): add thumbnail resolver use case"
```

---

### Task 2: Route `uc://thumbnail/<representation_id>` in Tauri protocol handler

**Files:**

- Modify: `src-tauri/src/main.rs`

**Step 1: Write the failing test**

Use an integration-style unit test in `src-tauri/src/main.rs` or a small helper test module to assert `thumbnail` host routes to the new resolver (if no current test harness, add a minimal unit test for host routing).

**Step 2: Run test to verify it fails**

Run: `RUSTC_WRAPPER= cargo test -p uc-tauri thumbnail_protocol` (from `src-tauri/`)  
Expected: FAIL due to missing handler.

**Step 3: Write minimal implementation**

```rust
match host {
    "blob" => resolve_uc_blob_request(app_handle, request).await,
    "thumbnail" => resolve_uc_thumbnail_request(app_handle, request).await,
    _ => text_response(StatusCode::BAD_REQUEST, "Unsupported uc URI host", origin),
}
```

`resolve_uc_thumbnail_request` should parse representation id, call `runtime.usecases().resolve_thumbnail_resource()`, and build response with mime/bytes.

**Step 4: Run test to verify it passes**

Run: `RUSTC_WRAPPER= cargo test -p uc-tauri thumbnail_protocol`  
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "feat(tauri): add uc thumbnail protocol handler"
```

---

### Task 3: Emit thumbnail_url as `uc://thumbnail/<preview_rep_id>` and update tests

**Files:**

- Modify: `src-tauri/crates/uc-app/src/usecases/clipboard/list_entry_projections/list_entry_projections.rs`
- Modify: `src/api/__tests__/clipboardItems.test.ts`

**Step 1: Write the failing test**

Update existing uc-app test to expect `thumbnail_url == "uc://thumbnail/<rep_id>"`.

**Step 2: Run test to verify it fails**

Run: `RUSTC_WRAPPER= cargo test -p uc-app list_entry_projections`  
Expected: FAIL with mismatched URL.

**Step 3: Write minimal implementation**

```rust
let thumbnail_url = if is_image {
    match self.thumbnail_repo.get_by_representation_id(&selection.selection.preview_rep_id).await {
        Ok(Some(_)) => Some(format!("uc://thumbnail/{}", preview_rep_id)),
        Ok(None) => None,
        Err(err) => { tracing::error!(...); None }
    }
} else {
    None
};
```

**Step 4: Run test to verify it passes**

Run: `RUSTC_WRAPPER= cargo test -p uc-app list_entry_projections`  
Expected: PASS.

**Step 5: Update frontend mapping test**

Update expected thumbnail URL string in `src/api/__tests__/clipboardItems.test.ts`.

**Step 6: Run frontend test**

Run: `bun run test --run src/api/__tests__/clipboardItems.test.ts`  
Expected: PASS.

**Step 7: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/clipboard/list_entry_projections/list_entry_projections.rs \
        src/api/__tests__/clipboardItems.test.ts
git commit -m "feat: use uc thumbnail url in projections"
```

---

### Task 4: Smoke verification

**Files:**

- None

**Step 1: Manual verification**

- Copy a new image.
- Confirm dashboard shows thumbnail (no `Representation not found` errors).
- Click expand and confirm full preview loads via `get_clipboard_entry_resource`.

**Step 2: Commit**

```bash
# No commit needed if no code changes
```
