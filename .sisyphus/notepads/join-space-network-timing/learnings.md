# Learnings - Join Space Network Timing Fix

## Conventions

- All Rust tests run from `src-tauri/` directory
- NetworkControlPort is the boundary between uc-app and uc-platform
- Libp2pNetworkAdapter::spawn_swarm() is currently single-shot due to take_keypair()

## Key Files

- `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs` - Network adapter implementation
- `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs` - Setup orchestrator
- `src-tauri/crates/uc-app/src/usecases/app_lifecycle/mod.rs` - Lifecycle coordinator
- `src/pages/SetupPage.tsx` - Frontend setup page
- `src/pages/setup/JoinPickDeviceStep.tsx` - Device selection UI

## Status

- Wave 1: Task 1 + Task 4 (parallel)
- Wave 2: Task 2 + Task 3 + Task 5 (after Wave 1)
- Wave 3: Task 6 (final integration)

## 2026-02-08

- Added uc-app integration test in to assert call order .
- Verified lifecycle reentry coverage already exists as in .
- Verification commands passed: and .

## 2026-02-08 (correction)

- Added uc-app integration test ensure_discovery_starts_network_before_listing_peers in src-tauri/crates/uc-app/tests/setup_flow_integration_test.rs to assert call order [network, discovery].
- Verified lifecycle reentry coverage already exists as ensure_ready_succeeds_when_network_already_started in src-tauri/crates/uc-app/tests/app_lifecycle_status_test.rs.
- Verification commands passed from src-tauri: cargo test -p uc-platform and cargo test -p uc-app.

## 2026-02-08 - Plan Completed

All 6 tasks completed successfully:

### Completed Tasks

1. ✅ Task 1: NetworkControlPort idempotent start_network with CAS state machine
2. ✅ Task 2: EnsureDiscovery starts network before listing peers
3. ✅ Task 3: ensure_ready idempotent for pre-started network
4. ✅ Task 4: Frontend polling and scanning UI
5. ✅ Task 5: Backend tests for setup/lifecycle/network
6. ✅ Task 6: Frontend tests and final verification

### Test Results

- Rust (uc-platform): 76 passed, 2 ignored
- Rust (uc-app): 22 passed, 3 ignored
- Frontend: 4 passed (3 polling + 1 setup-ready-flow)

### Key Learnings

- Frontend tests with fake timers require stable function references in mocks
- react-i18next mock caused infinite re-renders due to new function references each call
- Solution: Cache translation functions by keyPrefix to maintain stable references
