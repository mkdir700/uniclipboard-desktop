# Learnings – start-network-after-unlock

## 2026-02-07 Network-start audit

- `AppLifecycleCoordinator::ensure_ready` 是唯一的 Ready 发射器，并且在 `src-tauri/crates/uc-app/src/usecases/app_lifecycle/mod.rs:137-175` 中先调度 `StartNetworkAfterUnlock`（经 `self.network.execute()`）再把状态写成 `LifecycleState::Ready`
- 当前所有网络启动路径都只是不同入口去触发 `ensure_ready`：
  - Setup 创建空间流程的 `SetupAction::CreateEncryptedSpace`（`src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs:147-158`）在初始化加密后立刻调用 `app_lifecycle.ensure_ready()`
  - `initialize_encryption` 命令（`src-tauri/crates/uc-tauri/src/commands/encryption.rs:69-76`）完成初始化后无条件触发 `ensure_ready`
  - `unlock_encryption_session_with_runtime` 的成功分支（`src-tauri/crates/uc-tauri/src/commands/encryption.rs:125-138`）在 Keyring 解锁成功后触发 `ensure_ready`；它被 UI 命令 `unlock_encryption_session` 复用，也被 `main.rs` 启动时的自动解锁任务（`src-tauri/src/main.rs:559-586`）后台调用
  - `retry_lifecycle` 命令（`src-tauri/crates/uc-tauri/src/commands/lifecycle.rs:15-33`）直接调用 `ensure_ready` 来重新开机
- `StartNetwork` 用例目前仅在测试里引用，生产代码的网络启动都走 `StartNetworkAfterUnlock`

## 2026-02-07 Ready→Network TDD (Task 2)

- 新增 `unlock_triggers_ready_and_network_once`、`repeated_unlock_attempts_do_not_restart_network_when_ready` 两个生命周期用例测试（`src-tauri/crates/uc-app/tests/app_lifecycle_coordinator_test.rs`）。第二个测试当前 RED，证明 Ready 后重复解锁仍会重启 watcher/network。
- 在 `src-tauri/crates/uc-core/src/setup/state_machine.rs` 添加 `mark_setup_complete_is_the_ready_bridge` 测试，并用 TODO 说明 Ready 发生在 uc-app 层，Setup 状态机只能通过 `MarkSetupComplete` 将控制权交给应用层。
- 命令 `cd src-tauri && cargo test -p uc-app repeated_unlock_attempts_do_not_restart_network_when_ready` 失败（期望 watcher/network=1，实际为2；待任务3修复）。`cd src-tauri && cargo test -p uc-core mark_setup_complete_is_the_ready_bridge` 通过，说明 Setup Completed → `MarkSetupComplete` 的铰链工作正常。

## 2026-02-07 Ready幂等实现（Task 3）

- `AppLifecycleCoordinator::ensure_ready` 现在在写入 Pending 之前先读取 `LifecycleStatusPort`，如果已为 `LifecycleState::Ready` 则记录日志并直接返回，阻止 watcher/network 被重复启动。
- 完整 `cargo test -p uc-app app_lifecycle` 以及单测 `cargo test -p uc-app repeated_unlock_attempts_do_not_restart_network_when_ready` 均转绿，验证 Ready 幂等行为生效。
- 对 `start_network`/`start_network_after_unlock` 调用点复查（`rg "StartNetwork"` / `rg "start_network("`），确认生产路径只通过 `ensure_ready` 间接调用网络启动；无额外入口需要删减。

## 2026-02-07 Setup UI 解耦（Task 4）

- 初始审计发现 `SetupPage` 已仅依赖 `getSetupState()`，但缺少覆盖，故新增独立测试文件 `src/pages/__tests__/setup-ready-flow.test.tsx`。由于 `bun test` 默认 runner 不支持 Vitest 的 `vi.importActual`/环境钩子，命令 `bun test --filter setup-ready-flow` 会提示 unsupported features；改用 `bunx vitest run src/pages/__tests__/setup-ready-flow.test.tsx` 作为可执行替代并记录在日志中。
- 新测试验证：当 `getSetupState` 返回 `'Completed'` 时 `SetupDoneStep` 立即渲染，点击“进入 UniClipboard”按钮会触发 `onCompleteSetup` 并导航至 `/`；证明 Setup UI 与网络状态完全解耦，只依赖 setup 状态。

## 2026-02-07 Observability/Docs（Task 5）

- 新增 `docs/plans/2026-02-07-lifecycle-ready-network.md` 描述 Ready→网络启动顺序、幂等行为与日志/验证命令，作为架构记录。
- `AppLifecycleCoordinator::ensure_ready` 已在 span `usecase.app_lifecycle_coordinator.ensure_ready` 内记录 Pending/Ready，以及重复调用被跳过的 info 日志；文档也强调这些日志可观察性。

## 2026-02-08 start_network 幂等 + 失败可重试

- `Libp2pNetworkAdapter` 增加了 `AtomicU8` 启动状态（Idle/Starting/Started/Failed），`NetworkControlPort::start_network()` 使用 CAS 把并发入口收敛到单次启动；当状态为 Starting 或 Started 时直接返回 `Ok(())`，避免重复启动。
- `start_network()` 在启动失败时先写入 Failed 再回滚到 Idle，允许下一次重试；并通过测试中“先拿走 business receiver 再恢复后重试”的场景验证回滚可用。
- `keypair` 从一次性 `Mutex<Option<Keypair>>` 调整为可重复读取的 `Mutex<Keypair>` 并在 `take_keypair()` 中克隆，避免首次失败后出现“keypair already taken”导致不可重试。

## 2026-02-08 ensure_ready 对已启动网络安全重入

- `AppLifecycleCoordinator::ensure_ready` 的 network step 在收到包含 `already started` 的错误文案时不再走 `NetworkFailed` 分支，而是记录结构化 `info!(error = %msg, "network already started; skip")` 并继续后续 Ready 流程。
- 新增回归测试 `ensure_ready_succeeds_when_network_already_started`（`src-tauri/crates/uc-app/tests/app_lifecycle_status_test.rs`），覆盖状态迁移 `Pending -> Ready` 且仅发出 `LifecycleEvent::Ready`，防止误报 `NetworkFailed`。
