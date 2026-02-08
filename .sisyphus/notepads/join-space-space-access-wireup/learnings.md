## 2026-02-08

- 在 `wiring.rs` 的 `PairingMessage::Busy` 里先做 transport JSON 解析，再继续调用 `orchestrator.handle_busy(...)`，可以在不改变现有流程的前提下为后续路由任务预留入口。
- Busy `reason` 的 JSON 解析需要按 `kind` 分支反序列化并启用 `deny_unknown_fields`，这样能在 wiring 层尽早发现 schema 漂移并带上 `session_id`/`peer_id` 记录结构化日志。
- 在 `PairingStateMachine` 添加 `role()` 方法可以暴露当前会话角色，供 wiring 层在 `EmitResult` 时判断是否为 Sponsor。
- Sponsor auto-trigger 在 wiring 层通过 `key_slot_store.load()` 获取 keyslot 并提取 `space_id`，再用 Noop ports 构造 `SpaceAccessExecutor` 来调用 `start_sponsor_authorization`。
- 会话关闭策略在 `EmitResult` 处理中实现：仅当 `!success` 时调用 `close_pairing_session`，成功时不关闭，等待 space-access 终端状态。
- `start_background_tasks(...)` 新增 `key_slot_store` 参数后，`main.rs` 需要按 wiring 同源路径规则构造 `JsonKeySlotStore(vault_dir/keyslot.json)` 并传入，才能保持 sponsor auto-trigger 读取一致性。
- `run_pairing_action_loop(...)` 签名新增 `space_access_orchestrator` 与 `key_slot_store` 后，wiring 底部测试调用点必须全部补齐参数，否则会在编译阶段报参数数量不匹配。
- `NoopKeySlotStore` 测试桩应返回 `uc_core::security::model::EncryptionError`，不要使用 `uc_core::ports::errors::EncryptionError`，否则 trait 实现类型不一致。
- `seed_waiting_decision_state` 必须使用可成功 `build_proof` 的 proof port（`SuccessSpaceAccessProof`），否则 `PassphraseSubmitted` 无法到达 `WaitingDecision`，会直接在预置阶段报 `Crypto` 错误。
