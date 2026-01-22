# Thumbnail Generation Design (Worker)

## Summary / 摘要

Add thumbnail generation for image representations after blob materialization.
Thumbnails are stored as separate blob content with metadata persisted in a new table,
keyed by representation_id. This enables future list-entry queries to prefer thumbnails.

在图片 representation 的 blob 物化完成后生成缩略图。缩略图作为独立 blob
存储，并在新表中记录元数据，主键为 representation_id，为后续列表展示
提供缩略图优先能力。

## Goals / 目标

- Generate thumbnails for image/\* representations after BlobReady.
- Store thumbnail bytes in blob storage (deduplicated).
- Persist thumbnail metadata in a dedicated table keyed by representation_id.
- Maintain hexagonal boundaries: all external capabilities via ports.
- Failures do not block the main blob materialization path.

## Non-Goals / 非目标

- No UI or API exposure in this phase.
- No thumbnail generation for inline-only representations.
- No failure-state persistence for thumbnails (success-only records).

## Architecture / 架构

Location: `BackgroundBlobWorker` (uc-infra) after successful blob materialization.

New ports in uc-core:

- `ThumbnailRepositoryPort`: read/write metadata by representation_id.
- `ThumbnailGeneratorPort`: decode, resize, encode (image/\* -> webp).

New infra implementations:

- Diesel repository for thumbnail metadata.
- Pure Rust thumbnail generator (image + webp encoder).

## Data Model / 数据模型

New table: `clipboard_representation_thumbnail`

Fields:

- representation_id (PK, string)
- thumbnail_blob_id (string)
- thumbnail_mime_type (string, fixed `image/webp`)
- width (int, original image width)
- height (int, original image height)
- size_bytes (int, original image size in bytes)
- created_at_ms (int, optional, current time)

Note: width/height/size_bytes are from the original image, per requirement.

## Data Flow / 数据流

1. BackgroundBlobWorker processes staged representation bytes.
2. Blob materialization succeeds and representation becomes BlobReady.
3. If mime_type matches image/\*, check ThumbnailRepositoryPort.exists(rep_id).
4. If exists -> skip (idempotent).
5. Else load bytes (cache -> spool), call ThumbnailGeneratorPort:
   - output: webp bytes + original width/height + original size_bytes
6. Write thumbnail bytes to blob storage via BlobWriterPort::write_if_absent.
7. Persist metadata in thumbnail table.

## Error Handling / 错误处理

- Thumbnail generation and write are best-effort.
- On failure: log error with rep_id and reason, do not change representation state.
- No failure record stored in thumbnail table.

## Testing / 测试

- Worker unit test: image/\* rep triggers thumbnail generation and metadata insert.
- Worker unit test: generator failure logs error and does not insert metadata.
- Thumbnail repo tests: insert + get by representation_id.
- Thumbnail generator tests: resize to longest edge 128px and output webp.

## Migration / 迁移

- Add new table and Diesel schema entry.
- Add repository wiring in uc-infra and bootstrap wiring in uc-tauri.

## Open Questions / 待确认

- Exact `created_at_ms` semantics (if needed now or later).
- Which image decoding crate/version to use (pure Rust only).
