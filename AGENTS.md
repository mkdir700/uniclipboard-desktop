# AGENTS.md

## Engineering Principles

- **Fix root causes, not symptoms.** No patchy or workaround-driven solutions.
- **Do not “fix feelings”, fix structure.** Repeated workarounds indicate architectural flaws.
- **Short-term compromises must be reversible.**
- **Never break boundaries.** If something must be deferred, leave an explicit `TODO`.

## Hexagonal Architecture Boundaries (Strict)

- **Layering is fixed:**
  - `uc-app → uc-core ← uc-infra / uc-platform`

- **Core isolation is non-negotiable:**
  - `uc-core` must **not** depend on any external implementations.

- **All external capabilities go through Ports (no exceptions):**
  - DB, FS, Clipboard, Network, Crypto

## Rust Error Handling (Production Code)

- **No `unwrap()` / `expect()` in production code.**
  - **Tests are the only exception.**

- **No silent failures in async or event-driven code.**
  - Errors must be **logged** and **observable** by upper layers.

## Tauri Command Tracing (Required)

- **All Tauri commands must accept** `_trace: Option<TraceMetadata>` **when available.**
- Each command must:
  - Create an `info_span!` with **`trace_id`** and **`trace_ts`** fields
  - Call `record_trace_fields(&span, &_trace)`
  - `.instrument(span)` the async body

## Rust Logging (tracing) — Required Best Practices

- **Use `tracing` for all logging.** Do not use `println!`, `eprintln!`, or `log` macros in production code.
- **Prefer structured fields over string formatting.**
  - ✅ `info!(peer_id = %peer_id, attempt, "dial started");`
  - ❌ `info!("dial started: peer_id={}, attempt={}", peer_id, attempt);`

- **Use spans to model request/task lifetimes.** Attach contextual fields once, log events inside.
- **Record errors with context, not silence.**
  - Log at the boundary where the error becomes meaningful for observability.
  - Propagate errors upward after logging unless explicitly handled.

- **Use appropriate levels consistently:**
  - `error!`: user-visible failure / operation failed
  - `warn!`: unexpected but recovered / degraded behavior
  - `info!`: major lifecycle events / state transitions
  - `debug!`: detailed flow useful for debugging
  - `trace!`: very noisy internal steps

- **Avoid logging secrets.**
  - Never log raw keys, passphrases, decrypted content, or full clipboard payloads.
  - If needed, log sizes, hashes, or redacted markers.

### Best-practice Example (structured + span + error context)

```rust
use tracing::{info, warn, error, debug, info_span, Instrument};

pub async fn sync_peer(peer_id: &str, attempt: u32) -> Result<(), SyncError> {
    // Attach stable context to a span once.
    let span = info_span!(
        "sync_peer",
        peer_id = %peer_id,
        attempt = attempt
    );

    async move {
        info!("start");

        // Prefer explicit error handling; log with context at the right layer.
        let session = match open_session(peer_id).await {
            Ok(s) => s,
            Err(e) => {
                warn!(error = %e, "open_session failed; will retry if possible");
                return Err(SyncError::OpenSession(e));
            }
        };

        debug!(session_id = %session.id(), "session opened");

        if let Err(e) = push_updates(&session).await {
            // error logging should preserve the causal chain where possible
            error!(error = %e, "push_updates failed");
            return Err(SyncError::PushUpdates(e));
        }

        info!("done");
        Ok(())
    }
    .instrument(span)
    .await
}
```

### Example: recording `_trace` fields into an existing span (Tauri-compatible)

```rust
use tracing::{info_span, Instrument};

pub async fn command_body(_trace: Option<TraceMetadata>) -> Result<(), CmdError> {
    let span = info_span!(
        "cmd.do_something",
        trace_id = tracing::field::Empty,
        trace_ts = tracing::field::Empty
    );
    record_trace_fields(&span, &_trace);

    async move {
        // logs inside automatically inherit trace fields via the span
        tracing::info!(op = "do_something", "start");
        // ...
        Ok(())
    }
    .instrument(span)
    .await
}
```

## Tauri State Lifecycle (Required)

- Any type accessed via `tauri::State<T>` must be registered **before startup** with `.manage()`

## Frontend Layout Rules

- **No fixed-pixel layouts.**
  - Use **Tailwind utilities** or **rem** units.

## Cargo Command Location (CRITICAL)

- **All Rust-related commands** (`cargo build`, `cargo test`, `cargo check`, etc.) **must be executed from `src-tauri/`.**
- **Never run Cargo commands from the project root.**
- If `Cargo.toml` is **not present** in the current directory:
  - **Stop immediately and do not retry.**

## Rustdoc Bilingual Documentation Guide

### Recommended Approach: Structured Bilingual Side-by-Side

**Applicable scenarios**

- Long-term maintenance projects
- Need complete `cargo doc` output
- API / core / public interface documentation

**Example**

```rust
/// Load or create a local device identity.
///
/// 加载或创建本地设备标识。
///
/// # Behavior / 行为
/// - If an ID exists on disk, it will be loaded.
/// - Otherwise, a new ID will be generated and persisted.
///
/// - 如果磁盘上已有 ID，则直接加载。
/// - 否则生成新的 ID 并持久化保存。
pub fn load_or_create() -> Result<Self> {
    // ...
}
```

**Advantages**

- Fully supported by Rustdoc
- English-first for external ecosystem conventions; Chinese as internal supplement
- Minimal cost to remove either language later

**Best practices**

- English first, Chinese second
- Use subheadings to differentiate sections (e.g., `# Behavior / 行为`)
