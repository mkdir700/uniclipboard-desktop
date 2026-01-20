# ADR-001: Snapshot Cache Pipeline Alignment

## Context

Large clipboard payloads lost original bytes because placeholders were stored
without preserving snapshot data, and the runtime pipeline diverged from the
intended design. The capture path must remain non-blocking (no disk I/O), while
payloads must survive restarts and eventually reach the blob store. Prior
implementation also suffered from silent queue drops, unbounded spool growth,
premature Lost transitions, and spool placement under the vault directory.

## Decision

- Adopt a three-layer availability model: in-memory cache -> disk spool -> blob store.
- Capture path only writes metadata + cache.put and uses try_send for spool/worker;
  it never awaits and never performs disk I/O.
- SpoolerTask is the only component that writes to the disk spool, and the spool
  directory lives under the OS cache root with strong permissions.
- BackgroundBlobWorker is the only blob writer and uses an atomic
  BlobWriterPort::write_if_absent for idempotent writes.
- PayloadAvailability is state-only; inline_data and blob_id are the sole data
  carriers with explicit invariants.
- Repository updates use CAS-style transactional updates (update_processing_result)
  so blob_id and payload_state advance atomically; state mismatches are non-errors.
- Resolver is read-only: for Staged/Processing/Failed it returns cache/spool bytes
  and re-queues the worker; it never writes blobs.
- Lost is only set after TTL/cleanup or explicit policy, not on transient cache/spool
  misses.
- A SpoolJanitor enforces TTL cleanup and marks Lost for expired entries.
- Cache and spool are bounded with eviction and configured limits/backoff.
- Port traits live in core; infra/platform only provide implementations.

## Consequences

- AppDirs/AppPaths gain a cache root and cache_dir; spool moves under cache_dir
  and requires a one-release migration scan.
- More startup wiring is required (spooler, worker, janitor), and background task
  failures must be observable to upper layers.
- Backpressure behavior is explicit: queue full degrades and logs; queue closed
  is treated as an error.

## Follow-ups

- Evaluate lightweight spool encryption or secure delete options if the threat
  model expands beyond same-user processes.
- Move cache/spool/worker limits from hardcoded defaults into user-facing config.
