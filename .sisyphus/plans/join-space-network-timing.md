# Fix Join Space Network Startup Timing

## TL;DR

> **Quick Summary**: 修复 Join Space 在未初始化加密前无法发现设备的问题：在 `EnsureDiscovery` 动作内确保网络已启动，并把网络启动变成幂等操作，避免 `ensure_ready()` 二次启动失败。
>
> **Deliverables**:
>
> - 后端：`NetworkControlPort::start_network()` 幂等化 + `EnsureDiscovery` 先启动网络后发现 peers
> - 后端：`AppLifecycleCoordinator::ensure_ready()` 遇到“网络已启动”可安全跳过
> - 前端：Join 设备选择页增加 scanning/loading + 周期轮询
> - 测试：Rust 单测 + 前端 Vitest 覆盖关键路径
>
> **Estimated Effort**: Medium
> **Parallel Execution**: YES - 3 waves
> **Critical Path**: Task 1 → Task 2 → Task 3 → Task 6

---

## Context

### Original Request

用户在日志中观察到 `StartJoinSpace` 已触发 `EnsureDiscovery`，但怀疑 network 启动时序有问题：当前 network 在 app ready/解锁后才启动，而 Join Space 设备发现依赖 mDNS。

### Interview Summary

**Key Discussions**:

- 用户决定把 network 启动集成到 Join Space 路径（`EnsureDiscovery`）中。
- 用户决定幂等性在 `NetworkControlPort` 层实现，而不是只在上层绕过。
- 用户决定 `ensure_ready` 仍保留启动逻辑，但网络已启动时打印日志并跳过。
- 用户决定前端本次纳入轮询 + loading 状态，不留到后续。
- 测试策略选择 `Tests-after`。

**Research Findings**:

- `EnsureDiscovery` 当前仅调用 `discovery_port.list_discovered_peers()` 读取缓存，不会真正启动发现能力。
- `Libp2pNetworkAdapter::spawn_swarm()` 当前依赖 `take_keypair()`，天然一次性，不可重入。
- `get_discovered_peers()` 仅返回缓存；swarm 未启动时缓存为空，不报错但结果空。
- Setup 页面当前仅进入 `JoinSpaceSelectDevice` 时刷新一次 peers，缺少周期轮询。

### Metis Review

**Identified Gaps (addressed in this plan)**:

- Gap 1: 未明确“幂等 + 启动失败重试”语义。→ 本计划为 Task 1 明确状态机与失败回滚标准。
- Gap 2: `SetupOrchestrator` 缺少 `NetworkControlPort` 注入路径。→ Task 2 增加依赖并更新 wiring/test 构建点。
- Gap 3: 轮询方案与后端事件方案边界不清。→ 本计划锁定“前端轮询”，不引入新 Tauri 事件面。

---

## Work Objectives

### Core Objective

在不破坏现有六边形架构边界前提下，让 Join Space 在未完成加密初始化时也能触发 mDNS 发现，并保证后续 lifecycle 启动流程幂等与可观测。

### Concrete Deliverables

- `Libp2pNetworkAdapter` 的 `start_network()` 幂等实现（含失败回滚语义）。
- `SetupOrchestrator` 在 `EnsureDiscovery` 中调用 `NetworkControlPort::start_network()`。
- `AppLifecycleCoordinator::ensure_ready()` 对“网络已启动”场景不报错。
- `SetupPage` 在 `JoinSpaceSelectDevice` 状态下自动轮询 `getP2PPeers()` 并展示扫描状态。
- 覆盖上述行为的 Rust + 前端自动化测试。

### Definition of Done

- [x] Join Space 首次进入时会触发网络启动（若未启动），随后 peers 列表可在轮询窗口内更新。
- [x] `ensure_ready()` 在网络已启动时不报错、不重复启动、不污染状态。
- [x] 相关测试通过（Rust 与前端测试）。

### Must Have

- Join Space 路径提前启动 swarm network。
- `NetworkControlPort` 层幂等，含日志与失败重试语义。
- 前端 scanning/loading + 轮询行为。

### Must NOT Have (Guardrails)

- 不改为“应用启动即全局启动网络”。
- 不扩展 pairing 协议或 setup 状态机大重构。
- 不引入人工验证步骤；所有验收由 agent 执行。

---

## Verification Strategy (MANDATORY)

> **UNIVERSAL RULE: ZERO HUMAN INTERVENTION**
>
> 所有验收必须可由 agent 执行，不允许“用户手动点击验证”。

### Test Decision

- **Infrastructure exists**: YES
- **Automated tests**: Tests-after
- **Framework**: Rust `cargo test` + Frontend `vitest`

### Agent-Executed QA Scenarios (MANDATORY — ALL tasks)

Scenario: Join Space 首次进入触发网络启动并开始扫描
Tool: Bash
Preconditions: dev 环境可运行测试，network adapter 测试桩可观察 `start_network` 调用
Steps: 1. 在 `src-tauri/` 执行 `cargo test -p uc-app ensure_discovery_starts_network_before_listing_peers -- --exact` 2. 断言输出包含测试名与 `ok`
Expected Result: 测试通过，证明 EnsureDiscovery 顺序正确（先 start 再 list）
Failure Indicators: 测试失败或断言顺序不满足
Evidence: 终端输出日志

Scenario: Lifecycle 重入调用不重复启动网络
Tool: Bash
Preconditions: network start 幂等测试已实现
Steps: 1. 在 `src-tauri/` 执行 `cargo test -p uc-app ensure_ready_succeeds_when_network_already_started -- --exact` 2. 断言输出包含 `ok`
Expected Result: `ensure_ready` 不因已启动网络失败
Failure Indicators: 返回 network failed 或重复启动错误
Evidence: 终端输出日志

Scenario: Join 页面轮询在设备选择状态自动执行
Tool: Bash
Preconditions: 前端测试已添加 fake timers + API mock
Steps: 1. 在项目根执行 `bun test src/pages/__tests__/setup-peer-discovery-polling.test.tsx` 2. 断言 mock `getP2PPeers` 在定时窗口内被多次调用 3. 断言离开 Join 状态后轮询清理（调用次数停止增长）
Expected Result: 轮询生命周期正确、无泄露
Failure Indicators: 调用次数不变或持续增长（未清理）
Evidence: 测试输出日志

Scenario: 网络启动失败后可恢复重试（负向场景）
Tool: Bash
Preconditions: adapter 测试可注入首次失败
Steps: 1. 在 `src-tauri/` 执行 `cargo test -p uc-platform start_network_can_retry_after_failed_start -- --exact` 2. 断言输出包含 `ok`
Expected Result: 首次失败后状态回滚，后续可重试成功
Failure Indicators: 状态卡死在 starting/started，后续无法启动
Evidence: 终端输出日志

---

## Execution Strategy

### Parallel Execution Waves

Wave 1 (Start Immediately):
├── Task 1: NetworkControlPort 幂等契约与实现
└── Task 4: 前端轮询与 scanning 交互设计（实现可先行）

Wave 2 (After Wave 1):
├── Task 2: SetupOrchestrator EnsureDiscovery 接入 network_control
├── Task 3: ensure_ready 对已启动网络的幂等处理
└── Task 5: 后端测试补齐（依赖 1/2/3）

Wave 3 (After Wave 2):
└── Task 6: 前端测试 + 全量验证命令

Critical Path: Task 1 → Task 2 → Task 3 → Task 6
Parallel Speedup: ~30% faster than sequential

### Dependency Matrix

| Task | Depends On | Blocks | Can Parallelize With |
| ---- | ---------- | ------ | -------------------- |
| 1    | None       | 2,3,5  | 4                    |
| 2    | 1          | 5      | 3,4                  |
| 3    | 1          | 5      | 2,4                  |
| 4    | None       | 6      | 1,2,3                |
| 5    | 1,2,3      | 6      | None                 |
| 6    | 4,5        | None   | None                 |

### Agent Dispatch Summary

| Wave | Tasks | Recommended Agents                                                                                           |
| ---- | ----- | ------------------------------------------------------------------------------------------------------------ |
| 1    | 1,4   | `delegate_task(category="unspecified-high", load_skills=["test-driven-development","systematic-debugging"])` |
| 2    | 2,3,5 | backend tasks 并行调度，前提是 Task 1 完成                                                                   |
| 3    | 6     | 集成验证与收敛                                                                                               |

---

## TODOs

- [x] 1. 让 `NetworkControlPort::start_network()` 具备幂等与失败可重试语义

  **What to do**:
  - 在 `Libp2pNetworkAdapter` 增加运行状态标记（建议 CAS + 状态回滚，而非单一 bool 盲跳过）。
  - 明确语义：
    - 已启动：记录 `info` 并返回 `Ok(())`。
    - 启动中：避免重复启动（可直接 `Ok(())` 或等待，按现有架构取最小变更）。
    - 启动失败：必须回滚状态，允许后续重试。
  - 保持 `NetworkControlPort` 边界，不把 libp2p 细节泄漏到 uc-app。

  **Must NOT do**:
  - 不在 uc-app 写平台实现细节。
  - 不使用 panic/unwrap 处理启动失败。

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: 并发状态与一次性资源（keypair）处理属于高风险后端改动。
  - **Skills**: `systematic-debugging`, `test-driven-development`
    - `systematic-debugging`: 处理竞态与状态回滚。
    - `test-driven-development`: 保障幂等契约可回归。
  - **Skills Evaluated but Omitted**:
    - `frontend-ui-ux`: 与本任务域无关。

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Task 4)
  - **Blocks**: 2, 3, 5
  - **Blocked By**: None

  **References**:
  - `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs:258` - `spawn_swarm()` 当前为一次性路径。
  - `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs:333` - `take_keypair()` 二次调用会失败，是幂等修复核心约束。
  - `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs:491` - `NetworkControlPort` 实现入口。
  - `src-tauri/crates/uc-core/src/ports/network_control.rs` - port 契约定义（若存在同名文件；执行时核实并遵循）。

  **Acceptance Criteria**:
  - [x] 新增测试：`start_network_is_idempotent_when_called_twice`
  - [x] 新增测试：`start_network_can_retry_after_failed_start`
  - [x] `cargo test -p uc-platform start_network_is_idempotent_when_called_twice -- --exact` → PASS
  - [x] `cargo test -p uc-platform start_network_can_retry_after_failed_start -- --exact` → PASS

  **Commit**: YES
  - Message: `fix(network): make start_network idempotent with retry-safe state`
  - Files: `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs`, related tests
  - Pre-commit: `cargo test -p uc-platform`

- [x] 2. 在 `EnsureDiscovery` 中先启动网络，再读取已发现 peers

  **What to do**:
  - 给 `SetupOrchestrator` 注入 `Arc<dyn NetworkControlPort>`。
  - 更新 `SetupOrchestrator::new(...)` 及所有构建点（runtime + tests）。
  - 修改 `SetupAction::EnsureDiscovery` 流程：先 `network_control.start_network().await`，再 `discovery_port.list_discovered_peers().await`。
  - 启动失败时记录结构化日志，并将错误映射为 setup 可观测错误路径（避免静默吞错）。

  **Must NOT do**:
  - 不把 `Libp2pNetworkAdapter` 具体类型注入 uc-app。
  - 不跳过现有 setup 状态机事件流。

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: 涉及 usecase 构造签名和跨模块依赖更新。
  - **Skills**: `systematic-debugging`, `verification-before-completion`
    - `systematic-debugging`: 防止 wiring 漏改导致运行时崩溃。
    - `verification-before-completion`: 强制全调用点编译验证。
  - **Skills Evaluated but Omitted**:
    - `ui-ux-pro-max`: 不涉及 UI 设计。

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 2
  - **Blocks**: 5
  - **Blocked By**: 1

  **References**:
  - `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs:45` - orchestrator 依赖字段定义。
  - `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs:207` - `EnsureDiscovery` 当前执行位置。
  - `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs:256` - runtime 构建 orchestrator 的注入点。
  - `src-tauri/crates/uc-app/src/deps.rs:63` - `network_control` 已在 `AppDeps` 可用。
  - `src-tauri/crates/uc-core/src/setup/action.rs:3` - `EnsureDiscovery` 语义注释（“Ensure device discovery is running”）。

  **Acceptance Criteria**:
  - [x] `EnsureDiscovery` 逻辑顺序为 start_network → list_discovered_peers。
  - [x] `SetupOrchestrator` 构建点全部编译通过。
  - [x] `cargo test -p uc-app ensure_discovery_starts_network_before_listing_peers -- --exact` → PASS

  **Commit**: YES
  - Message: `fix(setup): start network before discovery in join flow`
  - Files: `orchestrator.rs`, `runtime.rs`, impacted tests
  - Pre-commit: `cargo test -p uc-app`

- [x] 3. 让 `AppLifecycleCoordinator::ensure_ready()` 对已启动网络安全重入

  **What to do**:
  - 保留 `ensure_ready()` 的流程职责，但当 network 已启动时不视为失败。
  - 输出结构化 `info!` 日志，标记“network already started; skip”。
  - 校验状态机：最终仍可到 `Ready`，不误报 `NetworkFailed`。

  **Must NOT do**:
  - 不删除 network step。
  - 不改变 watcher 失败/网络失败的错误分流语义。

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 局部逻辑调整，但对可观测性要求高。
  - **Skills**: `systematic-debugging`
    - `systematic-debugging`: 保证状态迁移无回归。
  - **Skills Evaluated but Omitted**:
    - `frontend-ui-ux`: 非前端任务。

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Task 2 after Task 1)
  - **Blocks**: 5
  - **Blocked By**: 1

  **References**:
  - `src-tauri/crates/uc-app/src/usecases/app_lifecycle/mod.rs:158` - `ensure_ready()` 主流程。
  - `src-tauri/crates/uc-app/src/usecases/app_lifecycle/mod.rs:183` - network step 失败处理。
  - `src-tauri/crates/uc-app/src/usecases/start_network_after_unlock.rs:24` - network 启动 usecase。

  **Acceptance Criteria**:
  - [x] 已启动网络场景下 `ensure_ready()` 返回 `Ok(())`。
  - [x] `LifecycleState` 最终可设置为 `Ready`。
  - [x] `cargo test -p uc-app ensure_ready_succeeds_when_network_already_started -- --exact` → PASS

  **Commit**: YES
  - Message: `fix(lifecycle): make ensure_ready tolerant to pre-started network`
  - Files: `app_lifecycle/mod.rs`, related tests
  - Pre-commit: `cargo test -p uc-app`

- [x] 4. 前端 Join 设备选择页增加自动轮询与 scanning/loading 体验

  **What to do**:
  - 在 `SetupPage` 进入 `JoinSpaceSelectDevice` 时启动 interval 轮询（调用 `getP2PPeers()`）。
  - 离开该状态时清理 interval，避免内存泄漏和后台调用。
  - 增加首轮扫描状态（例如 `isScanningInitial`），让空列表时文案区分“正在扫描”与“未发现设备”。
  - 保留手动刷新按钮，避免交互倒退。

  **Must NOT do**:
  - 不引入新的 Tauri setup 命令（本次锁定前端直接轮询 `get_p2p_peers`）。
  - 不改动 setup 状态结构或后端 API contract。

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: 前端状态与交互行为改造。
  - **Skills**: `frontend-ui-ux`
    - `frontend-ui-ux`: 保持现有视觉语言并增强状态反馈。
  - **Skills Evaluated but Omitted**:
    - `vercel-react-best-practices`: 可选，但本任务非性能瓶颈主导。

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Task 1)
  - **Blocks**: 6
  - **Blocked By**: None

  **References**:
  - `src/pages/SetupPage.tsx:53` - 现有手动刷新实现。
  - `src/pages/SetupPage.tsx:72` - 仅首次进入 Join 状态刷新一次的现状。
  - `src/pages/SetupPage.tsx:139` - Join step 渲染入口。
  - `src/pages/setup/JoinPickDeviceStep.tsx:69` - 空列表与 loading UI 呈现位置。
  - `src/api/p2p.ts:152` - `getP2PPeers()` API。

  **Acceptance Criteria**:
  - [x] 进入 `JoinSpaceSelectDevice` 后自动轮询 peers。
  - [x] 离开 `JoinSpaceSelectDevice` 后轮询停止。
  - [x] loading/scanning 状态可区分“扫描中”与“空结果”。

  **Commit**: YES
  - Message: `feat(setup-ui): add join-space peer polling and scanning state`
  - Files: `src/pages/SetupPage.tsx`, `src/pages/setup/JoinPickDeviceStep.tsx`, i18n if needed
  - Pre-commit: `bun test src/pages/__tests__/setup-peer-discovery-polling.test.tsx`

- [x] 5. 补齐后端测试（setup/lifecycle/network）

  **What to do**:
  - 在 `uc-platform` 增加 start_network 幂等与失败重试测试。
  - 在 `uc-app` setup orchestrator 测试中增加 EnsureDiscovery 启动顺序测试。
  - 在 `uc-app` lifecycle 测试中增加“网络已启动时 ensure_ready 仍成功”测试。

  **Must NOT do**:
  - 不写依赖人工观察日志的测试。

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 已有测试框架，主要是补测与断言校准。
  - **Skills**: `test-driven-development`
    - `test-driven-development`: 规范命名与断言完整性。
  - **Skills Evaluated but Omitted**:
    - `writing-plans`: 非规划任务。

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 2
  - **Blocks**: 6
  - **Blocked By**: 1,2,3

  **References**:
  - `src-tauri/crates/uc-app/tests/app_lifecycle_coordinator_test.rs` - lifecycle 测试模式。
  - `src-tauri/crates/uc-app/tests/setup_flow_integration_test.rs` - setup flow 集成测试样式。
  - `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs:1117` - 现有 mock wiring 位置。
  - `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs` - network adapter 待测行为。

  **Acceptance Criteria**:
  - [x] `cargo test -p uc-platform` 通过。
  - [x] `cargo test -p uc-app` 通过。
  - [x] 新增测试名与行为语义一致（见 Task 1/2/3 的测试名）。

  **Commit**: YES
  - Message: `test(setup): cover network start idempotency and lifecycle reentry`
  - Files: uc-app / uc-platform tests
  - Pre-commit: `cargo test -p uc-platform && cargo test -p uc-app`

- [x] 6. 前端测试与全量验收收敛

  **What to do**:
  - 新增 `SetupPage` 轮询生命周期测试（fake timers + mocked API）。
  - 校验进入/退出 Join 状态时轮询启停。
  - 运行关键后端+前端命令形成最终验收证据。

  **Must NOT do**:
  - 不省略负向场景（例如 API 报错时 loading 恢复）。

  **Recommended Agent Profile**:
  - **Category**: `unspecified-low`
    - Reason: 集成验证与收尾。
  - **Skills**: `verification-before-completion`
    - `verification-before-completion`: 防止“口头通过”。
  - **Skills Evaluated but Omitted**:
    - `requesting-code-review`: 可选，不是本步硬要求。

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 3
  - **Blocks**: None
  - **Blocked By**: 4,5

  **References**:
  - `src/pages/__tests__/setup-ready-flow.test.tsx` - SetupPage 测试风格。
  - `src/api/__tests__/setup.test.ts` - API mock 结构模式。
  - `src/pages/SetupPage.tsx` - 被测组件主入口。

  **Acceptance Criteria**:
  - [x] `bun test src/pages/__tests__/setup-peer-discovery-polling.test.tsx` → PASS
  - [x] `bun test src/pages/__tests__/setup-ready-flow.test.tsx` → PASS
  - [x] `cd src-tauri && cargo test -p uc-platform && cargo test -p uc-app` → PASS

  **Commit**: YES
  - Message: `test(frontend): verify setup peer polling lifecycle`
  - Files: frontend tests + related setup page adjustments
  - Pre-commit: listed acceptance commands

---

## Commit Strategy

| After Task | Message                                                             | Files                                     | Verification                |
| ---------- | ------------------------------------------------------------------- | ----------------------------------------- | --------------------------- |
| 1          | `fix(network): make start_network idempotent with retry-safe state` | `libp2p_network.rs` + tests               | `cargo test -p uc-platform` |
| 2          | `fix(setup): start network before discovery in join flow`           | `orchestrator.rs`, `runtime.rs`, tests    | `cargo test -p uc-app`      |
| 3          | `fix(lifecycle): make ensure_ready tolerant to pre-started network` | `app_lifecycle/mod.rs`, tests             | `cargo test -p uc-app`      |
| 4          | `feat(setup-ui): add join-space peer polling and scanning state`    | `SetupPage.tsx`, `JoinPickDeviceStep.tsx` | `bun test ...polling...`    |
| 5-6        | `test(setup): cover join discovery timing and polling behavior`     | test files only                           | full test suite commands    |

> Commit 必须遵守 atomic rule：每个 commit 单一意图，不混入无关重构。

---

## Success Criteria

### Verification Commands

```bash
# Frontend
bun test src/pages/__tests__/setup-peer-discovery-polling.test.tsx
bun test src/pages/__tests__/setup-ready-flow.test.tsx

# Backend (MUST run from src-tauri/)
cd src-tauri
cargo test -p uc-platform
cargo test -p uc-app
```

### Final Checklist

- [x] Join Space 进入后无需先完成 encryption 也能启动发现流程
- [x] `EnsureDiscovery` 语义与实现一致（真正“ensure” discovery）
- [x] `ensure_ready` 重入安全，日志可观测
- [x] 前端轮询仅在 Join 设备选择状态启用，退出即停止
- [x] 所有自动化测试通过
