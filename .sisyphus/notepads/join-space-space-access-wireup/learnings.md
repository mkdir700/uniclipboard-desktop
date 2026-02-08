## 2026-02-08

- 在 `wiring.rs` 的 `PairingMessage::Busy` 里先做 transport JSON 解析，再继续调用 `orchestrator.handle_busy(...)`，可以在不改变现有流程的前提下为后续路由任务预留入口。
- Busy `reason` 的 JSON 解析需要按 `kind` 分支反序列化并启用 `deny_unknown_fields`，这样能在 wiring 层尽早发现 schema 漂移并带上 `session_id`/`peer_id` 记录结构化日志。
- 在 `PairingStateMachine` 添加 `role()` 方法可以暴露当前会话角色，供 wiring 层在 `EmitResult` 时判断是否为 Sponsor。
- Sponsor auto-trigger 在 wiring 层通过 `key_slot_store.load()` 获取 keyslot 并提取 `space_id`，再用 Noop ports 构造 `SpaceAccessExecutor` 来调用 `start_sponsor_authorization`。
- 会话关闭策略在 `EmitResult` 处理中实现：仅当 `!success` 时调用 `close_pairing_session`，成功时不关闭，等待 space-access 终端状态。
