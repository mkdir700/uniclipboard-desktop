# Snapshot Cache Alignment Design

- Created: 2026-01-18
- Status: Design Update (Alignment with 2026-01-18 snapshot-cache background worker design)
- Scope: Cache/Spool/Worker pipeline hardening and full design compliance
- Non-Goals: Changing capture semantics or introducing new storage backends

## Context

The snapshot cache pipeline is implemented but deviates from the converged design in several
places (silent queue drops, unbounded spool growth, incorrect Lost transitions, and spool
placement under vault). This design aligns the implementation with the agreed architecture
and fixes root causes rather than patching symptoms.

## Goals

- Enforce three-layer availability: in-memory cache -> disk spool -> blob store.
- Ensure capture path is non-blocking and never performs disk IO.
- Make failure modes observable (no silent drops).
- Guarantee bounded resource usage (cache and spool).
- Ensure Lost transitions are correct and delayed, not premature.
- Place spool under OS cache directory, not vault.
- Preserve hexagonal boundaries: changes to ports live in core, implementations in infra.

## Non-Goals

- Changing clipboard event schema beyond required fields.
- Introducing new crypto or storage formats.
- Refactoring the UI or front-end behavior.

## Design Summary

1. Extend AppDirs/AppPaths to include a cache root directory.
2. Move spool_dir to OS cache root (app_cache_root/spool).
3. Expand ClipboardStorageConfig for cache/spool sizing and worker retry.
4. Upgrade RepresentationCache to track entry status and implement priority eviction.
5. Add bounded SpoolManager with TTL and size-based eviction.
6. Split update_processing_result into a typed outcome to distinguish mismatch vs error.
7. Update BackgroundBlobWorker to avoid premature Lost and to handle outcomes correctly.
8. Add SpoolJanitor for TTL-based Lost marking and cleanup.

## Component Changes

### AppDirs / AppPaths

- AppDirs: add `app_cache_root` derived from `dirs::cache_dir()`.
- AppPaths: add `cache_dir`.
- Wiring: derive `spool_dir = cache_dir.join("spool")`.

Migration: on startup, if an old spool directory exists under vault, either migrate files
into the new spool dir or scan both locations for one release cycle.

### ClipboardStorageConfig

Add fields:

- cache_max_entries, cache_max_bytes
- spool_max_bytes, spool_ttl_days
- worker_retry_max_attempts, worker_retry_backoff_ms

Wiring should instantiate RepresentationCache and SpoolManager using these values.

### RepresentationCache

- Track CacheEntryStatus (Pending, Spooling, Completed).
- Provide: put, get, mark_spooled, mark_completed, remove.
- Eviction: prefer oldest Completed; if none, evict oldest Pending.
- No await while holding mutex.

### SpoolManager

- Track current size and file index (by mtime).
- On write: enforce max_bytes via eviction; return stats.
- Add cleanup_expired(now, ttl_days) to prune expired files.
- Strong permissions remain (0700 dir, 0600 files on Unix).

### SpoolerTask

- After successful spool write: mark_spooled in cache, then try_send worker.
- On try_send failure: warn with queue capacity and rep_id (no silent drop).

### BackgroundBlobWorker

- Use typed update outcome:
  - StateMismatch -> treat as Completed without error.
  - NotFound/DbError -> log error and retry.
- On cache/spool miss: do NOT mark Lost immediately.
  - Transition back to Staged (if Processing) and return MissingBytes.
- Lost is reserved for TTL expiration or explicit policy.

### SpoolJanitor (new)

- Periodic task scans spool dir and removes expired files.
- For expired entries still in Staged/Processing, mark Lost with last_error and delete file.
- Emits metrics and warnings on failures.

### Payload Resolver

- Staged/Processing/Failed: read cache/spool, re-queue worker, return inline bytes.
- On miss: return error without changing state; logging is required.

## Error Handling and Observability

- Capture: warn on spool queue drops.
- Spooler: error on write failures; warn on worker enqueue failures.
- Worker: distinguish state mismatch from infrastructure errors.
- Janitor: warn on cleanup failures; metrics for evictions and Lost transitions.

## Testing Strategy

- Unit: cache eviction priority, spool eviction/TTL, update_processing_result outcomes.
- Integration: spooler writes and worker notifications; worker cache->spool fallback;
  resolver re-queue behavior.
- Recovery: spool scan on restart, old spool migration path.

## Rollout

- Implement behind config defaults to preserve existing behavior while enabling safe limits.
- Run integration tests and stress tests to validate no data loss under backpressure.
