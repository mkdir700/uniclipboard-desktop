# Space Access (New Space Initialize) + Setup Migration Plan

## TL;DR

> **Quick Summary**: 将 Setup 作为一等用例驱动 New Space 初始化的 Space Access 路径，并移除 `is_encryption_initialized` 与 active onboarding 相关代码；Join Space 仅停在 Pairing/设备选择，后续 Space Access (Join) 明确排除。
>
> **Deliverables**:
>
> - Setup → Space Access(NewSpace Initialize) 编排链路对齐（uc-app/uc-core/uc-tauri）
> - `is_encryption_initialized` 迁移/移除与前端替换
> - active onboarding 清理（后端/前端/文档）
> - 失败回滚与错误事件策略（KeyChain/Stronghold）
>
> **Estimated Effort**: Large
> **Parallel Execution**: YES - 2 waves
> **Critical Path**: Task 1 → Task 2 → Task 3 → Task 5

---

## Context

### Original Request

计划 Setup + Space Access（仅 New Space 初始化路径），移除 `is_encryption_initialized` 与 active onboarding 代码；Join Space 的 Space Access 不在范围内。

### Interview Summary

**Key Discussions**:

- Setup 是一等用例（uc-app），前端仅基于 SetupStatus 渲染。
- Space Access 复用 state machine + ports 模式，仅实现 New Space 初始化。
- KeyChain 写入失败为致命阻塞；恢复策略=重试/退出（允许输出错误事件/退出码）。
- 清理范围仅 Active（Legacy/Archive 不动）。
- TDD（表驱动测试优先）。

**Research Findings**:

- Setup 与 Space Access 状态机/动作/端口路径已明确（见 References）。
- SetupOrchestrator 的 dispatch/execute_actions 串行化模式需遵循。
- Stronghold: Argon2 + salt，KDF 32 bytes；`save()` 显式持久化；`destroy()` 会自动保存；回滚建议删除 snapshot 而不保存半成品。
- Keyring 写入失败需原子性回滚（已写条目需删除）。

### Metis Review

**Identified Gaps (addressed)**:

- SetupStatus 前端字段必须完全替换旧字段（已确认）。
- Active-only 清理范围按草稿清单执行（已确认）。
- 失败允许输出上层错误事件/退出码（已确认）。

---

## Work Objectives

### Core Objective

让 Setup 用例独立完成 New Space 初始化的 Space Access 编排，并彻底移除 `is_encryption_initialized` 与 active onboarding 逻辑，同时确保失败可回滚且错误可观测。

### Concrete Deliverables

- Setup → Space Access(NewSpace Initialize) 的完整链路（uc-app/uc-core/uc-tauri）
- 移除 `is_encryption_initialized` 命令/用例/引用
- 删除 active onboarding 后端/前端/文档引用
- 清晰的失败回滚策略（KeyChain + Stronghold）

### Definition of Done

- [x] `rg "is_encryption_initialized" src-tauri src docs --glob '!docs/archive/**' --glob '!docs/plans/archive/**'` 无匹配
- [x] `rg "onboarding" src-tauri src docs --glob '!docs/archive/**' --glob '!docs/plans/archive/**'` 仅剩 Legacy/Archive（Active 清单已清空）
- [x] `cargo test -p uc-core` 通过（在 `src-tauri/` 下执行）
- [x] `cargo test -p uc-app` 通过（在 `src-tauri/` 下执行）

### Must Have

- SetupStatus 前端完全替换旧字段
- KeyChain 写入失败阻塞 Setup；可重试/退出；产生明确错误事件/退出码

### Must NOT Have (Guardrails)

- 不实现 Join Space 的 Space Access（keyset/proof/unlock）
- 不修改 Legacy/Archive 目录
- 不引入 `is_encryption_initialized` 新引用

---

## Verification Strategy (MANDATORY)

> **UNIVERSAL RULE: ZERO HUMAN INTERVENTION**
>
> ALL tasks in this plan MUST be verifiable WITHOUT any human action.
> This is NOT conditional — it applies to EVERY task, regardless of test strategy.

### Test Decision

- **Infrastructure exists**: YES (cargo test + tokio::test + mockall)
- **Automated tests**: TDD (table-driven preferred)
- **Framework**: cargo test (Rust)

### If TDD Enabled

Each TODO follows RED-GREEN-REFACTOR with table-driven tests.

### Agent-Executed QA Scenarios (MANDATORY — ALL tasks)

Use Bash (rg/cargo test) for non-UI verification. All commands must be run from proper directories.

---

## Execution Strategy

### Parallel Execution Waves

Wave 1 (Start Immediately):
├── Task 1: Setup/Space Access NewSpace wiring + tests
└── Task 4: Remove is_encryption_initialized (Rust + Tauri commands)

Wave 2 (After Wave 1):
├── Task 2: KeyChain/Stronghold failure/rollback behavior
└── Task 3: Frontend switch to SetupStatus + onboarding cleanup

Wave 3 (After Wave 2):
└── Task 5: Active onboarding cleanup (backend/infra/docs/tests)

Critical Path: Task 1 → Task 2 → Task 3 → Task 5

---

## TODOs

- [x] 1. Setup → Space Access(NewSpace Initialize) 编排对齐 + TDD

  **What to do**:
  - 在 `SetupOrchestrator` 中将 `CreateEncryptedSpace` 动作改为调用 Space Access 的 NewSpace Initialize 流程（替代旧 `InitializeEncryption`）。
  - 将 `CompleteOnboarding` 语义迁移为 Setup 完成语义（例如 `MarkSetupComplete` 用例），并在 Setup 编排中使用。
  - 在 `uc-app/src/usecases/space_access/orchestrator.rs` 落地 NewSpace Initialize 执行路径（复用 state machine + ports）。
  - 确保 state machine/event/action 只覆盖 NewSpace 初始化所需路径。
  - 新增表驱动测试覆盖 SetupStateMachine 的 NewSpace 路径（RED→GREEN→REFACTOR）。

  **Must NOT do**:
  - 不接入 Join Space 的 Space Access 路径。
  - 不改变现有 Hex 架构边界（uc-core 不依赖外部实现）。

  **Recommended Agent Profile**:
  - **Category**: unspecified-high
    - Reason: 需要跨 uc-core/uc-app/uc-tauri 串联状态机与编排。
  - **Skills**: ["test-driven-development", "systematic-debugging"]
    - `test-driven-development`: 强制 RED-GREEN-REFACTOR，表驱动测试。
    - `systematic-debugging`: 如果状态机路径不通，需系统性定位。
  - **Skills Evaluated but Omitted**:
    - `frontend-ui-ux`: 本任务不涉及 UI 设计。

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Task 4)
  - **Blocks**: Task 2, Task 3, Task 5
  - **Blocked By**: None

  **References**:
  - `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs` - dispatch/execute_actions 串行化模式与动作执行入口。
  - `src-tauri/crates/uc-app/src/usecases/setup/context.rs` - dispatch_lock 语义，保持并发安全。
  - `src-tauri/crates/uc-core/src/setup/state_machine.rs` - Setup 状态机产出 `CreateEncryptedSpace`。
  - `src-tauri/crates/uc-core/src/setup/action.rs` - `SetupAction::CreateEncryptedSpace` 定义。
  - `src-tauri/crates/uc-core/src/security/space_access/state_machine.rs` - Space Access 状态机（表驱动测试模式）。
  - `src-tauri/crates/uc-app/src/usecases/space_access/orchestrator.rs` - Space Access 编排器入口。
  - `src-tauri/crates/uc-app/src/usecases/space_access/executor.rs` - 聚合 ports 的执行层。
  - `src-tauri/crates/uc-core/src/ports/space/*` - Space Access 端口抽象。

  **Acceptance Criteria**:

- [x] 新增表驱动测试覆盖 Setup NewSpace 事件到 `CreateEncryptedSpace` 动作产出
- [x] `cargo test -p uc-core` 通过（在 `src-tauri/` 下执行）
- [x] `cargo test -p uc-app` 通过（在 `src-tauri/` 下执行）

  **Agent-Executed QA Scenarios**:

  Scenario: Setup NewSpace 状态机产出 CreateEncryptedSpace
  Tool: Bash (cargo test)
  Preconditions: 在 `src-tauri/` 目录
  Steps: 1. `cargo test -p uc-core setup_state_machine_new_space`
  Expected Result: 测试通过，断言动作包含 `CreateEncryptedSpace`
  Evidence: `.sisyphus/evidence/task-1-setup-newspace-test.txt`

- [x] 2. KeyChain/Stronghold 失败回滚与错误事件策略落地

  **What to do**:
  - 在 Space Access NewSpace Initialize 过程中加入事务性持久化顺序：验证→KeyChain 写入→snapshot save。
  - KeyChain 写入失败：回滚已写入条目；返回 Setup 可恢复错误状态；允许输出错误事件/退出码。
  - Stronghold snapshot 失败：不调用 `destroy()`，删除 snapshot 文件并回滚状态。
  - 记录错误日志（tracing，结构化字段；禁止日志泄露敏感信息）。

  **Must NOT do**:
  - 不允许部分成功写入导致“半初始化”状态。
  - 不在 uc-core 引入外部实现依赖。

  **Recommended Agent Profile**:
  - **Category**: unspecified-high
    - Reason: 涉及安全/持久化一致性与错误传播。
  - **Skills**: ["systematic-debugging"]
    - `systematic-debugging`: 用于处理失败回滚路径。
  - **Skills Evaluated but Omitted**:
    - `test-driven-development`: 已在 Task 1 覆盖核心 TDD，任务内以新增测试补齐即可。

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Task 3)
  - **Blocks**: Task 3, Task 5
  - **Blocked By**: Task 1

  **References**:
  - `src-tauri/crates/uc-app/src/usecases/space_access/executor.rs` - 写入/持久化的执行层。
  - `src-tauri/crates/uc-core/src/ports/space/persistence.rs` - 持久化端口定义。
  - `src-tauri/crates/uc-core/src/ports/space/crypto.rs` - 密钥派生端口（避免泄露）。
  - `src-tauri/crates/uc-infra/src/network/space/mod.rs` - DenyReason 依赖示例。

  **Acceptance Criteria**:

- [x] KeyChain 写入失败时，已写入条目回滚完成（测试或模拟验证）
- [x] snapshot 失败不触发 `destroy()`，而是删除 snapshot 文件
- [x] 失败路径产生结构化错误事件/退出码

  **Agent-Executed QA Scenarios**:

  Scenario: KeyChain 写入失败回滚
  Tool: Bash (cargo test)
  Preconditions: 在 `src-tauri/` 目录
  Steps: 1. `cargo test -p uc-app space_access_keychain_rollback`
  Expected Result: 失败路径测试通过，断言回滚与错误事件被触发
  Evidence: `.sisyphus/evidence/task-2-keychain-rollback.txt`

- [x] 3. 前端切换到 SetupStatus（完全替换旧字段）

  **What to do**:
  - 前端 API/Context/Store 只使用 SetupStatus（移除旧 onboarding 状态依赖）。
  - `OnboardingPage` 改为纯 SetupStatus 驱动；移除 `completeOnboarding` 调用。
  - `TitleBar`/`App` 等入口移除 onboarding 依赖。

  **Must NOT do**:
  - 不保留旧字段映射或过渡逻辑。

  **Recommended Agent Profile**:
  - **Category**: unspecified-low
    - Reason: 前端是替换/清理型变更。
  - **Skills**: ["test-driven-development"]
    - `test-driven-development`: 如有前端测试，补充最小化验证。
  - **Skills Evaluated but Omitted**:
    - `frontend-ui-ux`: 无 UI 视觉改动。

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Task 2)
  - **Blocks**: Task 5
  - **Blocked By**: Task 1

  **References**:
  - `src/api/onboarding.ts` - 旧 onboarding API 与 setup API 混合处。
  - `src/contexts/OnboardingContext.tsx` - 旧状态提供者。
  - `src/contexts/onboarding-context.ts` - 旧 Context 类型。
  - `src/store/api.ts` - RTK Query onboarding status。
  - `src/pages/OnboardingPage.tsx` - 入口页面逻辑。
  - `src/App.tsx` - 顶层入口切换。
  - `src/components/TitleBar.tsx` - onboarding 状态控制显示。

  **Acceptance Criteria**:
  - [x] `rg "Onboarding" src` 仅剩 Legacy/Archive 或新 Setup 名称
  - [x] `rg "getOnboardingState|initializeOnboarding|completeOnboarding" src` 无匹配

  **Agent-Executed QA Scenarios**:

  Scenario: 前端 onboarding 依赖清理验证
  Tool: Bash (rg)
  Preconditions: 项目根目录
  Steps: 1. `rg "getOnboardingState|initializeOnboarding|completeOnboarding" src`
  Expected Result: 无匹配输出
  Evidence: `.sisyphus/evidence/task-3-frontend-onboarding-rg.txt`

- [x] 4. 移除 `is_encryption_initialized` 用例/命令/引用

  **What to do**:
  - 删除 usecase、runtime accessor、command 与注册项。
  - 替换任何业务判断为 SetupStatus。
  - 更新相关文档（非 archive）。
  - 如涉及 Tauri command 签名调整，确保 `_trace: Option<TraceMetadata>` 与 span/instrument 合规。

  **Must NOT do**:
  - 不修改 legacy/archived 版本的同名命令。

  **Recommended Agent Profile**:
  - **Category**: quick
    - Reason: 以删除/替换为主。
  - **Skills**: ["systematic-debugging"]
    - `systematic-debugging`: 避免遗漏引用导致构建失败。
  - **Skills Evaluated but Omitted**:
    - `test-driven-development`: 以引用清理与编译验证为主。

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Task 1)
  - **Blocks**: Task 5
  - **Blocked By**: None

  **References**:
  - `src-tauri/crates/uc-app/src/usecases/is_encryption_initialized.rs` - usecase 实现。
  - `src-tauri/crates/uc-app/src/usecases/mod.rs` - usecase 导出。
  - `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs` - runtime accessor。
  - `src-tauri/crates/uc-tauri/src/commands/encryption.rs` - command 定义。
  - `src-tauri/src/main.rs` - command 注册。

  **Acceptance Criteria**:

- [x] `rg "is_encryption_initialized" src-tauri src docs` 无匹配

  **Agent-Executed QA Scenarios**:

  Scenario: is_encryption_initialized 引用清零
  Tool: Bash (rg)
  Preconditions: 项目根目录
  Steps: 1. `rg "is_encryption_initialized" src-tauri src docs --glob '!docs/archive/**' --glob '!docs/plans/archive/**'`
  Expected Result: 无匹配输出
  Evidence: `.sisyphus/evidence/task-4-is-encryption-rg.txt`

- [x] 5. Active onboarding 清理（后端/前端/文档/测试）

  **What to do**:
  - 删除 active onboarding 后端模块（uc-core/uc-app/uc-infra/uc-tauri）并更新 wiring。
  - 替换 `SetupOrchestrator` 对 `CompleteOnboarding` 的依赖（迁移到 Setup 语义）。
  - 删除前端 onboarding context、API、测试与文案引用。
  - 更新 active 文档中 onboarding 相关描述（非 archive）。

  **Must NOT do**:
  - 不删除 Legacy/Archive 目录。

  **Recommended Agent Profile**:
  - **Category**: unspecified-high
    - Reason: 涉及多层次清理与 wiring。
  - **Skills**: ["systematic-debugging"]
    - `systematic-debugging`: 防止依赖断裂。
  - **Skills Evaluated but Omitted**:
    - `frontend-ui-ux`: 无 UI 设计变更。

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 3
  - **Blocks**: None
  - **Blocked By**: Task 2, Task 3, Task 4

  **References** (Active-only):
  - `src-tauri/crates/uc-tauri/src/commands/onboarding.rs` - 旧命令入口。
  - `src-tauri/src/main.rs` - 命令注册（onboarding）。
  - `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs` - onboarding usecase accessor。
  - `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs` - onboarding_state 注入。
  - `src-tauri/crates/uc-app/src/usecases/onboarding/*` - 旧用例。
  - `src-tauri/crates/uc-app/src/usecases/mod.rs` - re-export。
  - `src-tauri/crates/uc-app/src/deps.rs` - onboarding_state 依赖。
  - `src-tauri/crates/uc-core/src/onboarding/mod.rs` - domain。
  - `src-tauri/crates/uc-core/src/ports/onboarding.rs` - port。
  - `src-tauri/crates/uc-core/src/ports/mod.rs` - export。
  - `src-tauri/crates/uc-infra/src/onboarding_state.rs` - infra。
  - `src-tauri/crates/uc-infra/src/lib.rs` - export。
  - `src/api/onboarding.ts` - 旧 API 定义。
  - `src/contexts/OnboardingContext.tsx` / `src/contexts/onboarding-context.ts` - 旧 Context。
  - `src/pages/OnboardingPage.tsx` - 页面入口。
  - `src/pages/__tests__/OnboardingFlow.test.tsx` - 前端测试。
  - `docs/plans/2026-01-29-setup-onboarding-ui.md` - active 文档。
  - `docs/guides/error-handling.md` - 文档引用。

  **Acceptance Criteria**:
  - [x] `rg "onboarding" src-tauri src docs` 仅剩 Legacy/Archive
  - [x] `cargo test -p uc-app` 通过（在 `src-tauri/` 下执行）

  **Agent-Executed QA Scenarios**:

  Scenario: Active onboarding 清理验证
  Tool: Bash (rg)
  Preconditions: 项目根目录
  Steps: 1. `rg "onboarding" src-tauri src docs --glob '!docs/archive/**' --glob '!docs/plans/archive/**'`
  Expected Result: 无 active 引用（Legacy/Archive 可保留）
  Evidence: `.sisyphus/evidence/task-5-onboarding-rg.txt`

---

## Commit Strategy

| After Task | Message                                            | Files                               | Verification                                        |
| ---------- | -------------------------------------------------- | ----------------------------------- | --------------------------------------------------- |
| 1          | `feat(setup): wire new-space space-access`         | uc-app/uc-core setup + space_access | `cargo test -p uc-core`                             |
| 2          | `fix(security): add rollback on init failure`      | space_access executor/ports         | `cargo test -p uc-app`                              |
| 3          | `refactor(ui): switch to setup status`             | src/ frontend files                 | `rg "onboarding" src`                               |
| 4          | `refactor(core): remove is_encryption_initialized` | uc-app/uc-tauri                     | `rg "is_encryption_initialized" src-tauri src docs` |
| 5          | `chore(cleanup): remove active onboarding`         | backend+frontend+docs               | `rg "onboarding" src-tauri src docs`                |

---

## Success Criteria

### Verification Commands

```bash
rg "is_encryption_initialized" src-tauri src docs --glob '!docs/archive/**' --glob '!docs/plans/archive/**'
rg "onboarding" src-tauri src docs --glob '!docs/archive/**' --glob '!docs/plans/archive/**'
```

```bash
# run inside src-tauri/
cargo test -p uc-core
cargo test -p uc-app
```

### Final Checklist

- [x] SetupStatus 完全替换旧字段
- [x] New Space 初始化链路可达且可回滚
- [x] `is_encryption_initialized` 无引用
- [x] active onboarding 清理完成，Legacy/Archive 未动
- [x] 所有测试通过
