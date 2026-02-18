# UC-TAURI

Follow parent rules in `AGENTS.md`. This crate is Tauri-facing adapter/wiring layer.

## OVERVIEW

`uc-tauri` bridges frontend Tauri commands/events with app-layer usecases, and owns bootstrap/runtime wiring that composes all dependencies.

## WHERE TO LOOK

- Bootstrap composition: `crates/uc-tauri/src/bootstrap/wiring.rs`.
- Runtime accessor host: `crates/uc-tauri/src/bootstrap/runtime.rs`.
- Command handlers: `crates/uc-tauri/src/commands/`.
- Event emitters/payloads: `crates/uc-tauri/src/events/`.
- Tauri-specific adapters: `crates/uc-tauri/src/adapters/`.

## CONVENTIONS

- Command handlers call `runtime.usecases().*`; avoid direct `runtime.deps` access.
- In `commands/`, any IO/business flow (clipboard/network/encryption/settings) must be encapsulated behind usecase accessors.
- Exception: direct `runtime.deps` reads are allowed only for pure, side-effect-free status checks and must include a short reason comment.
- Command spans: include trace fields when `_trace: Option<TraceMetadata>` is available.
- Event payload structs emitted via `app.emit()` must use camelCase serde rename.
- Bootstrap file edits should be minimal and localized.

## ANTI-PATTERNS

- Adding business rules in command handlers.
- Emitting snake_case payloads to frontend listeners.
- Bypassing port wiring with ad-hoc concrete object creation.
- Broad refactors in `bootstrap/wiring.rs` during bugfixes.

## COMMAND LAYER REVIEW CHECKLIST

1. No direct `runtime.deps.*` business IO in `commands/*`.
2. New command paths invoke `runtime.usecases()` for workflow execution.
3. If a usecase constructor signature changes, runtime accessor wiring is updated in the same diff.
4. Verification includes `cargo check -p uc-tauri`.

## HIGH-RISK FILES

- `crates/uc-tauri/src/bootstrap/wiring.rs`
- `crates/uc-tauri/src/bootstrap/runtime.rs`
- `crates/uc-tauri/src/commands/encryption.rs`

## COMMANDS

```bash
# from src-tauri/
cargo check -p uc-tauri
cargo test -p uc-tauri
```
