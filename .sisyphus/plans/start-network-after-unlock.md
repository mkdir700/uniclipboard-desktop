# start-network-after-unlock

## TL;DR

> **Quick Summary**: Retire all legacy "start network after unlock" triggers so the network only starts when the app lifecycle reaches Ready. Audit and clean every entrypoint, enforce the Ready sequence via TDD, and ensure Setup UI flows independently of network status.
>
> **Deliverables**:
>
> - Complete inventory + removal of redundant network-start call sites
> - Lifecycle + setup tests proving Ready emits once unlock completes and triggers network start exactly once
> - Frontend polling/listener adjustments plus telemetry/logging updates
> - Verification scripts (cargo/bun) demonstrating the new sequencing
>
> **Estimated Effort**: Medium
> **Parallel Execution**: YES – two main waves (backend sequencing, then frontend/QA)
> **Critical Path**: Entry-point audit → Lifecycle TDD → Logic removal/unification → Frontend updates → Verification

---

## Context

### Original Request

用户希望删除旧的“解锁后立即手动启动网络”逻辑，只保留 lifecycle Ready 阶段的自动启动，并确保 Setup Done 页面展示与网络启动解耦。

### Interview Summary

**Key Discussions**:

- 网络现仅应在 Ready 状态触发；Setup 完成后自然进入 `SetupDoneStep`，与网络无关。
- 需要追溯任何可能在 Ready 前触发 `start_network*` 的入口（自动解锁、后台任务等）。
- 要求 TDD：先用测试锁定 Ready/网络启动顺序，再删旧逻辑。

**Research Findings**:

- `start_network_after_unlock` 目前只在 `AppLifecycleCoordinator::ensure_ready` 内被调用（`src-tauri/crates/uc-app/src/usecases/app_lifecycle/mod.rs`）。
- Ready 由 `SetupOrchestrator::execute_actions` 执行 `EnsureReady` 时触发，顺序为：启动剪贴板 watcher → 调 `start_network_after_unlock` → 广播 Ready 生命周期事件。
- 前端 `App.tsx` 每秒调用 `getSetupState()`，若非 `Completed` 会持续停留在 Setup UI。

### Metis Review

**Identified Gaps (addressed in plan)**:

- 全面搜索可能隐藏的网络启动入口（auto-unlock、恢复任务、命令 handler）。
- 删除旧逻辑时需保留观察性（日志/遥测）以便追踪 Ready 触发失败。
- 考虑离线/重复解锁等边界场景，确保 Ready 仍会发布且不会重复启动网络。
- 建议顺序：入口审计 → TDD（Ready & Setup state）→ 删除旧逻辑 → 调整前端监听 → 命令化验证。

---

## Work Objectives

### Core Objective

确保 App 只在 lifecycle Ready 阶段统一启动网络，删除冗余触发点，同时保持 Setup UI 与网络状态解耦，并以 TDD 覆盖解锁→Ready→网络启动的链路。

### Concrete Deliverables

- 更新/新增测试覆盖 Ready 触发与网络启动顺序
- 移除旧 `start_network_after_unlock` 入口及相关 wiring
- 调整前端 Setup 流程监听 Ready/Setup 状态而非网络事件
- 文档/日志说明新的启动机制

### Definition of Done

- [x] 所有测试命令（`cargo test -p uc-app app_lifecycle`, `cargo test -p uc-core setup_state_machine`, `bun test --filter setup-ready-flow`）通过
- [x] 代码中仅有 lifecycle Ready 触发网络启动的路径
- [x] Setup UI 正常进入 `SetupDoneStep`，无网络依赖
- [x] 计划任务全部完成并记录

### Must Have

- Ready→网络启动链路具备 TDD 保障
- 全量入口审计报告和移除
- 详细日志/遥测记录 Ready & 网络启动尝试，便于诊断

### Must NOT Have (Guardrails)

- 不得新增额外网络启动触发器
- 不得依赖人工验证（全部命令化）
- 不得更改 Setup 状态机既有顺序

---

## Verification Strategy (MANDATORY)

> **UNIVERSAL RULE: ZERO HUMAN INTERVENTION**
> 全部验收依赖命令化测试与自动 QA scen. Ready/网络逻辑不得依赖手工界面确认。

### Test Decision

- **Infrastructure exists**: YES（Rust + bun 测试）
- **Automated tests**: YES（TDD throughout）
- **Framework**: `cargo test`（uc-app / uc-core），`bun test`（前端）

### TDD Workflow

每个任务若涉及行为变更，按 RED→GREEN→REFACTOR：

1. 编写/更新 failing test（Rust 或 bun）
2. 实现最小逻辑使测试转绿
3. 重构并保持测试通过

### Agent-Executed QA Scenarios

- **Backend**: 在 `src-tauri/` 运行 `cargo test -p uc-app app_lifecycle`，期望输出包含 `ready_triggers_network_after_unlock` 绿色，表明 Ready 触发网络一次且记录日志。
- **Setup state machine**: `cargo test -p uc-core setup_state_machine`，确认解锁事件驱动 Ready transition。
- **Frontend**: `bun test --filter setup-ready-flow`（或新增 case）。验证 SetupDoneStep rendering 独立于网络状态；记录截图/输出路径 `.sisyphus/evidence/task-N-*`。
- **CLI Verification**: 如需额外脚本，使用 `interactive_bash` 捕获输出并保存证据。

Evidence：将测试命令输出或截图保存至 `.sisyphus/evidence/`，命名 `task-<n>-<scenario>.log/png`。

---

## Execution Strategy

### Parallel Execution Waves

- **Wave 1**: 任务 1（入口审计）可独立进行。
- **Wave 2**: 任务 2（TDD Ready tests）、任务 3（删除旧逻辑）并行，需依赖审计结论。
- **Wave 3**: 任务 4（前端调整）在后端完成后执行。
- **Wave 4**: 任务 5（日志/遥测确认与文档）与任务 6（验证与证据）收尾。

### Dependency Matrix

| Task | Depends On | Blocks | Parallel Notes          |
| ---- | ---------- | ------ | ----------------------- |
| 1    | 无         | 2,3    | 可先行完成              |
| 2    | 1          | 3      | 需先锁定入口后写测试    |
| 3    | 2          | 4      | 删除逻辑需测试护航      |
| 4    | 3          | 5,6    | 前端依赖统一 Ready 行为 |
| 5    | 3          | 6      | 日志/文档需新逻辑稳定   |
| 6    | 4,5        | 完结   | 最终验证                |

### Agent Dispatch Summary

- Wave1：category "quick" + skill `project-onboarding`
- Wave2：category "unspecified-high" + skills `project-onboarding`, `test-driven-development`
- Wave3：category "visual-engineering" + skills `project-onboarding`

---

## TODOs

> Implementation + Test = ONE Task. 每个任务包含推荐 agent、参考、验收标准。

- [x] 1. 全量审计网络启动入口
     **What to do**:
  - 使用 `ast_grep_search` / `rg` 搜索 `start_network`, `start_network_after_unlock`, `ensure_ready`, `LifecycleState::Ready` 等，列出所有调用路径（含 `main.rs`, `bootstrap/runtime.rs`, `commands/*.rs`).
  - 记录每个入口的调用顺序、意图及是否需要保留。
  - 输出审计文档（README 或计划注记）供后续任务使用。
    **Must NOT do**: 不修改代码，仅记录。
    **Recommended Agent Profile**: category "quick", skills [`project-onboarding`].
    **Parallelization**: 可与后续任务并行准备；不阻塞。
    **References**: `src-tauri/crates/uc-app/src/usecases/app_lifecycle/mod.rs`, `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`, `src-tauri/src/main.rs`, `src-tauri/crates/uc-tauri/src/commands/*.rs`.
    **Acceptance Criteria**:
  - 文档列出所有入口、文件、行号及保留/删除决策。
  - 作为 Task 2 的输入。

- [x] 2. TDD：Ready→网络启动测试
     **What to do**:
  - 在 `src-tauri/crates/uc-app/tests/` 或 `app_lifecycle` 模块新增测试：解锁成功时 Ready 只触发一次、网络启动一次；重复解锁不会再次启动。
  - 在 `src-tauri/crates/uc-core/src/setup/state_machine.rs` 的测试集中新增用例，确保解锁事件→Ready transition 恒成立。
  - 运行 `cargo test -p uc-app app_lifecycle` 与 `cargo test -p uc-core setup_state_machine`，记录日志。
    **Must NOT do**: 未通过测试前不修改实现。
    **Recommended Agent Profile**: category "unspecified-high", skills [`project-onboarding`,`test-driven-development`].
    **Parallelization**: 与任务 3 并行（实现需等待测试完成）。
    **References**: `src-tauri/crates/uc-app/src/usecases/app_lifecycle/mod.rs`, `src-tauri/crates/uc-core/src/setup/state_machine.rs`.
    **Acceptance Criteria**:
  - 新测试覆盖 Ready & 网络启动顺序，初始状态 FAIL，实现在任务 3 通过。
  - 命令：
    ```bash
    (cd src-tauri && cargo test -p uc-app app_lifecycle)
    (cd src-tauri && cargo test -p uc-core setup_state_machine)
    ```
  - 记录输出文件 `.sisyphus/evidence/task-2-cargo.log`。

- [x] 3. 删除旧逻辑并统一 Ready 启动
     **What to do**:
  - 根据任务 1 审计结果，删除/重构所有 Ready 前网络启动代码（可能在 `commands/encryption.rs`, `bootstrap/runtime.rs`, 旧 usecase 中）。
  - 确保 Ready 触发顺序：Setup 解锁 → `EnsureReady` → 启动剪贴板 watcher → 启动网络 → 广播 Ready。
  - 保留必要日志/遥测，记录 Ready & 网络启动成功/失败。
    **Must NOT do**: 引入新的启动入口；更改 Setup 状态机顺序。
    **Recommended Agent Profile**: category "unspecified-high", skills [`project-onboarding`].
    **Parallelization**: 依赖任务 2 的测试。
    **References**: `src-tauri/crates/uc-app/src/usecases/app_lifecycle/mod.rs`, `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs`, `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`, `src-tauri/crates/uc-tauri/src/commands/*.rs`.
    **Acceptance Criteria**:
  - 运行任务 2 的测试命令全部 PASS。
  - 代码审查确认仅剩 Ready 路径触发网络。

- [x] 4. 前端/IPC 调整：Setup UI 解耦
     **What to do**:
  - 检查 `src/pages/SetupPage.tsx`, `src/api/setup.ts`, IPC handlers，确保 SetupDoneStep 仅依赖 `getSetupState()` 返回 `Completed`。
  - 若存在网络状态依赖（按钮 disable、toast 等），改为监听 lifecycle Ready 事件或仅提示。
  - 为 `bun test --filter setup-ready-flow`（或新增）写用例，确保 SetupDoneStep 渲染不依赖网络成功。
    **Recommended Agent Profile**: category "visual-engineering", skills [`project-onboarding`].
    **Acceptance Criteria**:
  - `bun test --filter setup-ready-flow` PASS，输出保存 `.sisyphus/evidence/task-4-bun.log`。

- [x] 5. 观察性/文档更新
     **What to do**:
  - 更新相关 README/架构文档（如 `.sisyphus/notepads/...` 或 `docs/`）描述新的启动顺序。
  - 确保 tracing/span 记录 Ready & 网络启动字段，遵循 `AGENTS.md` 的 tracing 规范。
    **Acceptance Criteria**:
  - 文档中描述 Ready→网络启动链路。

- [x] 6. 终验与证据归档
     **What to do**:
  - 按 Verification Strategy 运行全部命令并收集 evidence。
  - 若发现差异，回溯相关任务调整。
    **Acceptance Criteria**:
  - `.sisyphus/evidence/` 中包含所有命令输出/截图；计划中的验收项全部完成。

---

## Commit Strategy

| After Task | Message                                             | Files                           | Verification                                         |
| ---------- | --------------------------------------------------- | ------------------------------- | ---------------------------------------------------- |
| 3          | `fix(app-lifecycle): unify network start at ready`  | uc-app lifecycle & tauri wiring | `cd src-tauri && cargo test -p uc-app app_lifecycle` |
| 4          | `chore(frontend): decouple setup done from network` | frontend setup files            | `bun test --filter setup-ready-flow`                 |

---

## Success Criteria

### Verification Commands

```bash
(cd src-tauri && cargo test -p uc-app app_lifecycle)   # 期望所有 lifecycle/ready 测试通过
(cd src-tauri && cargo test -p uc-core setup_state_machine)   # Setup 状态机验证 Ready 触发
bun test --filter setup-ready-flow   # 前端 SetupDoneStep 解耦验证
```

### Final Checklist

- [x] Ready/网络启动仅有单一触发链路
- [x] Setup UI 与网络状态完全解耦
- [x] 所有自动化测试通过并保存证据
- [x] 文档/日志更新完毕
