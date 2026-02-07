# Setup::JoinSpace — 实现加入已有空间流程

## TL;DR

> **Quick Summary**: 为 SetupOrchestrator 补齐 Join Space 路径的 5 个 Action 实现（EnsureDiscovery / EnsurePairing / ConfirmPeerTrust / AbortPairing / StartJoinSpaceAccess），将现有的 PairingOrchestrator 和 SpaceAccessOrchestrator 接入 Setup 编排，并补齐 SpaceAccessOrchestrator 内尚未实现的 Joiner 侧 Action（RequestSpaceKeyDerivation / SendProof / PersistJoinerAccess），使 Join Space 端到端可运行。
>
> **Deliverables**:
>
> - SetupOrchestrator 的 5 个 JoinSpace Action handler 完整实现
> - SpaceAccessOrchestrator 的 Joiner 侧 Action 补齐（RequestSpaceKeyDerivation / SendProof / PersistJoinerAccess）
> - SpaceAccessOrchestrator 的 Sponsor 侧 Action 补齐（SendOffer / SendResult / PersistSponsorAccess）
> - SpaceAccessExecutor 扩展以包含 SpaceAccessTransportPort 和 ProofPort 依赖
> - SetupOrchestrator 新增依赖注入（PairingOrchestrator / SpaceAccessOrchestrator / DiscoveryPort）及 bootstrap 接线更新
> - 修复 VerifyPassphrase → SubmitPassphrase 事件映射不一致
> - Rust 单元/集成测试覆盖 happy path 与失败回退
> - 前端 SetupPage 轻量修补
>
> **Estimated Effort**: Large
> **Parallel Execution**: YES - 4 waves
> **Critical Path**: Task 1 → Task 2 → Task 4 → Task 7 → Task 8

---

## Context

### Original Request

实现 Setup 的第二条主路径——Join Existing Space，使一台新设备能通过已有成员的协作完成：设备发现 → 配对 → 口令验证 → 空间密钥获取 → Setup Completed。

### Interview Summary

**Key Discussions**:

- Setup 是一等业务概念（Phase），不是 UI 或技术动作；NewSpace 已实现，JoinSpace 是另一条互斥路径。
- 状态机（uc-core）已完整定义 JoinSpace 分支，前端 SetupPage.tsx 已按状态驱动渲染所有 Join 子视图。
- Orchestrator（uc-app）5 个 Action 全部返回 `ActionNotImplemented`；底层能力 Pairing / SpaceAccess / Discovery 已有骨架。
- `is_encryption_initialized` 标记为待移除，但本任务聚焦于 JoinSpace Action 落地；该概念的完全清理可单独收尾。
- 测试策略：实现后补测试。

**Research Findings**:

- PairingOrchestrator（`uc-app/src/usecases/pairing/orchestrator.rs`）已具备 `initiate_pairing` / `user_accept_pairing` / `user_reject_pairing`。
- SpaceAccessOrchestrator（`uc-app/src/usecases/space_access/orchestrator.rs`）已实现 `RequestOfferPreparation` / `StartTimer` / `StopTimer`；但 `RequestSpaceKeyDerivation` / `SendProof` / `SendResult` / `PersistJoinerAccess` / `PersistSponsorAccess` 仍 TODO。`SendOffer` 仅打印 warn 日志。
- SpaceAccessExecutor（`uc-app/src/usecases/space_access/executor.rs`）包含 `crypto: &dyn CryptoPort`, `net: &dyn NetworkPort`, `timer: &dyn TimerPort`, `store: &dyn PersistencePort`；但缺少 `SpaceAccessTransportPort`（已定义于 `uc-core/src/ports/space/transport.rs`）和 `ProofPort`（已定义于 `uc-core/src/ports/space/proof.rs`）的引用。
- Discovery 已有 `list_discovered_peers` / `get_p2p_peers` command。
- SpaceAccess 协议定义完整（state / event / action state_machine），Joiner 侧需实现接收 Offer → 提交 passphrase → 等待裁决。
- **关键发现**：SetupOrchestrator 的 `verify_passphrase()` 方法派发 `VerifyPassphrase` 事件，但状态机在 `JoinSpaceInputPassphrase` 状态只匹配 `SubmitPassphrase` 事件。`VerifyPassphrase` 会落入 invalid transition。需要修复事件映射。
- **关键发现**：Sponsor 侧 `SendResult`、`PersistSponsorAccess`、`SendOffer` 也是 TODO。端到端 JoinSpace 要求 Sponsor 能发送裁决结果，否则 Joiner 永远等不到 `AccessGranted`。必须纳入 scope。

### Metis Review

**Identified Gaps** (addressed):

- SpaceAccessOrchestrator 内部 Joiner 侧 Action 也需补齐（不止 SetupOrchestrator 层面）→ 已纳入 Task 3。
- SetupOrchestrator 需要注入新依赖（PairingOrchestrator / SpaceAccessOrchestrator / DiscoveryPort）→ 已纳入 Task 1。
- 同设备中途切换 peer / Pairing 成功但 SpaceAccess 失败等边缘 case → 已纳入 guardrails 与 Task 4。
- 前端 PairingConfirmStep `onConfirm` 目前错误地调用 `cancelSetup()` → 已纳入 Task 5。

### Momus Review (Round 1)

**Blocking Issues** (all addressed):

1. **SpaceAccessExecutor 缺少 TransportPort 和 ProofPort**：`executor.net` 是 `&dyn NetworkPort`（通用网络 port），但 `SpaceAccessTransportPort`（`uc-core/src/ports/space/transport.rs`）和 `ProofPort`（`uc-core/src/ports/space/proof.rs`）已在 uc-core 中定义却未接入 executor → 已纳入 Task 1a（扩展 SpaceAccessExecutor）。
2. **Sponsor 侧 Action 缺失导致端到端不可完成**：`SendResult`、`PersistSponsorAccess`、`SendOffer` 仍 TODO；Joiner 等待 `AccessGranted` 事件需要 Sponsor 发送裁决 → 已纳入 Task 3（与 Joiner 侧合并为"补齐 SpaceAccess 所有未实现 Action"）。
3. **VerifyPassphrase 事件映射错误**：`SetupOrchestrator::verify_passphrase()` 派发 `VerifyPassphrase` 事件，但状态机 `JoinSpaceInputPassphrase` 只匹配 `SubmitPassphrase`。`VerifyPassphrase` 落入 invalid transition → 已纳入 Task 5a（修复事件映射，让 `verify_passphrase()` 派发 `SubmitPassphrase`）。

---

## Work Objectives

### Core Objective

让 SetupOrchestrator 的 Join Space 路径从 `ActionNotImplemented` 变为端到端可执行：用户选择 Join → 发现并选择 peer → 配对 → 验证 passphrase → 获得 Space 密钥 → Setup Completed。

### Concrete Deliverables

- `uc-app/src/usecases/setup/orchestrator.rs`：5 个 Action handler 完整实现 + `verify_passphrase()` 事件映射修复
- `uc-app/src/usecases/space_access/orchestrator.rs`：所有未实现 Action 补齐（Joiner: RequestSpaceKeyDerivation/SendProof/PersistJoinerAccess；Sponsor: SendOffer/SendResult/PersistSponsorAccess）
- `uc-app/src/usecases/space_access/executor.rs`：扩展以包含 `SpaceAccessTransportPort` 和 `ProofPort`
- `uc-tauri/src/bootstrap/runtime.rs`：`build_setup_orchestrator` 更新注入关系
- `SetupOrchestrator::new()` 签名扩展，接受新依赖
- Rust 测试文件：orchestrator happy path / failure path
- 前端 PairingConfirmStep 修正

### Definition of Done

- [x] `cd src-tauri && cargo test` 全量通过，无新 warning
- [x] `cd src-tauri && cargo test setup::orchestrator` 含 join_space happy path 测试且 PASS
- [x] `cd src-tauri && cargo test space_access::orchestrator` 含 joiner 侧测试且 PASS
- [x] `cargo check --workspace` 通过，无编译错误
- [x] Setup 状态机无任何修改（`uc-core/src/setup/state_machine.rs` 无 diff）

### Must Have

- SetupOrchestrator 的 5 个 JoinSpace Action 可执行，不再返回 ActionNotImplemented
- SpaceAccess 所有 Action 可执行（Joiner: RequestSpaceKeyDerivation/SendProof/PersistJoinerAccess；Sponsor: SendOffer/SendResult/PersistSponsorAccess）
- SpaceAccessExecutor 包含 `transport: &dyn SpaceAccessTransportPort` 和 `proof: &dyn ProofPort`
- `verify_passphrase()` 正确派发 `SubmitPassphrase` 事件（不再派发 `VerifyPassphrase`）
- bootstrap 正确注入新依赖，运行时可构建
- dispatch 锁序不变（dispatch_lock → state）

### Must NOT Have (Guardrails)

- ❌ 不修改 `uc-core/src/setup/state_machine.rs`（状态机纯函数已定义，无需变动）
- ❌ 不修改 `uc-core/src/setup/event.rs`、`state.rs`、`action.rs`（域层不变）
- ❌ 不改造 Pairing 协议或 Discovery 服务（只消费其 API）
- ❌ 不扩展 SpaceAccess 状态机（`uc-core/src/security/space_access/state_machine.rs` 无 diff）
- ❌ 不从 orchestrator 直接访问 infra 适配器（必须通过 port/usecase）
- ❌ 不在 UI 中引入对 `is_encryption_initialized` 的新调用
- ❌ 前端不做大规模重构，仅允许 Join 流程直接相关的小调整
- ❌ 生产代码不使用 `unwrap()` / `expect()`

---

## Verification Strategy

> **UNIVERSAL RULE: ZERO HUMAN INTERVENTION**
>
> ALL tasks in this plan MUST be verifiable WITHOUT any human action.

### Test Decision

- **Infrastructure exists**: YES — `cargo test` 已有大量 Rust 测试
- **Automated tests**: YES (tests-after)
- **Framework**: `cargo test`（Rust 原生）

### Agent-Executed QA Scenarios (MANDATORY — ALL tasks)

**Verification Tool by Deliverable Type:**

| Type        | Tool                                      | How Agent Verifies                 |
| ----------- | ----------------------------------------- | ---------------------------------- |
| Rust code   | Bash (`cd src-tauri && cargo check/test`) | Compile + test pass                |
| Frontend    | Bash (`bun run build`)                    | Build succeeds without type errors |
| Integration | Bash (`cd src-tauri && cargo test`)       | Full test suite green              |

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately):
├── Task 1: Expand SetupOrchestrator deps + constructor
└── Task 1a: Expand SpaceAccessExecutor with TransportPort + ProofPort

Wave 2 (After Task 1 + 1a):
├── Task 2: Implement EnsureDiscovery + EnsurePairing + ConfirmPeerTrust + AbortPairing
├── Task 3: Implement ALL SpaceAccess unimplemented Actions (Joiner + Sponsor)
├── Task 5: Frontend PairingConfirmStep fix (independent)
└── Task 5a: Fix verify_passphrase event mapping (independent)

Wave 3 (After Tasks 2 + 3):
├── Task 4: Implement StartJoinSpaceAccess (bridges Setup → SpaceAccess)
└── Task 7: Write tests (after Task 4)

Wave 4 (After all):
└── Task 8: Bootstrap wiring update + cargo check/test verification
```

### Dependency Matrix

| Task | Depends On     | Blocks       | Can Parallelize With |
| ---- | -------------- | ------------ | -------------------- |
| 1    | None           | 2, 3, 4, 8   | 1a                   |
| 1a   | None           | 3, 8         | 1                    |
| 2    | 1              | 4            | 3, 5, 5a             |
| 3    | 1, 1a          | 4            | 2, 5, 5a             |
| 4    | 2, 3           | 7, 8         | 5, 5a                |
| 5    | None           | 8            | 2, 3, 4, 5a          |
| 5a   | None           | 8            | 2, 3, 4, 5           |
| 7    | 4              | 8            | 5, 5a                |
| 8    | 1, 4, 5, 5a, 7 | None (final) | None                 |

### Agent Dispatch Summary

| Wave | Tasks       | Recommended Agents                                                              |
| ---- | ----------- | ------------------------------------------------------------------------------- |
| 1    | 1, 1a       | dispatch 2 parallel: unspecified-low × 2                                        |
| 2    | 2, 3, 5, 5a | dispatch 4 parallel: deep×2 + quick×2                                           |
| 3    | 4, 7        | sequential: deep → unspecified-high                                             |
| 4    | 8           | delegate_task(category="quick", load_skills=["verification-before-completion"]) |

---

## TODOs

- [x] 1. Expand SetupOrchestrator dependencies and constructor

  **What to do**:
  - 在 `SetupOrchestrator` struct 中新增字段：
    - `pairing_orchestrator: Arc<PairingOrchestrator>` — 配对编排器
    - `space_access_orchestrator: Arc<SpaceAccessOrchestrator>` — 空间访问编排器
    - `discovery_port: Arc<dyn DiscoveryPort>` — 设备发现（可复用 `list_discovered_peers` 背后的 port，或定义新的 trait 来抽象 mDNS peer list 查询）
  - 扩展 `SetupOrchestrator::new()` 签名以接收新参数
  - 为 pairing session 跟踪新增 `pairing_session_id: Arc<Mutex<Option<String>>>` 字段
  - 确保所有现有调用点（`runtime.rs:build_setup_orchestrator`）能传入新依赖（可先传 placeholder/todo stub，在 Task 7 正式接线）
  - 如果现有 crate 中没有合适的 `DiscoveryPort` trait，则在 `uc-core/src/ports/` 下新增一个 trait 定义（极简：`async fn list_discovered_peers() -> Result<Vec<DiscoveredPeer>>`）
  - 更新所有现有测试中的 mock 构造

  **Must NOT do**:
  - 不修改状态机逻辑
  - 不实现 Action handler（Task 2/3/4 的职责）

  **Recommended Agent Profile**:
  - **Category**: `unspecified-low`
    - Reason: 主要是 struct 字段新增 + 签名变更，逻辑简单
  - **Skills**: [`test-driven-development`]
    - `test-driven-development`: 确保编译通过且现有测试不破

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 1a)
  - **Parallel Group**: Wave 1
  - **Blocks**: Tasks 2, 3, 4, 8
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs:37-68` — 现有 SetupOrchestrator struct 和 `new()` 签名；按此模式新增字段
  - `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs:246-300` — 测试中的 mock 构造方式，新增依赖后 mock 需同步更新

  **API/Type References**:
  - `src-tauri/crates/uc-app/src/usecases/pairing/orchestrator.rs:91-106` — PairingOrchestrator struct 定义（Clone, 包含 sessions/device_repo/action_tx 等）
  - `src-tauri/crates/uc-app/src/usecases/space_access/orchestrator.rs:35-39` — SpaceAccessOrchestrator struct 定义
  - `src-tauri/crates/uc-core/src/network/mod.rs` — `DiscoveredPeer` 类型定义

  **Bootstrap References**:
  - `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs:137-181` — `build_setup_orchestrator` 函数；新依赖在此注入

  **Acceptance Criteria**:
  - [ ] `cd src-tauri && cargo check --workspace` 编译通过
  - [ ] `cd src-tauri && cargo test -p uc-app` 现有测试仍 PASS
  - [ ] `SetupOrchestrator::new()` 签名已包含 `pairing_orchestrator` / `space_access_orchestrator` / `discovery_port` 参数

  **Agent-Executed QA Scenarios:**

  ```
  Scenario: Workspace compiles after dependency expansion
    Tool: Bash
    Steps:
      1. cd src-tauri && cargo check --workspace
      2. Assert: exit code 0, no errors
    Expected Result: Compilation succeeds

  Scenario: Existing setup tests still pass
    Tool: Bash
    Steps:
      1. cd src-tauri && cargo test -p uc-app setup::orchestrator -- --nocapture
      2. Assert: output contains "test result: ok"
    Expected Result: All existing tests green
  ```

  **Commit**: YES
  - Message: `arch: expand SetupOrchestrator to accept pairing and space-access dependencies`
  - Files: `uc-app/src/usecases/setup/orchestrator.rs`, `uc-core/src/ports/` (if new trait), orchestrator test mocks
  - Pre-commit: `cd src-tauri && cargo check --workspace`

---

- [x] 1a. Expand SpaceAccessExecutor with TransportPort and ProofPort

  **What to do**:
  - 在 `SpaceAccessExecutor` struct（`uc-app/src/usecases/space_access/executor.rs`）中新增两个字段：
    - `transport: &'a dyn SpaceAccessTransportPort` — 已定义于 `uc-core/src/ports/space/transport.rs`
    - `proof: &'a dyn ProofPort` — 已定义于 `uc-core/src/ports/space/proof.rs`
  - 当前 executor 的 `net: &'a dyn NetworkPort` 保留不变（供 pairing 层面使用），新增字段用于 SpaceAccess 协议消息传输和 proof 构建/验证
  - 更新所有 `SpaceAccessExecutor` 的构造点，传入新依赖（可先传 placeholder/stub）
  - 更新 `SpaceAccessOrchestrator` 中调用 executor 的方式
  - 更新 `initialize_new_space.rs` 中构造 executor 的代码

  **Must NOT do**:
  - 不修改 `uc-core` 中的 trait 定义
  - 不实现 Action handler（Task 3 的职责）

  **Recommended Agent Profile**:
  - **Category**: `unspecified-low`
    - Reason: 主要是 struct 字段新增 + 签名变更，逻辑简单
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 1)
  - **Parallel Group**: Wave 1
  - **Blocks**: Tasks 3, 8
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `src-tauri/crates/uc-app/src/usecases/space_access/executor.rs:1-9` — 现有 SpaceAccessExecutor struct 定义
  - `src-tauri/crates/uc-app/src/usecases/space_access/initialize_new_space.rs:25-80` — 构造 executor 的代码

  **API/Type References**:
  - `src-tauri/crates/uc-core/src/ports/space/transport.rs:1-9` — `SpaceAccessTransportPort` trait（`send_offer`, `send_proof`, `send_result`）
  - `src-tauri/crates/uc-core/src/ports/space/proof.rs:1-23` — `ProofPort` trait（`build_proof`, `verify_proof`）

  **Acceptance Criteria**:
  - [ ] `SpaceAccessExecutor` struct 包含 `transport` 和 `proof` 字段
  - [ ] `cd src-tauri && cargo check --workspace` 编译通过
  - [ ] 现有 SpaceAccess 测试仍 PASS

  **Agent-Executed QA Scenarios:**

  ```
  Scenario: Workspace compiles after executor expansion
    Tool: Bash
    Steps:
      1. cd src-tauri && cargo check --workspace
      2. Assert: exit code 0, no errors
    Expected Result: Compilation succeeds
  ```

  **Commit**: YES
  - Message: `arch: expand SpaceAccessExecutor with TransportPort and ProofPort dependencies`
  - Files: `uc-app/src/usecases/space_access/executor.rs`, `uc-app/src/usecases/space_access/initialize_new_space.rs`
  - Pre-commit: `cd src-tauri && cargo check --workspace`

---

- [x] 2. Implement EnsureDiscovery / EnsurePairing / ConfirmPeerTrust / AbortPairing

  **What to do**:
  - 在 `SetupOrchestrator::execute_actions` 中实现 4 个 Action handler：
    - **EnsureDiscovery**: 调用 discovery port 的 `list_discovered_peers()`（幂等操作）。此动作不阻塞——它负责确保发现机制在运行。如果 discovery 已在运行，则为 no-op。不产生 follow-up event（状态机停留在 `JoinSpaceSelectDevice`，UI 通过前端轮询 `get_p2p_peers` 拿到设备列表）。
    - **EnsurePairing**: 读取 `self.selected_peer_id` 并调用 `pairing_orchestrator.initiate_pairing(peer_id)` 启动配对握手。存储返回的 session_id 到 `self.pairing_session_id`。配对是异步过程——orchestrator 需要订阅 PairingOrchestrator 的事件来得知 short code 生成/配对完成/配对失败，并据此 dispatch follow-up events。当配对产生 short code + fingerprint 后，通过 `context.set_state()` 转入 `JoinSpaceConfirmPeer { short_code, peer_fingerprint }`。
    - **ConfirmPeerTrust**: 读取 `self.pairing_session_id` 并调用 `pairing_orchestrator.user_accept_pairing(&session_id)` 确认信任。配对确认成功后，推进到 `JoinSpaceInputPassphrase` 状态。
    - **AbortPairing**: 如果有活跃的 pairing session，调用 `pairing_orchestrator.user_reject_pairing(&session_id)` 并清理 `selected_peer_id` 和 `pairing_session_id`。如果没有活跃 session 则 no-op。
  - 异步事件桥接：如果需要从 PairingOrchestrator 获取事件回调（short code 产生、pairing 完成等），可通过 `PairingOrchestrator::subscribe_events()` + tokio task 实现，或在 action handler 内部 await pairing 完成。

  **Must NOT do**:
  - 不修改状态机
  - 不修改 PairingOrchestrator 内部逻辑
  - 不直接访问 infra 层

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: 需要理解 PairingOrchestrator 的事件模型并正确桥接到 Setup 状态流
  - **Skills**: [`systematic-debugging`]
    - `systematic-debugging`: 调试异步事件桥接可能出现的竞态或死锁

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 3, Task 5, Task 5a)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 4
  - **Blocked By**: Task 1

  **References**:

  **Pattern References**:
  - `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs:149-164` — `CreateEncryptedSpace` action handler 示例：如何调用 usecase → 推送 follow-up event
  - `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs:113-140` — dispatch 循环：处理 follow-up events

  **API/Type References**:
  - `src-tauri/crates/uc-app/src/usecases/pairing/orchestrator.rs:167-220` — `initiate_pairing(peer_id)` 方法签名与返回 SessionId
  - `src-tauri/crates/uc-app/src/usecases/pairing/orchestrator.rs:91-106` — PairingOrchestrator 的 `event_senders` 字段（事件订阅机制）
  - `src-tauri/crates/uc-core/src/setup/state.rs:21-26` — `JoinSpaceConfirmPeer` 状态定义（需要 `short_code`, `peer_fingerprint`）
  - `src-tauri/crates/uc-core/src/setup/state_machine.rs:40-54` — Join Space 状态转换

  **Acceptance Criteria**:
  - [ ] `EnsureDiscovery` 不再返回 `ActionNotImplemented`
  - [ ] `EnsurePairing` 调用 PairingOrchestrator 的 `initiate_pairing`
  - [ ] `ConfirmPeerTrust` 调用 PairingOrchestrator 的 `user_accept_pairing`
  - [ ] `AbortPairing` 清理 pairing session
  - [ ] `cd src-tauri && cargo check --workspace` 编译通过

  **Agent-Executed QA Scenarios:**

  ```
  Scenario: Workspace compiles with all 4 action handlers
    Tool: Bash
    Steps:
      1. cd src-tauri && cargo check --workspace
      2. Assert: exit code 0
    Expected Result: No compilation errors

  Scenario: No remaining ActionNotImplemented for these 4 actions
    Tool: Bash
    Steps:
      1. grep -n "ActionNotImplemented.*EnsureDiscovery\|ActionNotImplemented.*EnsurePairing\|ActionNotImplemented.*ConfirmPeerTrust\|ActionNotImplemented.*AbortPairing" src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs
      2. Assert: no matches found (exit code 1)
    Expected Result: All 4 stubs replaced with real implementations
  ```

  **Commit**: YES
  - Message: `impl: implement EnsureDiscovery, EnsurePairing, ConfirmPeerTrust, AbortPairing in SetupOrchestrator`
  - Files: `uc-app/src/usecases/setup/orchestrator.rs`
  - Pre-commit: `cd src-tauri && cargo check --workspace`

---

- [x] 3. Implement ALL SpaceAccess unimplemented Actions (Joiner + Sponsor)

  **What to do**:
  - 在 `SpaceAccessOrchestrator::execute_actions` 中补齐所有未实现的 Action：
  - **Joiner 侧**:
    - **RequestSpaceKeyDerivation**: 调用 `executor.crypto.derive_master_key_from_keyslot()` 从 passphrase 派生 master key，然后通过 `executor.proof.build_proof()` 生成 proof artifact。将 proof 存入 context。
    - **SendProof**: 通过 `executor.transport.send_proof(&session_id)` 发送 Joiner 的证明消息给 Sponsor。
    - **PersistJoinerAccess**: 通过 `executor.store.persist_joiner_access(&space_id)` 持久化 Joiner 的空间访问权限。
  - **Sponsor 侧**:
    - **SendOffer**: 通过 `executor.transport.send_offer(&session_id)` 发送 SpaceAccessOffer 给 Joiner（当前只是 `warn!` 日志，需替换为真实实现）。
    - **SendResult**: 通过 `executor.transport.send_result(&session_id)` 发送裁决结果（Granted/Denied）给 Joiner。
    - **PersistSponsorAccess**: 通过 `executor.store.persist_sponsor_access(&space_id, &peer_id)` 持久化 Sponsor 对该 peer 的授权记录。

  **Must NOT do**:
  - 不修改 SpaceAccess 状态机（`uc-core/src/security/space_access/state_machine.rs`）
  - 不修改 SpaceAccess 的 event/state/action 定义
  - 不修改 uc-core 中的 port trait 定义
  - 不直接访问 infra 适配器

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: 涉及密码学操作（KEK 派生、keyslot 解密、proof 生成/验证）和网络传输，需要仔细理解 crypto/transport/proof port 的 API
  - **Skills**: [`systematic-debugging`]
    - `systematic-debugging`: 加密操作和网络传输失败时需要系统性排查

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 2, Task 5, Task 5a)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 4
  - **Blocked By**: Task 1, Task 1a

  **References**:

  **Pattern References**:
  - `src-tauri/crates/uc-app/src/usecases/space_access/orchestrator.rs:96-173` — 现有 `execute_actions`：`RequestOfferPreparation` 已实现为参考模式
  - `src-tauri/crates/uc-app/src/usecases/space_access/initialize_new_space.rs:25-80` — Sponsor 侧如何使用 SpaceAccessExecutor + crypto_factory

  **API/Type References**:
  - `src-tauri/crates/uc-app/src/usecases/space_access/executor.rs:1-9` — SpaceAccessExecutor 结构（crypto / net / timer / store / transport / proof）
  - `src-tauri/crates/uc-core/src/ports/space/transport.rs:1-9` — SpaceAccessTransportPort（send_offer / send_proof / send_result）
  - `src-tauri/crates/uc-core/src/ports/space/proof.rs:1-23` — ProofPort（build_proof / verify_proof）
  - `src-tauri/crates/uc-core/src/ports/space/crypto.rs:1-16` — CryptoPort（derive_master_key_from_keyslot）
  - `src-tauri/crates/uc-core/src/ports/space/persistence.rs:1-12` — PersistencePort（persist_joiner_access / persist_sponsor_access）
  - `src-tauri/crates/uc-core/src/security/space_access/action.rs:1-51` — SpaceAccessAction 枚举完整定义
  - `src-tauri/crates/uc-core/src/security/space_access/domain.rs` — SpaceAccessProofArtifact 定义

  **Acceptance Criteria**:
  - [ ] `RequestSpaceKeyDerivation` 不再返回 `ActionNotImplemented`
  - [ ] `SendProof` 不再返回 `ActionNotImplemented`
  - [ ] `PersistJoinerAccess` 不再返回 `ActionNotImplemented`
  - [ ] `SendOffer` 不再只是 `warn!` 日志
  - [ ] `SendResult` 不再返回 `ActionNotImplemented`
  - [ ] `PersistSponsorAccess` 不再返回 `ActionNotImplemented`
  - [ ] `cd src-tauri && cargo check --workspace` 编译通过

  **Agent-Executed QA Scenarios:**

  ```
  Scenario: SpaceAccess compiles without any ActionNotImplemented
    Tool: Bash
    Steps:
      1. cd src-tauri && cargo check --workspace
      2. grep -n "ActionNotImplemented" src-tauri/crates/uc-app/src/usecases/space_access/orchestrator.rs
      3. Assert: grep finds no matches (exit code 1)
    Expected Result: All stubs replaced with real implementations

  Scenario: SendOffer no longer just warns
    Tool: Bash
    Steps:
      1. grep -n "send_offer is not wired yet" src-tauri/crates/uc-app/src/usecases/space_access/orchestrator.rs
      2. Assert: no matches
    Expected Result: SendOffer replaced with real implementation
  ```

  **Commit**: YES
  - Message: `impl: implement all SpaceAccess actions (Joiner + Sponsor side)`
  - Files: `uc-app/src/usecases/space_access/orchestrator.rs`
  - Pre-commit: `cd src-tauri && cargo check --workspace`

---

- [x] 4. Implement StartJoinSpaceAccess — bridge Setup to SpaceAccess

  **What to do**:
  - 在 `SetupOrchestrator::execute_actions` 的 `SetupAction::StartJoinSpaceAccess` 分支中：
    1. 取出 cached passphrase（`self.take_passphrase().await?`）
    2. 取出 pairing_session_id（`self.pairing_session_id.lock().await`）
    3. 调用 SpaceAccessOrchestrator 启动 Joiner 流程（dispatch `JoinRequested` event）
    4. 等待 SpaceAccess 流程完成
    5. 如果 SpaceAccess 返回 `Granted`：
       - 调用 `self.app_lifecycle.ensure_ready()` 启动 clipboard watcher + network
       - push `SetupEvent::JoinSpaceSucceeded` 作为 follow-up event
    6. 如果 SpaceAccess 返回 `Denied` / `Cancelled`：
       - push `SetupEvent::JoinSpaceFailed { error }` 作为 follow-up event
  - 确保错误处理清理 pairing session 和 passphrase 缓存

  **Must NOT do**:
  - 不修改状态机
  - 不修改 SpaceAccess 状态机

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: 关键集成点，桥接两个编排器 + 异步事件传递
  - **Skills**: [`systematic-debugging`]
    - `systematic-debugging`: 异步编排的集成调试

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 3
  - **Blocks**: Tasks 7, 8
  - **Blocked By**: Tasks 2, 3

  **References**:

  **Pattern References**:
  - `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs:149-164` — `CreateEncryptedSpace` 实现模式：取 passphrase → 调用 usecase → lifecycle ensure_ready → push succeeded event
  - `src-tauri/crates/uc-app/src/usecases/space_access/orchestrator.rs:54-68` — `initialize_new_space` 入口

  **API/Type References**:
  - `src-tauri/crates/uc-core/src/security/space_access/event.rs:8-11` — `JoinRequested` event 定义（需要 `pairing_session_id`, `ttl_secs`）
  - `src-tauri/crates/uc-core/src/setup/error.rs:1-13` — SetupError 枚举
  - `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs:215-226` — `take_passphrase` 和 `split_passphrase` 方法

  **Acceptance Criteria**:
  - [ ] `StartJoinSpaceAccess` 不再返回 `ActionNotImplemented`
  - [ ] Happy path: passphrase 正确 → SpaceAccess Granted → JoinSpaceSucceeded → Completed
  - [ ] Failure path: passphrase 错误 → SpaceAccess Denied → JoinSpaceFailed
  - [ ] `cd src-tauri && cargo check --workspace` 编译通过

  **Agent-Executed QA Scenarios:**

  ```
  Scenario: StartJoinSpaceAccess stub removed
    Tool: Bash
    Steps:
      1. grep -n "ActionNotImplemented.*StartJoinSpaceAccess" src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs
      2. Assert: no matches (exit code 1)
    Expected Result: Stub replaced with real implementation

  Scenario: Compilation passes
    Tool: Bash
    Steps:
      1. cd src-tauri && cargo check --workspace
      2. Assert: exit code 0
    Expected Result: No errors
  ```

  **Commit**: YES
  - Message: `impl: implement StartJoinSpaceAccess bridging Setup to SpaceAccess orchestrator`
  - Files: `uc-app/src/usecases/setup/orchestrator.rs`
  - Pre-commit: `cd src-tauri && cargo check --workspace`

---

- [x] 5. Fix frontend PairingConfirmStep and minor UI adjustments

  **What to do**:
  - 在 `src/pages/SetupPage.tsx` 中修正 `PairingConfirmStep` 的 `onConfirm` 回调：
    - 当前错误：`onConfirm={() => runAction(() => cancelSetup())}` — 确认按钮调用了取消
    - 应改为调用一个 `confirmPeerTrust` API（需新增）
  - 在 `src/api/setup.ts` 新增 `confirmPeerTrust()` wrapper（调用后端 `confirm_peer_trust` command）
  - 在 `uc-tauri/src/commands/setup.rs` 新增 `confirm_peer_trust` command（调用 orchestrator 的确认方法）
  - 在 `SetupOrchestrator` 中新增 `confirm_peer_trust()` 公开方法（dispatch `ConfirmPeerTrust` event）

  **Must NOT do**:
  - 不做 Setup 页面的大规模重构
  - 不引入新的 `is_encryption_initialized` 调用

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 小范围前端修正 + Tauri command 新增
  - **Skills**: [`verification-before-completion`]
    - `verification-before-completion`: 确保前端构建通过

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 2, 3, 4, 5a)
  - **Parallel Group**: Wave 2 (independent)
  - **Blocks**: Task 8
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `src/pages/SetupPage.tsx:168-179` — 当前 PairingConfirmStep 渲染
  - `src/api/setup.ts:60-69` — 现有 API wrapper 模式（selectJoinPeer 为参考）
  - `src-tauri/crates/uc-tauri/src/commands/setup.rs:77-99` — select_device command 模式

  **Acceptance Criteria**:
  - [ ] `PairingConfirmStep.onConfirm` 不再调用 `cancelSetup()`
  - [ ] 新增 `confirmPeerTrust` API function
  - [ ] 新增 `confirm_peer_trust` Tauri command
  - [ ] `bun run build` 前端构建通过

  **Agent-Executed QA Scenarios:**

  ```
  Scenario: Frontend builds without errors
    Tool: Bash
    Steps:
      1. bun run build
      2. Assert: exit code 0
    Expected Result: Build succeeds

  Scenario: PairingConfirmStep no longer calls cancelSetup for confirm
    Tool: Bash
    Steps:
      1. grep -A2 "onConfirm=" src/pages/SetupPage.tsx
      2. Assert: output does NOT contain "cancelSetup"
    Expected Result: Confirm button wired to correct action
  ```

  **Commit**: YES
  - Message: `fix: wire PairingConfirmStep onConfirm to confirmPeerTrust instead of cancelSetup`
  - Files: `src/pages/SetupPage.tsx`, `src/api/setup.ts`, `uc-tauri/src/commands/setup.rs`, `uc-app/src/usecases/setup/orchestrator.rs`
  - Pre-commit: `bun run build`

---

- [x] 5a. Fix verify_passphrase event mapping (VerifyPassphrase → SubmitPassphrase)

  **What to do**:
  - **Problem**: `SetupOrchestrator::verify_passphrase()` 方法（`orchestrator.rs:96-101`）派发 `SetupEvent::VerifyPassphrase` 事件。但状态机在 `JoinSpaceInputPassphrase` 状态只匹配 `SetupEvent::SubmitPassphrase`（`state_machine.rs:69`）。`VerifyPassphrase` 会落入 invalid transition（`state_machine.rs:101-104`），导致 AbortPairing 被错误触发。
  - **Fix**: 修改 `SetupOrchestrator::verify_passphrase()` 方法，使其派发 `SetupEvent::SubmitPassphrase` 而非 `SetupEvent::VerifyPassphrase`。这样前端调用 `verifyPassphrase(passphrase)` 时，orchestrator 会正确进入 `ProcessingJoinSpace` 状态并触发 `StartJoinSpaceAccess` action。
  - **Alternative considered**: 不修改 uc-core 状态机（guardrail），也不引入新的状态机事件。让 orchestrator 层负责映射。
  - 同时评估 `SetupEvent::VerifyPassphrase` 变体是否还有其他用途。如果只在 `verify_passphrase()` 中使用，考虑在 `capture_context` 中将其映射为 `SubmitPassphrase`（保持 API 不变但正确触发状态机转换）。

  **Must NOT do**:
  - 不修改状态机（`uc-core/src/setup/state_machine.rs`）
  - 不修改事件定义（`uc-core/src/setup/event.rs`）
  - 不改变前端 API 或 Tauri command 签名

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 单行修复 + 验证
  - **Skills**: [`verification-before-completion`]
    - `verification-before-completion`: 确保修复正确且不破坏现有流程

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 2, 3, 4, 5)
  - **Parallel Group**: Wave 2 (independent)
  - **Blocks**: Task 8
  - **Blocked By**: None

  **References**:

  **Code References**:
  - `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs:96-101` — 当前 `verify_passphrase()` 方法，派发 `VerifyPassphrase`
  - `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs:204-210` — `capture_context` 中 `VerifyPassphrase` 的处理
  - `src-tauri/crates/uc-core/src/setup/state_machine.rs:69-74` — `JoinSpaceInputPassphrase + SubmitPassphrase → StartJoinSpaceAccess`
  - `src-tauri/crates/uc-core/src/setup/state_machine.rs:101-104` — invalid transition fallback
  - `src-tauri/crates/uc-core/src/setup/event.rs:15` — `VerifyPassphrase` 事件定义

  **Acceptance Criteria**:
  - [ ] `verify_passphrase()` 派发 `SubmitPassphrase` 事件（不再派发 `VerifyPassphrase`）
  - [ ] `cd src-tauri && cargo check --workspace` 编译通过
  - [ ] 现有测试仍 PASS
  - [ ] 在 `JoinSpaceInputPassphrase` 状态调用 `verify_passphrase()` 后，状态转为 `ProcessingJoinSpace`

  **Agent-Executed QA Scenarios:**

  ```
  Scenario: verify_passphrase dispatches SubmitPassphrase
    Tool: Bash
    Steps:
      1. grep -A3 "pub async fn verify_passphrase" src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs
      2. Assert: output contains "SubmitPassphrase" (not "VerifyPassphrase")
    Expected Result: Event mapping corrected

  Scenario: Compilation passes
    Tool: Bash
    Steps:
      1. cd src-tauri && cargo check --workspace
      2. Assert: exit code 0
    Expected Result: No errors
  ```

  **Commit**: YES
  - Message: `fix: map verify_passphrase to SubmitPassphrase event for correct JoinSpace state transition`
  - Files: `uc-app/src/usecases/setup/orchestrator.rs`
  - Pre-commit: `cd src-tauri && cargo check --workspace`

---

- [x] 7. Write tests for JoinSpace happy path and failure path

  **What to do**:
  - 在 `uc-app/src/usecases/setup/orchestrator.rs` 的 `#[cfg(test)] mod tests` 中新增：
    - **join_space_happy_path**: 模拟 Welcome → StartJoinSpace → ChooseJoinPeer → (mock pairing short code) → ConfirmPeerTrust → (mock pairing success) → SubmitPassphrase → (mock SpaceAccess Granted) → Completed
    - **join_space_pairing_fails**: 模拟配对失败 → JoinSpaceFailed
    - **join_space_passphrase_wrong**: 模拟口令错误 → SpaceAccess Denied → JoinSpaceFailed
    - **join_space_cancel_during_pairing**: 模拟用户取消 → AbortPairing → Welcome
  - 在 `uc-app/src/usecases/space_access/orchestrator.rs` 的测试模块中新增：
    - **joiner_side_happy_path**: JoinRequested → OfferAccepted → PassphraseSubmitted → AccessGranted
    - **joiner_side_denied**: JoinRequested → OfferAccepted → PassphraseSubmitted → AccessDenied
    - **sponsor_side_happy_path**: SponsorAuthorizationRequested → ProofVerified → Granted (with SendResult + PersistSponsorAccess)
    - **sponsor_side_denied**: SponsorAuthorizationRequested → ProofRejected → Denied (with SendResult)
  - 所有测试使用 Mock port/orchestrator，不依赖真实网络或加密

  **Must NOT do**:
  - 不使用 `unwrap()` 在生产代码中（测试中允许）
  - 不引入外部测试框架

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: 需要构造复杂的 mock 链来模拟多步异步流程
  - **Skills**: [`test-driven-development`]
    - `test-driven-development`: 测试编写专家

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 3 (after Task 4)
  - **Blocks**: Task 8
  - **Blocked By**: Task 4

  **References**:

  **Pattern References**:
  - `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs:246-500` — 现有测试模块：MockSetupStatusPort / NoopEncryption 等 mock 构建模式
  - `src-tauri/crates/uc-core/src/setup/state_machine.rs:109-327` — 状态机 table-driven 测试模式

  **Acceptance Criteria**:
  - [ ] `cd src-tauri && cargo test -p uc-app setup::orchestrator -- --nocapture` 输出 `test result: ok`
  - [ ] 至少 4 个新 join space 测试用例
  - [ ] `cd src-tauri && cargo test -p uc-app space_access::orchestrator -- --nocapture` 输出 `test result: ok`
  - [ ] 至少 4 个新 SpaceAccess 测试用例（含 joiner + sponsor）

  **Agent-Executed QA Scenarios:**

  ```
  Scenario: Setup orchestrator join tests pass
    Tool: Bash
    Steps:
      1. cd src-tauri && cargo test -p uc-app setup::orchestrator -- --nocapture
      2. Assert: output contains "test result: ok"
    Expected Result: All join space tests green

  Scenario: SpaceAccess joiner tests pass
    Tool: Bash
    Steps:
      1. cd src-tauri && cargo test -p uc-app space_access::orchestrator -- --nocapture
      2. Assert: output contains "test result: ok"
    Expected Result: Joiner-side tests green
  ```

  **Commit**: YES
  - Message: `test: add JoinSpace happy path and failure tests for Setup and SpaceAccess orchestrators`
  - Files: `uc-app/src/usecases/setup/orchestrator.rs`, `uc-app/src/usecases/space_access/orchestrator.rs`
  - Pre-commit: `cd src-tauri && cargo test -p uc-app`

---

- [x] 8. Update bootstrap wiring and final verification

  **What to do**:
  - 更新 `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs` 的 `build_setup_orchestrator` 函数：
    - 获取或构造 `PairingOrchestrator` 实例（可能已在 `AppRuntime` 中存在，复用之）
    - 构造 `SpaceAccessOrchestrator` 实例
    - 获取 discovery port 的适配器实例
    - 将三个新依赖传入 `SetupOrchestrator::new()` 的扩展签名
  - 确保 `AppDeps` 或 `AppRuntime` 中有所有必要的 port 实例
  - 运行完整 `cargo test` + `bun run build`

  **Must NOT do**:
  - 不在 bootstrap 中加入业务逻辑
  - 不修改 wiring 的职责边界

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 纯 wiring 工作
  - **Skills**: [`verification-before-completion`]
    - `verification-before-completion`: 最终验证

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 4 (final)
  - **Blocks**: None
  - **Blocked By**: Tasks 1, 1a, 4, 5, 5a, 7

  **References**:

  **Pattern References**:
  - `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs:137-181` — 现有 `build_setup_orchestrator`

  **Documentation References**:
  - `src-tauri/crates/uc-tauri/src/bootstrap/README.md` — Bootstrap 边界约束

  **Acceptance Criteria**:
  - [ ] `cd src-tauri && cargo check --workspace` 编译通过
  - [ ] `cd src-tauri && cargo test` 全量 PASS
  - [ ] `bun run build` 前端构建通过
  - [ ] `build_setup_orchestrator` 中 SetupOrchestrator::new() 接收所有新依赖
  - [ ] SpaceAccessExecutor 在 bootstrap 中正确接收 TransportPort 和 ProofPort
  - [ ] 无 JoinSpace 相关 `ActionNotImplemented` 残留
  - [ ] 无 SpaceAccess 相关 `ActionNotImplemented` 残留

  **Agent-Executed QA Scenarios:**

  ```
  Scenario: Full cargo test suite passes
    Tool: Bash
    Steps:
      1. cd src-tauri && cargo test 2>&1
      2. Assert: output contains "test result: ok"
      3. Assert: exit code 0
    Expected Result: All tests green

  Scenario: Frontend builds cleanly
    Tool: Bash
    Steps:
      1. bun run build
      2. Assert: exit code 0
    Expected Result: No errors

  Scenario: No ActionNotImplemented stubs remain for JoinSpace
    Tool: Bash
    Steps:
      1. grep -rn "ActionNotImplemented.*EnsureDiscovery\|ActionNotImplemented.*EnsurePairing\|ActionNotImplemented.*ConfirmPeerTrust\|ActionNotImplemented.*AbortPairing\|ActionNotImplemented.*StartJoinSpaceAccess" src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs
      2. Assert: no matches
      3. grep -rn "ActionNotImplemented" src-tauri/crates/uc-app/src/usecases/space_access/orchestrator.rs
      4. Assert: no matches
    Expected Result: All JoinSpace and SpaceAccess stubs replaced
  ```

  **Commit**: YES
  - Message: `impl: wire PairingOrchestrator and SpaceAccessOrchestrator into SetupOrchestrator bootstrap`
  - Files: `uc-tauri/src/bootstrap/runtime.rs`
  - Pre-commit: `cd src-tauri && cargo test`

---

## Commit Strategy

| After Task | Message                                                                          | Files                                         | Verification              |
| ---------- | -------------------------------------------------------------------------------- | --------------------------------------------- | ------------------------- |
| 1          | `arch: expand SetupOrchestrator to accept pairing and space-access dependencies` | orchestrator.rs, ports/                       | `cargo check --workspace` |
| 1a         | `arch: expand SpaceAccessExecutor with TransportPort and ProofPort dependencies` | executor.rs, initialize_new_space.rs          | `cargo check --workspace` |
| 2          | `impl: implement EnsureDiscovery, EnsurePairing, ConfirmPeerTrust, AbortPairing` | orchestrator.rs                               | `cargo check --workspace` |
| 3          | `impl: implement all SpaceAccess actions (Joiner + Sponsor side)`                | space_access/orchestrator.rs                  | `cargo check --workspace` |
| 4          | `impl: implement StartJoinSpaceAccess bridging Setup to SpaceAccess`             | orchestrator.rs                               | `cargo check --workspace` |
| 5          | `fix: wire PairingConfirmStep onConfirm to confirmPeerTrust`                     | SetupPage.tsx, setup.ts, setup.rs             | `bun run build`           |
| 5a         | `fix: map verify_passphrase to SubmitPassphrase for correct state transition`    | orchestrator.rs                               | `cargo check --workspace` |
| 7          | `test: add JoinSpace happy path and failure tests`                               | orchestrator.rs, space_access/orchestrator.rs | `cargo test -p uc-app`    |
| 8          | `impl: wire pairing and space-access into setup bootstrap`                       | runtime.rs                                    | `cargo test`              |

---

## Success Criteria

### Verification Commands

```bash
cd src-tauri && cargo check --workspace   # Expected: 0 errors
cd src-tauri && cargo test                # Expected: all tests pass
bun run build                             # Expected: build succeeds
```

### Final Checklist

- [x] All "Must Have" present
- [x] All "Must NOT Have" absent
- [x] All tests pass (existing + new)
- [x] No `ActionNotImplemented` for JoinSpace-related actions in SetupOrchestrator
- [x] No `ActionNotImplemented` for any actions in SpaceAccessOrchestrator
- [x] `SendOffer` no longer just warns
- [x] `verify_passphrase()` correctly maps to `SubmitPassphrase`
- [x] SpaceAccessExecutor includes TransportPort and ProofPort
- [x] SetupStateMachine has zero diff (uc-core unchanged)
- [x] SpaceAccess state_machine has zero diff (uc-core unchanged)
- [x] Bootstrap correctly wires all new dependencies
- [x] PairingConfirmStep UI bug fixed
