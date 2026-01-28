# Pairing Handshake (Security Explainable) Design

## Goals

- Complete the "security explainable" pairing handshake in the new architecture.
- Keep the current frontend working by emitting legacy event names, while marking them deprecated.
- Persist Trusted pairing state for both sides and allow business protocols only after Trusted.

## Non-Goals

- Do not modify legacy pairing implementation.
- Do not redesign the UI or introduce new frontend event names in this phase.
- Do not introduce per-device E2EE beyond existing identity public keys.

## Scope

- uc-core: complete PairingStateMachine transitions and actions.
- uc-app: execute pairing actions, timers, persistence, and UI bridging.
- uc-tauri: expose pairing commands and emit legacy events with deprecation markers.
- uc-platform: rely on existing ConnectionPolicy gating after Trusted.

## Event Compatibility and Deprecation

New architecture emits legacy event names to keep current UI working:

- p2p-pairing-request
- p2p-pin-ready
- p2p-pairing-complete
- p2p-pairing-failed

All emitted payloads include:

- deprecated: true
- deprecated_reason: "legacy event name; will be replaced by p2p-pairing-verification"

The backend logs warnings whenever these events are emitted to highlight the deprecation path.

## Backend Flow and State Machine Closure

The pairing loop is closed by completing state transitions and action production in
PairingStateMachine, with PairingOrchestrator executing the side effects.

Responder flow:

1. RecvRequest -> WaitingForRequest, generate Challenge (device_id, identity_pubkey, nonce, pin).
2. UserAccept -> WaitingForResponse, produce ShowVerification (short_code, fingerprints).
3. RecvResponse -> verify pin_hash, send Confirm, produce PersistPairedDevice.

Initiator flow:

1. RecvChallenge -> WaitingUserVerification, produce ShowVerification.
2. UserAccept -> ResponseSent, send Response.
3. RecvConfirm -> PersistingTrust, produce PersistPairedDevice.

Persistence flow:

1. PersistPairedDevice -> repo upsert
2. PersistOk -> Paired, EmitResult(success=true)
3. PersistErr -> Failed, EmitResult(success=false)

Short code and fingerprints are computed with IdentityFingerprint and ShortCodeGenerator using
identity_pubkey + nonce from both peers to guarantee the same result on both devices.

## UI and Command Bridging

The new architecture bridges to the UI only via uc-tauri. Command surfaces keep the existing
API shape (initiate_p2p_pairing / accept_p2p_pairing / reject_p2p_pairing / verify_p2p_pairing_pin)
but internally route to the new orchestrator and state machine.

Event mapping:

- ShowVerification -> p2p-pin-ready
- RecvRequest -> p2p-pairing-request
- EmitResult(success=true) -> p2p-pairing-complete
- EmitResult(success=false) -> p2p-pairing-failed

## Trusted Persistence and Gating

PairedDeviceRepository is the authority for Trusted status. Each record includes:

- peer_id
- identity_fingerprint
- pairing_state = Trusted
- paired_at
- last_seen_at

Business protocol gating uses ConnectionPolicy::allowed_protocols(Trusted). Protocols are denied
until Trusted and allowed after persistence. This serves as a hard acceptance test.

## Error Handling and Timeouts

- UserReject leads to Cancelled and EmitResult(false).
- Timeout for WaitingChallenge / WaitingUserVerification / WaitingForResponse / WaitingForConfirm
  triggers failure and EmitResult(false).
- Session mismatch or transport errors trigger failure and error logs.
- PIN verification failure sends Confirm(success=false) and ends the session.

Timers are managed in the orchestrator (StartTimer / CancelTimer) and must be cancelled
when sessions complete to avoid late failures.

## Testing and Acceptance

Unit tests (uc-core):

- Initiator transitions to Paired via RecvConfirm + PersistOk.
- Responder transitions to Paired via RecvResponse + PersistOk.
- Short code / fingerprint consistency for same transcript inputs.
- Failure paths: timeout, reject, pin mismatch.

Integration tests (uc-app / uc-tauri):

- ShowVerification emits legacy event with deprecated markers.
- EmitResult produces pairing-complete/failed events.
- PersistPairedDevice writes Trusted state in repo.

Manual acceptance:

1. Initiate pairing from A, B confirms short code/fingerprint.
2. Both sides persist Trusted.
3. Restart both sides; no re-confirmation required.
4. Business protocol denied before Trusted, allowed after Trusted.

## Rollout Notes

- Legacy event names are deprecated but kept for compatibility.
- Migration to new event names is deferred to a follow-up plan.
