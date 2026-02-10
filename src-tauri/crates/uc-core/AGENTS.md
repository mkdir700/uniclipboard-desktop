# UC-CORE

Follow parent rules in `AGENTS.md`. This crate is pure domain + ports.

## OVERVIEW

`uc-core` defines domain models, value objects, protocol state machines, and Port traits. No concrete IO/system dependencies.

## WHERE TO LOOK

- Port contracts: `crates/uc-core/src/ports/`.
- Clipboard domain model/policy: `crates/uc-core/src/clipboard/`.
- Pairing protocol/domain FSM: `crates/uc-core/src/network/`.
- Security domain model: `crates/uc-core/src/security/`.

## CONVENTIONS

- Add external capability as a trait in `src/ports/` first.
- Keep DTO/config modules policy-free (pure data).
- Preserve deterministic behavior for state machines (no side effects).
- If changing protocol/state types, update dependent uc-app tests.

## ANTI-PATTERNS

- Importing Diesel/Tauri/libp2p/system APIs in `uc-core`.
- Embedding adapter-specific behavior in port traits.
- Mixing domain model changes with infra implementation in one commit.
- `unwrap()/expect()` in production domain code paths.

## HIGH-RISK FILES

- `crates/uc-core/src/network/pairing_state_machine.rs`
- `crates/uc-core/src/ports/` (boundary-breaking risk)

## COMMANDS

```bash
# from src-tauri/
cargo check -p uc-core
cargo test -p uc-core
```
