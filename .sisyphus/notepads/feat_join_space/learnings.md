## Pairing UI State Machine

- The pairing flow involves two distinct event streams: `p2p-pairing-verification` (handshake) and `space-access-completed` (data sync/access).
- The UI must wait for `space-access-completed` before showing success, even if the P2P handshake is complete.
- `p2p-pairing-verification: complete` is an intermediate state, not the final success state for the UI.
- Cancellation during the flow requires explicit rejection via `rejectP2PPairing` to clean up the backend state.
