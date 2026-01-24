## Summary / 摘要

Move clipboard change origin semantics into uc-core and consolidate capture
policy decisions in uc-app. This removes restore-context logic from uc-tauri and
creates a single, extensible path for local capture, local restore, and remote
push sources.

将剪贴板变化的“来源语义”上移到 uc-core，并将捕获策略统一放在 uc-app。
这会移除 uc-tauri 中的 restore_context 逻辑，使本地捕获、本地恢复、
远端推送都走同一条可扩展路径。

## Goals / 目标

- Define explicit clipboard change origins in uc-core.
- Centralize capture policy decisions in uc-app.
- Remove restore_context and any dedup/echo-cancel policy from uc-tauri.
- Enable future remote push and cloud sync sources without new adapter logic.

## Non-Goals / 非目标

- No implementation of remote sync or cloud transport.
- No change to multi-representation restore behavior.
- No UI changes.

## Architecture / 架构

### Domain model (uc-core)

- Add `ClipboardChangeOrigin` enum (e.g. `LocalCapture`, `LocalRestore`, `RemotePush`).
- Optionally add `ClipboardChange` struct wrapping `SystemClipboardSnapshot` + origin.

### Application layer (uc-app)

- Extend capture entrypoint to accept origin:
  - `CaptureClipboardUseCase::execute(snapshot, origin)` or
  - `HandleClipboardChangeUseCase::execute(ClipboardChange)`.
- Policy decisions live here:
  - `LocalRestore` -> skip capture to avoid echo.
  - `RemotePush` -> capture with rules for sync propagation (future work).

### Adapter (uc-tauri)

- Adapter only labels origin; no policy decisions.
- Remove `restore_context` from `AppRuntime`.
- Provide origin for the next clipboard change via a port (see below).

### Origin tracking port (uc-core)

Add a port to decouple origin tracking from adapters:

- `ClipboardChangeOriginPort`:
  - `set_next_origin(origin, ttl)`
  - `consume_origin_or_default(default_origin)`

This allows uc-tauri to label the next change (e.g. after restore) without
containing policy logic. The use case consumes the origin and decides behavior.

## Data Flow / 数据流

1. Platform watcher detects clipboard change.
2. uc-tauri calls use case entrypoint with:
   - `origin = origin_port.consume_origin_or_default(LocalCapture)`
3. uc-app capture use case evaluates origin:
   - `LocalCapture` -> capture and persist.
   - `LocalRestore` -> skip capture.
   - `RemotePush` -> capture with sync-safe policy.

For restore:

1. Restore command sets `origin_port.set_next_origin(LocalRestore, ttl)`.
2. Clipboard change event fires and is handled as above.
3. No capture occurs, avoiding echo and dedup in adapter layer.

## Error Handling / 错误处理

- Unknown origin defaults to `LocalCapture` with a warning log.
- Origin tracking expiration returns default origin to avoid blocking capture.
- Capture policy errors remain in uc-app; adapters only log and propagate.

## Testing / 测试

- Unit tests for capture use case:
  - `LocalRestore` -> skip capture (no repo writes).
  - `LocalCapture` -> capture and persist.
  - `RemotePush` -> capture path is invoked (policy stub for now).
- Unit tests for origin port implementation:
  - `set_next_origin` then `consume` returns origin once.
  - Expired origin returns default.

## Migration / 迁移

1. Add `ClipboardChangeOrigin` (and optional `ClipboardChange`) in uc-core.
2. Add `ClipboardChangeOriginPort` in uc-core.
3. Implement an in-memory origin port in uc-infra (or uc-app state) and wire it
   in uc-tauri bootstrap.
4. Update capture use case signature to accept origin; keep old signature for
   compatibility during migration.
5. Update uc-tauri clipboard change handler to consume origin from the port and
   call the new use case entrypoint.
6. Update restore command to set `LocalRestore` origin via the port.
7. Remove `restore_context` from `AppRuntime`.
