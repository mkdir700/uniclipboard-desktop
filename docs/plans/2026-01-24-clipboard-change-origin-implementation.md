# Clipboard Change Origin Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Move clipboard change origin semantics into uc-core and route capture policy decisions through uc-app, removing restore-context logic from uc-tauri.

**Architecture:** Introduce a uc-core origin model and a new port for origin tracking. uc-app capture use case consumes origin to decide whether to capture. uc-tauri sets origin after restore and passes origin through to use cases without policy logic.

**Tech Stack:** Rust, Tauri, uc-core ports, uc-app use cases, uc-infra implementations.

---

### Task 1: Add origin model in uc-core

**Files:**

- Create: `src-tauri/crates/uc-core/src/clipboard/change.rs`
- Modify: `src-tauri/crates/uc-core/src/clipboard/mod.rs`

**Step 1: Write the failing test**

Create a unit test in `change.rs` verifying enum variants and default behavior (if any).

```rust
#[test]
fn clipboard_change_origin_variants() {
    let origin = ClipboardChangeOrigin::LocalCapture;
    assert_eq!(origin, ClipboardChangeOrigin::LocalCapture);
}
```

**Step 2: Run test to verify it fails**

Run (from `src-tauri/`):

```bash
cargo test -p uc-core clipboard_change_origin_variants
```

Expected: FAIL (type not found).

**Step 3: Write minimal implementation**

Define `ClipboardChangeOrigin` enum (LocalCapture, LocalRestore, RemotePush) and optional
`ClipboardChange` struct containing `SystemClipboardSnapshot` + origin.

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test -p uc-core clipboard_change_origin_variants
```

Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-core/src/clipboard/change.rs src-tauri/crates/uc-core/src/clipboard/mod.rs
git commit -m "feat: add clipboard change origin model"
```

### Task 2: Add origin tracking port in uc-core

**Files:**

- Create: `src-tauri/crates/uc-core/src/ports/clipboard/clipboard_change_origin.rs`
- Modify: `src-tauri/crates/uc-core/src/ports/clipboard/mod.rs`

**Step 1: Write the failing test**

Add a minimal mock implementation test in the new port module to prove the trait
shape compiles and is callable.

```rust
struct MockOriginPort;

#[async_trait::async_trait]
impl ClipboardChangeOriginPort for MockOriginPort {
    async fn set_next_origin(&self, _origin: ClipboardChangeOrigin, _ttl: std::time::Duration) {}
    async fn consume_origin_or_default(
        &self,
        default_origin: ClipboardChangeOrigin,
    ) -> ClipboardChangeOrigin {
        default_origin
    }
}

#[tokio::test]
async fn origin_port_returns_default() {
    let port = MockOriginPort;
    let origin = port
        .consume_origin_or_default(ClipboardChangeOrigin::LocalCapture)
        .await;
    assert_eq!(origin, ClipboardChangeOrigin::LocalCapture);
}
```

**Step 2: Run test to verify it fails**

Run (from `src-tauri/`):

```bash
cargo test -p uc-core origin_port_returns_default
```

Expected: FAIL (trait not found).

**Step 3: Write minimal implementation**

Define `ClipboardChangeOriginPort` trait with:

- `set_next_origin(origin, ttl)`
- `consume_origin_or_default(default_origin)`

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test -p uc-core origin_port_returns_default
```

Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-core/src/ports/clipboard/clipboard_change_origin.rs src-tauri/crates/uc-core/src/ports/clipboard/mod.rs
git commit -m "feat: add clipboard change origin port"
```

### Task 3: Implement origin port in uc-infra

**Files:**

- Create: `src-tauri/crates/uc-infra/src/clipboard/change_origin.rs`
- Modify: `src-tauri/crates/uc-infra/src/clipboard/mod.rs`

**Step 1: Write the failing test**

Add unit tests for the in-memory implementation:

```rust
#[tokio::test]
async fn origin_is_consumed_once() {
    let port = InMemoryClipboardChangeOrigin::new();
    port.set_next_origin(ClipboardChangeOrigin::LocalRestore, Duration::from_secs(1)).await;
    let first = port.consume_origin_or_default(ClipboardChangeOrigin::LocalCapture).await;
    let second = port.consume_origin_or_default(ClipboardChangeOrigin::LocalCapture).await;
    assert_eq!(first, ClipboardChangeOrigin::LocalRestore);
    assert_eq!(second, ClipboardChangeOrigin::LocalCapture);
}
```

**Step 2: Run test to verify it fails**

Run (from `src-tauri/`):

```bash
cargo test -p uc-infra origin_is_consumed_once
```

Expected: FAIL (type not found).

**Step 3: Write minimal implementation**

Implement a mutex-protected struct storing `{ origin, expires_at }` with TTL behavior.

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test -p uc-infra origin_is_consumed_once
```

Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-infra/src/clipboard/change_origin.rs src-tauri/crates/uc-infra/src/clipboard/mod.rs
git commit -m "feat: add in-memory clipboard change origin port"
```

### Task 4: Extend capture use case to accept origin

**Files:**

- Modify: `src-tauri/crates/uc-app/src/usecases/internal/capture_clipboard.rs`

**Step 1: Write the failing test**

Add a unit test in `capture_clipboard.rs` that passes `LocalRestore` origin and
asserts no persistence calls occur (use mocks with counters).

**Step 2: Run test to verify it fails**

Run (from `src-tauri/`):

```bash
cargo test -p uc-app capture_skips_local_restore
```

Expected: FAIL (new signature / behavior missing).

**Step 3: Write minimal implementation**

Add an overload or new method, e.g. `execute_with_origin(snapshot, origin)` and
have `execute(snapshot)` call it with `LocalCapture`. In `execute_with_origin`,
return early for `LocalRestore`.

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test -p uc-app capture_skips_local_restore
```

Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/internal/capture_clipboard.rs
git commit -m "feat: route clipboard capture by origin"
```

### Task 5: Wire origin port and use origin in uc-tauri

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`
- Modify: `src-tauri/crates/uc-tauri/src/commands/clipboard.rs`

**Step 1: Write the failing test**

Add a unit test for runtime origin consumption in `runtime.rs` (if no tests exist,
add a small test module that instantiates the in-memory port and validates origin
is passed to the capture use case).

**Step 2: Run test to verify it fails**

Run (from `src-tauri/`):

```bash
cargo test -p uc-tauri runtime_consumes_origin
```

Expected: FAIL (origin port not wired).

**Step 3: Write minimal implementation**

- Add `origin_port` to `AppDeps` and wire it in `bootstrap/wiring.rs`.
- In `on_clipboard_changed`, read origin via port and call the new use case method.
- In `restore_clipboard_entry`, call `origin_port.set_next_origin(LocalRestore, ttl)`.
- Remove `restore_context` and related methods from `AppRuntime`.

**Step 4: Run test to verify it passes**

Run:

```bash
cargo test -p uc-tauri runtime_consumes_origin
```

Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs src-tauri/crates/uc-tauri/src/commands/clipboard.rs
git commit -m "feat: use clipboard change origin in runtime"
```

### Task 6: Clean up and run focused tests

**Files:**

- Check: modified files above

**Step 1: Run diagnostics**

Run `lsp_diagnostics` on modified Rust files.

**Step 2: Run focused tests**

Run (from `src-tauri/`):

```bash
cargo test -p uc-core clipboard_change_origin_variants
cargo test -p uc-core origin_port_returns_default
cargo test -p uc-infra origin_is_consumed_once
cargo test -p uc-app capture_skips_local_restore
cargo test -p uc-tauri runtime_consumes_origin
```

Expected: PASS.
