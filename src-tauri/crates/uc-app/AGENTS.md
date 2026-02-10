# UC-APP

Follow parent rules in `AGENTS.md`. This crate orchestrates use cases over `uc-core` ports.

## OVERVIEW

`uc-app` coordinates workflows (setup, pairing, space access, clipboard) and composes domain operations into application actions.

## WHERE TO LOOK

- Usecase entrypoints: `crates/uc-app/src/usecases/mod.rs`.
- Setup orchestration: `crates/uc-app/src/usecases/setup/orchestrator.rs`.
- Pairing orchestration: `crates/uc-app/src/usecases/pairing/orchestrator.rs`.
- Space access flow: `crates/uc-app/src/usecases/space_access/orchestrator.rs`.
- Dependency manifest: `crates/uc-app/src/deps.rs`.

## CONVENTIONS

- Depend on traits from `uc-core::ports`; no direct infra/platform concrete coupling.
- Keep orchestrators explicit about state transitions and side effects.
- For new usecase: add file -> export in module -> wire accessor in `uc-tauri` runtime.
- Use structured tracing around long async flows.

## ANTI-PATTERNS

- Business flow leaked into tauri command layer.
- Orchestrator mutating persistence/network without port abstraction.
- Refactoring unrelated usecases during bugfix.

## HIGH-RISK FILES

- `crates/uc-app/src/usecases/setup/orchestrator.rs`
- `crates/uc-app/src/usecases/pairing/orchestrator.rs`
- `crates/uc-app/src/usecases/space_access/orchestrator.rs`

## COMMANDS

```bash
# from src-tauri/
cargo check -p uc-app
cargo test -p uc-app
```
