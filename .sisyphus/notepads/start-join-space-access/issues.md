# StartJoinSpaceAccess Issues

- `cargo test -p uc-app --lib` 当前未全绿：`usecases::space_access::proof_adapter::tests::build_and_verify_round_trip_succeeds` 在 `crates/uc-app/src/usecases/space_access/proof_adapter.rs:63` 断言失败（`assertion failed: valid`）。
- 已解决：实现 `HmacProofAdapter` 的 HMAC 生成与校验后，上述测试与 `cargo test -p uc-app --lib` 已全绿。

- 已解决：`SetupOrchestrator` 引入 `SpaceAccessExecutor` 后，Tauri command future 出现 `!Send`（`dyn CryptoPort` / `dyn ProofPort` 非 `Sync`）编译错误；通过将 `uc-core` 的 `CryptoPort` 与 `ProofPort` trait bound 提升为 `Send + Sync` 消除。

- 已解决：join 集成测试初版在 `join_space_access_propagates_space_access_error` 超时，根因是测试里持有 `transport/timer/persistence` 的 mutex guard 未释放，导致 `submit_passphrase()` 获取同一组 port 锁时阻塞。
