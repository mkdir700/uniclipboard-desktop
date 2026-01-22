# Dashboard Image Thumbnail Design

**Goal:** Show image thumbnails in the dashboard list, display a placeholder when the thumbnail is missing, and keep expand loading the preview representation image via `uc://`.

**Architecture:**

- **uc-app:** `ListClipboardEntryProjections` aggregates thumbnail metadata via `ThumbnailRepositoryPort` using `selection.preview_rep_id`. When `content_type` is `image/*` and a thumbnail exists, it returns `thumbnail_url = uc://blob/<thumbnail_blob_id>`. Missing thumbnails are treated as normal; lookup errors are logged and the list continues.
- **uc-tauri:** `ClipboardEntryProjection` exposes `thumbnail_url` to the frontend.
- **frontend:** `getClipboardItems` maps entries by `content_type`. Image entries populate `item.image.thumbnail` from `thumbnail_url`; non-image entries remain text. `ClipboardItem` renders a placeholder if the thumbnail is missing; expand still calls `getClipboardEntryResource` (preview representation) and swaps in the full image URL on success.

**Data Flow:**
`thumbnail_repo` → `ListClipboardEntryProjections` → `ClipboardEntryProjection.thumbnail_url` → `getClipboardItems` → `ClipboardItem` thumbnail/placeholder → expand → `getClipboardEntryResource` → `uc://blob/<blob_id>` → `resolve_blob_resource` → decrypted bytes from `EncryptedBlobStore`.

**Error Handling:**

- Thumbnail lookup failure: log error and continue without `thumbnail_url`.
- Expand failure: toast in the UI; item remains placeholder.

**Testing:**

- `uc-app` unit test for `thumbnail_url` in image projections.
- Frontend API mapping test for image entries.
- Existing `ClipboardItem` expand tests remain valid.
