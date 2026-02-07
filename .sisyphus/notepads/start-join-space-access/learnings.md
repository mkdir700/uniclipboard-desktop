# StartJoinSpaceAccess Plan Notes

## Wave 1: Independent Adapter Implementations

### Task 1: Fix SpaceAccessTransportPort trait + network_adapter.rs

- Change trait methods to return `anyhow::Result<()>`
- Update ALL consumers in orchestrator.rs
- Rewrite network_adapter.rs to use NetworkPort.send_pairing_on_session()
- Register in mod.rs

### Task 2: Implement derive_master_key_from_keyslot()

- Deserialize keyslot blob into KeySlot struct
- Derive KEK from passphrase + salt
- Unwrap master key using encryption port
- Store via key_material port
- Set in encryption_session
- Mark initialized via encryption_state
- Add rollback logic

### Task 3: Implement ProofPort + PersistencePort adapters

- HmacProofAdapter: HMAC-SHA256 over (pairing_session_id, space_id, challenge_nonce)
- SpaceAccessPersistenceAdapter: persist_joiner_access() and persist_sponsor_access()
- Register both in mod.rs

## Wave 2: Integration

### Task 4: Wire SetupOrchestrator

- Add Arc port fields to SetupOrchestrator
- Implement start_join_space_access() method
- Replace placeholder in execute_actions
- Update test helpers

### Task 5: Update SetupRuntimePorts bootstrap wiring

- Add new port fields
- Update constructors
- Create production adapter instances

### Task 6: Integration tests

- Add join flow tests
- Verify error propagation
- Mock-based testing

### Task 7: Final verification

- cargo check --workspace
- cargo test -p uc-app --lib
- bun run build
- No unwrap/expect in new code

- Joiner 端的 derive_master_key_from_keyslot 需要先把 sponsor 提供的 keyslot blob 作为 serde_json::from_slice::<KeySlotFile> 解析，再复用 export_keyslot_blob 的回滚模式（store_kek -> store_keyslot -> set_master_key -> persist_initialized，任何环节失败都要删除 KEK/KeySlot 并清理 session）。
- Added HmacProofAdapter using HMAC-SHA256 and SpaceAccessPersistenceAdapter bridging encryption state + paired device repo so Setup can construct production Proof/Persistence ports.
- SetupRuntimePorts now owns injected space-access runtime adapters (crypto/transport/proof/timer/persistence), and `build_setup_orchestrator()` consumes those fields instead of constructing adapters inline.

- Added integration tests for StartJoinSpaceAccess dispatch and error propagation using mock TimerPort to force SpaceAccess failure paths.
- Added joiner-side key derivation test that records derive_master_key_from_keyslot inputs in MockCrypto to validate joiner flow wiring.

- Phase 0 检查结果：`SpaceAccessTransportPort` 在 `uc-core` 仍是 `async fn ...;` 返回 `()`，但 `space_access/network_adapter.rs` 已实现为 `-> anyhow::Result<()>`，当前签名不一致。
- `space_access/mod.rs` 未声明 `mod network_adapter;`，也未 `pub use` 该 adapter；`SpaceAccessNetworkAdapter` 当前仅在其定义文件出现。
- `space_access/orchestrator.rs` 在 `SendOffer/SendProof/SendResult` 分支调用 transport 时未使用 `?` 传播错误；测试中的 `MockTransport` 也仍为返回 `()`。

- Task 2 implementation landed in `uc-app/src/usecases/space_access/crypto_adapter.rs`: `derive_master_key_from_keyslot()` now deserializes `keyslot_blob` via `serde_json::from_slice::<KeySlot>`, derives KEK, unwraps master key, stores key material/session, and persists initialized state.
- Rollback pattern for joiner derive now mirrors export flow: after `store_kek` succeeds, any failure in `store_keyslot` / `unwrap_master_key` / `set_master_key` / `persist_initialized` triggers `delete_keyslot` + `delete_kek`; persist failure also clears encryption session.
- Added focused unit tests for derive success and persist-failure rollback, plus refactored test doubles to capture side effects (`store_kek_called`, `store_keyslot_called`, `clear_called`, `persist_initialized_called`).

- `SpaceAccessTransportPort` 统一改为 `anyhow::Result<()>` 后，需要同步更新 orchestrator 三个 `.await` 调用点为 `.await?`，并把所有 `MockTransport` 实现改为返回 `Ok(())`。
- `SpaceAccessNetworkAdapter` 重写时可直接复用 `NetworkPort::send_pairing_on_session()`：将 `SpaceAccessContext` 快照序列化为 JSON 字符串（`serde_json::to_string`）并封装进 `PairingMessage::Busy.reason` 发送。
- `space_access/mod.rs` 需要显式 `mod network_adapter;` + `pub use network_adapter::SpaceAccessNetworkAdapter;`，否则 adapter 即使实现完成也不会被模块导出。
- `HmacProofAdapter` 在当前 `ProofPort` 签名下（`verify_proof` 无 `master_key` 参数）可通过缓存 `build_proof` 时的 `MasterKey`（按 `pairing_session_id + space_id + nonce` 键）实现“重算 HMAC 再比对”。
- HMAC 输入建议固定编码为 `len(session_id) + session_id + len(space_id) + space_id + nonce`，避免简单拼接导致边界歧义。
- `SpaceAccessPersistenceAdapter::persist_sponsor_access()` 可直接调用 `PairedDeviceRepositoryPort::set_state(peer_id, PairingState::Trusted)` 来更新现有配对设备记录，保持在 port 边界内。

- Wiring `SetupOrchestrator::start_join_space_access()` through Tauri commands requires `SpaceAccessExecutor` to be `Send`; this forced `CryptoPort` and `ProofPort` trait bounds in `uc-core` to become `Send + Sync` so borrowed trait references can cross async await points safely.
- For Task 4 verification, a reliable RED/GREEN test path is to pre-drive `SpaceAccessOrchestrator` state to `WaitingUserPassphrase` (`JoinRequested` -> `OfferAccepted`) and then assert `SubmitPassphrase` maps missing joiner context errors to `SetupError::PairingFailed`.
- 集成测试里要触发 `StartJoinSpaceAccess`，必须先把 Setup 流程推进到 `JoinSpaceInputPassphrase`；当前需要 `JoinSpaceConfirmPeer + ConfirmPeerTrust` 直接落到 `JoinSpaceInputPassphrase`，否则 `confirm_peer_trust_action()` 内部设态会被 dispatch 最终写回覆盖。
- 在 integration test 中复用 `SpaceAccessExecutor` 预置状态时，调用完 `space_access_orchestrator.dispatch(...)` 后要立即 `drop(executor/store/timer/transport)`，否则后续 `SetupOrchestrator.submit_passphrase()` 会等待同一把 mutex 造成卡住。
