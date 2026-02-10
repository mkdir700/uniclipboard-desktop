# UC-INFRA

Follow parent rules in `AGENTS.md`. This crate implements persistence/security/file/time adapters.

## OVERVIEW

`uc-infra` contains concrete implementations for `uc-core` ports using Diesel/SQLite, filesystem storage, encryption services, and timers.

## WHERE TO LOOK

- DB repositories: `crates/uc-infra/src/db/repositories/`.
- DB models/mappers: `crates/uc-infra/src/db/models/`, `crates/uc-infra/src/db/mappers/`.
- Security adapters: `crates/uc-infra/src/security/`.
- Clipboard background workers/queues: `crates/uc-infra/src/clipboard/`.
- Active migrations: `crates/uc-infra/migrations/`.

## CONVENTIONS

- Implement trait contracts exactly as defined in `uc-core::ports`.
- Keep repository mapper logic deterministic and side-effect minimal.
- Migrations for active schema live under `crates/uc-infra/migrations/`.
- Log errors with context via `tracing`; never silently swallow failures.

## ANTI-PATTERNS

- Introducing domain policy decisions in repository/adapters.
- Editing legacy `migrations/` for new schema work.
- Returning partially valid data on crypto/decryption failures.
- `expect/unwrap` in production adapter paths.

## HIGH-RISK FILES

- `crates/uc-infra/src/clipboard/background_blob_worker.rs`
- `crates/uc-infra/src/security/decrypting_representation_repo.rs`
- `crates/uc-infra/src/db/repositories/`

## COMMANDS

```bash
# from src-tauri/
cargo check -p uc-infra
cargo test -p uc-infra
```
