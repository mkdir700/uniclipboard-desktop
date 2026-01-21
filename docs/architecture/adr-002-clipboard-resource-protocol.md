# ADR-002: Clipboard Resource Protocol for Detail Content

## Context

Clipboard entries are displayed in a list with preview data, while full content can be large
and expensive to load. Returning full content from the detail API couples semantic preview
and raw bytes, and requires the backend to read blobs for every detail fetch. We need a
clear separation between preview and raw content, while keeping a consistent access path
for different content types (text, image, etc.).

## Decision

Introduce a resource metadata API and a custom protocol for raw bytes:

- The detail API returns resource metadata via `get_clipboard_entry_resource`, including
  `blob_id`, `mime_type`, `size_bytes`, and `url`.
- Raw bytes are fetched through `uc://blob/{blob_id}`. The protocol handler returns
  only bytes and MIME type.
- The list API continues to return preview content only. The frontend expands by
  calling `get_clipboard_entry_resource` and then `fetch(resource.url)` to load bytes.

Inline-only content that has no `blob_id` remains unsupported by the resource protocol.
Errors are logged and surfaced to upper layers; no silent failures.

## Consequences

- Preview and raw content are strictly separated, reducing blob reads for list views.
- The frontend has a single, consistent path to load full content across content types.
- Inline-only entries cannot be fetched via `uc://` without additional support.
- Access control is not implemented at this stage; future work may add authorization
  or short-lived tokens for protocol access.
