- **Event Listener State Access**: When using `tauri/event` listeners in `useEffect`, the closure captures the initial state. To access updated state (like `activeSessionId`) without re-subscribing (which can miss events), use a `useRef` to track the current value.
- **Sonner Toast Actions**: `sonner` toasts allow `action` and `cancel` callbacks which are closures. They can access the scope where `toast()` was called.

## Work Completed - 2026-02-08

All 7 tasks completed successfully:

1. **Event Contracts** (`fa97fdd8`): Added `SetupStateChangedEvent` and `SpaceAccessCompletedEvent` types with `onSetupStateChanged()` and `onSpaceAccessCompleted()` listeners.

2. **Backend Event Emission** (`impl(setup): emit setup-state-changed`): Added `SetupEventPort` in uc-core, implemented in SetupOrchestrator to emit state changes, and Tauri adapter in wiring.rs.

3. **SetupPage Subscription** (`e508cff4`): SetupPage now listens to `setup-state-changed` events and automatically transitions from loading to short code confirmation.

4. **Global Provider** (`feat(pairing-ui): add global pairing notification provider`): Created `PairingNotificationProvider` mounted in App.tsx that shows toast notifications on any page.

5. **Space Access Completion** (`impl(space-access): emit completion event`): SpaceAccessOrchestrator emits `space-access-completed` event after Sponsor successfully persists joiner access.

6. **Complete State Machine** (`feat(pairing-ui): gate responder completion`): Provider implements full state machine: request → verification → verifying → success (gated on space-access-completed).

7. **Verification**: All Rust tests pass (105 passed), frontend builds successfully.

### Key Technical Decisions

- **Session Isolation**: All events filtered by `sessionId` to prevent cross-talk between concurrent pairing attempts
- **Completion Gating**: `p2p-pairing-verification: complete` does NOT trigger final success; only `space-access-completed: success=true` does
- **DevicesPage Independence**: New provider is completely independent of existing DevicesPage pairing logic
- **Hexagonal Boundaries**: New Port (`SetupEventPort`) defined in uc-core, implemented in uc-tauri, maintaining clean architecture
