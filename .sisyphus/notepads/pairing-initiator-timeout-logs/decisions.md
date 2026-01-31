# Decisions - Pairing Initiator Timeout Logs

- Decided to match on `PairingMessage` in `wiring.rs` to extract `message_kind` instead of modifying `uc-core` to add a `kind()` method, to minimize changes outside the target crate.
- Fixed the test error in `wiring.rs` to ensure `cargo check --tests` passes, which is part of the verification process.
