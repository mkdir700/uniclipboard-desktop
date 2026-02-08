# Join Space Pairing Flow (Device Select → Global Responder Modal)

## TL;DR

> **Quick Summary**: 在不重构 `DevicesPage` 的前提下，补齐 Join Space 从“选择设备”到“双方短码确认+接收方全局流程完成提示”的完整体验。核心做法是：新增 Setup 状态变更事件（发起方）和 Space Access 完成事件（接收方），前端增加全局 `PairingNotificationProvider` 统一处理 toast + modal。
>
> **Deliverables**:
>
> - 发起方：`ProcessingJoinSpace` 可自动推进到 `JoinSpaceConfirmPeer`
> - 接收方：任意页面收到配对请求 toast，Accept 后立即显示 short code modal
> - 接收方：仅在 space access + persist 完成后显示“完成”
> - 后端：新增 `setup-state-changed` 与 `space-access-completed` 事件链路
>
> **Estimated Effort**: Medium
> **Parallel Execution**: YES - 3 waves
> **Critical Path**: Backend event contracts → Frontend listeners/provider → Integration QA

---

## Context

### Original Request

- 已实现设备发现；现在需要实现：
  1. 发起方选择设备后进入后续流程
  2. 接收方全局 toast 提示
  3. 接收方 Accept 后弹全局 modal 显示 short code，可取消
  4. 接收方完成态必须等待整个 join space 结束（不仅是 pairing）

### Interview Summary

- 你已确认技术方向：
  - 发起方状态同步使用 **Tauri 事件监听**（非轮询）
  - 接收方采用独立 **`PairingNotificationProvider`**（与 `DevicesPage` 独立）
  - toast 采用内置 Accept/Reject 操作
  - Accept 后 modal **直接显示 short code**
  - 接收方完成条件：不是 `p2p-pairing-verification: complete`，而是新增 `space-access-completed`
- 测试策略：后端 Rust tests + Agent-Executed QA

### Research Findings

- Setup 状态机已具备主流程：`ChooseJoinPeer -> ProcessingJoinSpace -> JoinSpaceConfirmPeer` 语义存在
  - `src-tauri/crates/uc-core/src/setup/state_machine.rs`
- `SetupOrchestrator` 异步监听配对事件并 `set_state(JoinSpaceConfirmPeer)`，但前端 SetupPage 无异步订阅
  - `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs`
- 现有 pairing 事件链：
  - request/verification/verifying/complete/failed 已通过 `p2p-pairing-verification` 发给前端
  - `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`
  - `src/api/p2p.ts`
- `PairingPinDialog` 已支持 display/verifying/success 三相 UI，可复用
  - `src/components/PairingPinDialog.tsx`
- SpaceAccess 领域层已区分 `Granted/Denied`，但前端无对应完成事件
  - `src-tauri/crates/uc-core/src/security/space_access/state_machine.rs`
  - `src-tauri/crates/uc-app/src/usecases/space_access/orchestrator.rs`

### Metis Review (Addressed)

- 要求新增会话幂等规则：所有事件必须带 `session_id`，前端仅处理 active session
- 要求锁定范围：**不重构 `DevicesPage`**；不改配对协议语义；只补“可观测 + UI 编排”
- 要求明确完成判定边界：pairing complete != join space complete

---

## Work Objectives

### Core Objective

- 让 Join Space 在发起方和接收方两端都具备可连续、可观察、可取消的交互闭环，并严格区分 pairing 阶段与 space access 阶段完成语义。

### Concrete Deliverables

- 新增后端事件：`setup-state-changed`
- 新增后端事件：`space-access-completed`
- 新增前端 Provider：`PairingNotificationProvider`
- 新增前端 API 监听函数：`onSetupStateChanged`、`onSpaceAccessCompleted`
- `SetupPage` 接入 setup 状态变更事件，自动进入 `PairingConfirmStep`

### Definition of Done

- [x] 发起方在 `JoinPickDeviceStep` 选择设备后，无需手动刷新，可自动显示 short code 确认页
- [x] 接收方在非 `DevicesPage` 页面也能收到 pairing request toast
- [x] 接收方 Accept 后立即弹 modal 显示 short code
- [x] 接收方 modal 仅在 `space-access-completed: success=true` 时进入完成态
- [x] 用户在 modal 点击取消会触发取消配对动作并关闭流程

### Must Have

- 会话隔离：所有前后端事件处理必须基于 `session_id`
- 兼容既有 `p2p-pairing-verification` 事件，不破坏现有 `DevicesPage`
- Hexagonal 边界不破坏（uc-app 不直接依赖 tauri）

### Must NOT Have (Guardrails)

- 不重构 `DevicesPage`
- 不修改 pairing 协议状态机核心语义（仅补事件发射/消费）
- 不把 Tauri 类型下沉到 `uc-core` / `uc-app` 领域逻辑
- 不把 pairing complete 当作 join space complete

---

## Verification Strategy (MANDATORY)

> **UNIVERSAL RULE: ZERO HUMAN INTERVENTION**
>
> 所有验收必须由执行代理通过命令/浏览器自动化完成；禁止“用户手动点击确认”。

### Test Decision

- **Infrastructure exists**: YES
- **Automated tests**: Tests-after
- **Framework**: Rust `cargo test`（后端），前端以 Agent E2E 验证为主

### Agent-Executed QA Scenarios (Global)

Scenario: Initiator auto-transitions to short-code confirmation
Tool: Playwright
Preconditions: 两台实例运行；发起方在 SetupPage Join device list；接收方在线
Steps: 1. 发起方访问 setup 页面并进入 `JoinPickDeviceStep` 2. 点击某设备的 `Select` 按钮 3. 断言页面先出现 processing loading 4. 等待 setup-state-changed 事件触发的 UI 更新 5. 断言出现 short code 文本块（`PairingConfirmStep`）
Expected Result: 无需手动刷新，发起方显示短码确认页
Failure Indicators: 永久停留 loading；没有 short code
Evidence: `.sisyphus/evidence/task-global-initiator-shortcode.png`

Scenario: Responder global toast + modal lifecycle
Tool: Playwright
Preconditions: 接收方当前不在 `/devices`
Steps: 1. 发起方触发 pairing request 2. 接收方断言出现全局 toast（含 Accept/Reject）3. 点击 Accept 4. 断言立即弹出全局 modal，且显示 short code 5. 断言 modal 后续进入 loading，等待 `space-access-completed` 6. 模拟完成事件后断言显示 success
Expected Result: 全流程可见且完成语义正确
Failure Indicators: 无 toast；Accept 后不弹 modal；完成条件提前触发
Evidence: `.sisyphus/evidence/task-global-responder-modal.png`

---

## Execution Strategy

### Parallel Execution Waves

Wave 1 (Start Immediately):

- Task 1: Event contracts + types + guardrails
- Task 2: Backend setup-state-changed emission pipeline

Wave 2 (After Wave 1):

- Task 3: SetupPage initiator event subscription
- Task 4: PairingNotificationProvider toast + accept/reject + modal shell
- Task 5: Backend space-access-completed emission pipeline

Wave 3 (After Wave 2):

- Task 6: Provider completion/cancel semantics and integration wiring
- Task 7: Rust tests + Agent QA scenarios

Critical Path: Task 1 → Task 2 → Task 3 → Task 6 → Task 7

### Dependency Matrix

| Task | Depends On | Blocks  | Can Parallelize With |
| ---- | ---------- | ------- | -------------------- |
| 1    | None       | 2,3,4,5 | 2                    |
| 2    | 1          | 3       | 1                    |
| 3    | 2          | 6       | 4,5                  |
| 4    | 1          | 6       | 3,5                  |
| 5    | 1          | 6       | 3,4                  |
| 6    | 3,4,5      | 7       | None                 |
| 7    | 6          | None    | None                 |

---

## TODOs

- [x] 1. 定义事件契约与会话幂等规则

  **What to do**:
  - 明确新增事件 payload：
    - `setup-state-changed`: `{ state, source, ts }`
    - `space-access-completed`: `{ sessionId, peerId, success, reason?, ts }`
  - 统一前端 TypeScript 类型与监听 API（`src/api/setup.ts`、`src/api/p2p.ts`）
  - 约定 active-session 过滤与重复事件去重规则

  **Must NOT do**:
  - 不修改现有 `p2p-pairing-verification` 字段语义
  - 不引入与本需求无关的全局通知抽象

  **Recommended Agent Profile**:
  - **Category**: `unspecified-low`
    - Reason: 以契约梳理为主，低风险
  - **Skills**: [`writing-plans`, `git-master`]
    - `writing-plans`: 保证契约与验收可执行
    - `git-master`: 保证小步原子提交
  - **Skills Evaluated but Omitted**:
    - `ui-ux-pro-max`: 本任务非视觉实现

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: 2,3,4,5
  - **Blocked By**: None

  **References**:
  - `src/api/p2p.ts:86` - 现有 pairing event kind 枚举，避免破坏兼容
  - `src/api/p2p.ts:96` - 现有 `P2PPairingVerificationEvent` shape，复用命名风格
  - `src/api/setup.ts:13` - `SetupState` TS 定义，`setup-state-changed` 需与其一致
  - `src-tauri/crates/uc-core/src/setup/state.rs:6` - 后端 SetupState 真值结构

  **Acceptance Criteria**:
  - [x] `src/api/setup.ts` 暴露 `onSetupStateChanged` 监听函数与事件类型
  - [x] `src/api/p2p.ts` 暴露 `onSpaceAccessCompleted` 监听函数与事件类型
  - [x] 所有新增事件类型包含 `sessionId` 或可映射会话字段（去重依据）

  **Commit**: YES
  - Message: `impl(setup): define event contracts for async state sync`

- [x] 2. 后端发起方事件链：SetupOrchestrator 发射 `setup-state-changed`

  **What to do**:
  - 在应用层增加 setup 状态变更通知 port（保持 hexagonal 边界）
  - 在 `SetupOrchestrator::dispatch` 每次 `set_state` 后发事件
  - 在 `start_pairing_verification_listener` 异步切换到 `JoinSpaceConfirmPeer` 时同样发事件
  - 在 tauri 适配层实现该 port，并 emit `setup-state-changed`

  **Must NOT do**:
  - 不在 `uc-app` 直接引用 `AppHandle` / tauri 类型
  - 不更改 SetupStateMachine 的 transition 业务语义

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: [`git-master`, `verification-before-completion`]
  - **Skills Evaluated but Omitted**:
    - `frontend-ui-ux`: 非前端任务

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Task 1)
  - **Blocks**: Task 3
  - **Blocked By**: Task 1

  **References**:
  - `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs:158` - dispatch 主循环与 `set_state`
  - `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs:436` - pairing verification 异步监听更新状态
  - `src-tauri/crates/uc-core/src/setup/state_machine.rs:41` - 设备选择进入 processing 的 transition
  - `src-tauri/crates/uc-core/src/ports/mod.rs:61` - ports 暴露位置，新增 port 时保持导出一致
  - `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs:1078` - 现有事件发射集中区，保持事件命名与日志风格一致

  **Acceptance Criteria**:
  - [ ] 触发 `select_device` 后，后端能 emit 至少一次 `setup-state-changed`
  - [ ] `PairingVerificationRequired` 到来时，事件 payload 中 `state` 为 `JoinSpaceConfirmPeer`
  - [ ] Rust tests 覆盖：dispatch 同步状态变更 + 异步 listener 状态变更均有事件发射

  **Commit**: YES
  - Message: `impl(setup): emit setup-state-changed from orchestrator transitions`

- [x] 3. 发起方前端：`SetupPage` 接入 `setup-state-changed` 监听

  **What to do**:
  - 在 `SetupPage` 挂载监听，收到事件后更新 `setupState`
  - 仅在 setup 流程活跃时消费事件，卸载时安全清理
  - 保留原有 `runAction`，事件机制用于补齐异步推进

  **Must NOT do**:
  - 不删除现有 `runAction` 控制流
  - 不改动已稳定的 peer 刷新逻辑

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: [`frontend-ui-ux`, `verification-before-completion`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 6
  - **Blocked By**: Task 2

  **References**:
  - `src/pages/SetupPage.tsx:86` - 当前只在 API 返回时 `setSetupState`
  - `src/pages/SetupPage.tsx:147` - 选择设备触发 `selectJoinPeer`
  - `src/pages/SetupPage.tsx:178` - `JoinSpaceConfirmPeer` 的 UI 渲染入口
  - `src/api/setup.ts:44` - `getSetupState` 与状态类型定义

  **Acceptance Criteria**:
  - [ ] 选择设备后，页面先显示 processing，再自动切到 `PairingConfirmStep`
  - [ ] 事件重复推送不会导致 UI 抖动或无限重渲染
  - [ ] 组件卸载后无悬挂监听（无 console 泄漏警告）

  **Commit**: YES
  - Message: `impl(setup-ui): subscribe setup-state-changed for initiator flow`

- [x] 4. 新增全局 `PairingNotificationProvider`（独立于 DevicesPage）

  **What to do**:
  - 新建 provider，挂载于 `App` 顶层（`Routes` 同层或上层）
  - 监听 `p2p-pairing-verification`
  - `kind=request` 时显示全局 toast（Accept/Reject）
  - Accept 后立即打开 modal，并进入 short-code 展示态（不等待后续阶段）

  **Must NOT do**:
  - 不改动 `DevicesPage.tsx` 的现有配对逻辑
  - 不让 provider 接管设备页内部列表渲染职责

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  - **Skills**: [`frontend-ui-ux`, `verification-before-completion`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 6
  - **Blocked By**: Task 1

  **References**:
  - `src/App.tsx:111` - 顶层路由与 `Toaster` 挂载位置
  - `src/pages/DevicesPage.tsx:52` - 现有 responder 事件消费模式（可复用处理分支）
  - `src/api/p2p.ts:235` - pairing verification 监听 API
  - `src/components/PairingPinDialog.tsx:13` - 可复用 modal 的 phase 机制

  **Acceptance Criteria**:
  - [ ] 在 `/` 页面也能收到 pairing request toast
  - [ ] toast 按钮可触发 `accept_p2p_pairing` / `reject_p2p_pairing`
  - [ ] Accept 后 modal 立刻显示 short code（来自 verification 事件）

  **Commit**: YES
  - Message: `feat(pairing-ui): add global pairing notification provider`

- [x] 5. 后端接收方完成信号：新增 `space-access-completed` 事件

  **What to do**:
  - 在 space-access sponsor 成功持久化后发出 `space-access-completed(success=true)`
  - 在 denied/timeout/cancel/failure 场景发出 `space-access-completed(success=false, reason=...)`
  - 事件 payload 包含 `sessionId` 与 `peerId`（可选但推荐）

  **Must NOT do**:
  - 不复用 `p2p-pairing-verification: complete` 充当最终完成信号
  - 不破坏 `PairingAction::EmitResult` 现有行为

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: [`git-master`, `systematic-debugging`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 6
  - **Blocked By**: Task 1

  **References**:
  - `src-tauri/crates/uc-core/src/security/space_access/state_machine.rs:121` - Sponsor 在 ProofVerified 后进入 Granted 并触发 PersistSponsorAccess
  - `src-tauri/crates/uc-app/src/usecases/space_access/orchestrator.rs:89` - dispatch 获取 next state 的拦截点
  - `src-tauri/crates/uc-app/src/usecases/space_access/orchestrator.rs:209` - PersistSponsorAccess 执行点
  - `src-tauri/crates/uc-core/src/security/space_access/event.rs:50` - AccessGranted/AccessDenied 领域事件定义
  - `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs:1344` - 现有 tauri event emit 风格

  **Acceptance Criteria**:
  - [ ] Sponsor 侧授权成功后，前端能收到 `space-access-completed` 且 `success=true`
  - [ ] Sponsor 侧授权失败后，前端能收到 `space-access-completed` 且 `success=false`
  - [ ] 事件具备可去重字段（sessionId）

  **Commit**: YES
  - Message: `impl(space-access): emit completion event for responder finalization`

- [x] 6. Provider 完整状态机：short-code → loading → success/failure + cancel

  **What to do**:
  - Provider 监听并组合两个事件流：
    - `p2p-pairing-verification`（request/verification/verifying/failed）
    - `space-access-completed`（final)
  - 状态推进：
    - Accept 后：modal 打开并显示 short code
    - 收到 verifying：进入 loading
    - 收到 `space-access-completed(success=true)`：进入 success
    - 收到失败：进入失败态并允许重试或关闭
  - 取消语义：modal cancel 触发取消动作（优先走现有 reject/cancel API）

  **Must NOT do**:
  - 不修改 `DevicesPage` 的事件消费
  - 不在 success 前提前关闭 modal

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  - **Skills**: [`frontend-ui-ux`, `systematic-debugging`]

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 3
  - **Blocks**: Task 7
  - **Blocked By**: Tasks 3,4,5

  **References**:
  - `src/components/PairingPinDialog.tsx:13` - `display/verifying/success` phase 支撑
  - `src/pages/DevicesPage.tsx:70` - verification/verifying/complete/failed 处理顺序
  - `src/api/p2p.ts:192` - reject API
  - `src/api/p2p.ts:221` - accept API
  - `src/api/p2p.ts:178` - verify pin API

  **Acceptance Criteria**:
  - [ ] Accept 后 modal 立即可见 short code
  - [ ] `p2p complete` 不会触发最终成功（必须等 `space-access-completed`）
  - [ ] Cancel 后会话终止，UI 关闭且不残留 active session
  - [ ] 并发 request 时仅处理 active session，其他请求不串台

  **Commit**: YES
  - Message: `feat(pairing-ui): gate responder completion on space-access-completed`

- [x] 7. 验证与回归（Rust tests + Agent QA）

  **What to do**:
  - 增加后端测试：
    - setup-state-changed 发射测试
    - space-access-completed 成功/失败测试
  - 执行 Rust 测试（必须在 `src-tauri/`）
  - 执行前端 Agent QA 场景（playwright/browser automation）

  **Must NOT do**:
  - 不跳过失败用例
  - 不用“手工验证”替代自动验证

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: [`verification-before-completion`, `systematic-debugging`]

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Final
  - **Blocks**: None
  - **Blocked By**: Task 6

  **References**:
  - `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs` - setup orchestrator tests 现有模式
  - `src-tauri/crates/uc-app/src/usecases/space_access/orchestrator.rs` - space access tests 现有 harness
  - `src/App.tsx:111` - provider 与 toaster 集成后的全局观察点

  **Acceptance Criteria**:
  - [ ] `cd src-tauri && cargo test -p uc-app` 全绿
  - [ ] `cd src-tauri && cargo test -p uc-tauri` 全绿（涉及 wiring 事件）
  - [ ] Agent UI 场景覆盖：发起方自动推进、接收方全局 toast、modal 完成门控、取消路径

  **Agent-Executed QA Scenarios**:

  Scenario: Responder cancel from modal aborts pairing
  Tool: Playwright
  Preconditions: 请求已到达并 Accept，modal 正在显示
  Steps: 1. 点击 modal cancel 2. 断言调用取消接口（网络请求或事件）3. 断言 modal 关闭 4. 断言后续 verification/complete 不再更新 UI
  Expected Result: 会话终止且 UI 清理干净
  Evidence: `.sisyphus/evidence/task-7-cancel-abort.png`

  Scenario: Pairing complete does not finish responder flow early
  Tool: Playwright
  Preconditions: 已收到 pairing `complete`，但未收到 space-access-completed
  Steps: 1. 注入/模拟 pairing complete 2. 断言 modal 仍处于 loading（非 success）3. 注入 `space-access-completed(success=true)` 4. 断言切换 success
  Expected Result: 完成门控正确
  Evidence: `.sisyphus/evidence/task-7-completion-gate.png`

  **Commit**: YES
  - Message: `test(join-space): verify event-driven pairing and completion gating`

---

## Commit Strategy

| After Task | Message                                                                | Files                | Verification                     |
| ---------- | ---------------------------------------------------------------------- | -------------------- | -------------------------------- |
| 1          | `impl(setup): define event contracts for async state sync`             | api types            | type-check                       |
| 2          | `impl(setup): emit setup-state-changed from orchestrator transitions`  | uc-app + uc-tauri    | cargo test -p uc-app             |
| 4          | `feat(pairing-ui): add global pairing notification provider`           | frontend provider    | build + QA smoke                 |
| 5          | `impl(space-access): emit completion event for responder finalization` | uc-app/uc-tauri      | cargo test -p uc-app -p uc-tauri |
| 6/7        | `test(join-space): verify event-driven pairing and completion gating`  | tests + QA artifacts | full verification                |

---

## Success Criteria

### Verification Commands

```bash
cd src-tauri && cargo test -p uc-app
cd src-tauri && cargo test -p uc-tauri
```

### Final Checklist

- [x] 发起方选择设备后可自动进入短码确认页
- [x] 接收方任意页面收到配对请求 toast
- [x] Accept 后 modal 立即显示 short code
- [x] 接收方完成态由 `space-access-completed` 驱动
- [x] Cancel 路径可中断会话并正确清理
- [x] `DevicesPage` 无行为回归
