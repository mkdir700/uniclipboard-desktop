# Draft: Join Space - Device Selection → Pairing Confirmation Flow

## Requirements (confirmed)

- 用户选择发现的设备后，触发配对流程
- 接收方显示 toast 提示有配对请求
- 用户点击 toast，跳转到 device page，显示 short code
- 发起方显示 short code，提示用户确认码一致

## Research Findings

### 现有系统状态

**前端**:

- `SetupPage.tsx` 已有完整的状态机渲染：`JoinPickDeviceStep` → `PairingConfirmStep` → `JoinVerifyPassphraseStep`
- `selectJoinPeer(peerId)` API 已定义，调用后端 `select_device` 命令
- `PairingConfirmStep` 组件已实现，显示 `shortCode` + `peerFingerprint` + 确认按钮
- `DevicesPage.tsx` 已完整实现接收方的配对请求处理（toast + accept/reject + PinDialog）

**后端**:

- `SetupStateMachine::transition` 完整实现：
  - `JoinSpaceSelectDevice` + `ChooseJoinPeer` → `ProcessingJoinSpace` (Action: `EnsurePairing`)
  - 异步：`PairingVerificationRequired` 事件 → 设置状态为 `JoinSpaceConfirmPeer { short_code, peer_fingerprint }`
  - `JoinSpaceConfirmPeer` + `ConfirmPeerTrust` → `JoinSpaceInputPassphrase` (Action: `ConfirmPeerTrust`)
- `SetupOrchestrator` 完整实现：
  - `ensure_pairing_session()`: 调用 `pairing_orchestrator.initiate_pairing()` + 启动 verification listener
  - `confirm_peer_trust_action()`: 调用 `pairing_orchestrator.user_accept_pairing()`
  - `start_pairing_verification_listener()`: 监听 `PairingVerificationRequired` 事件并更新 Setup 状态
- `wiring.rs` 中 `run_pairing_event_loop` 和 `run_pairing_action_loop` 已实现：
  - 接收 `PairingMessage::Request` → 发出 `p2p-pairing-verification` (kind: `request`) 事件
  - `PairingAction::ShowVerification` → 发出 `p2p-pairing-verification` (kind: `verification`) 事件

**关键发现 - 两套独立的配对系统**:

1. **Setup 流程**: `SetupPage` → `SetupOrchestrator` → `PairingOrchestrator` (发起方通过 Setup 状态机驱动)
2. **Device 页面流程**: `DevicesPage` → `PairingDialog` → `PairingOrchestrator` (独立的配对 UI，支持发起方+接收方)

用户点击 `select` 按钮后，前端调用 `selectJoinPeer(peerId)` → 后端 `select_device` → 编排器 `dispatch(ChooseJoinPeer)` → 状态转 `ProcessingJoinSpace` → 执行 `EnsurePairing` → 发起配对请求

### 接收方视角（关键需求）

**现有机制**:

- `wiring.rs` 收到 `PairingMessage::Request` 后立即 emit `p2p-pairing-verification` (kind: `request`)
- `DevicesPage.tsx` 监听这个事件并显示配对请求（在 "Pairing Requests" 区域）
- 但 **没有全局 toast 通知**！DevicesPage 必须处于前台才能看到请求

**缺口**:

- 需要一个 **全局** 的 toast 通知，无论用户在哪个页面都能看到
- toast 需要有 "查看" 按钮跳转到 DevicesPage
- 接收方点击 Accept 后，后端生成 short code，通过 `p2p-pairing-verification` (kind: `verification`) 事件发给前端
- `PairingPinDialog` 弹出显示 short code

### 发起方视角

**已实现但需验证**:

- 发起方调用 `selectJoinPeer` 后，前端进入 `ProcessingJoinSpace` loading 状态
- 后端 `start_pairing_verification_listener` 异步等待 `PairingVerificationRequired` 事件
- 收到事件后，SetupOrchestrator 将状态更新为 `JoinSpaceConfirmPeer { short_code, peer_fingerprint }`
- 前端 `SetupPage.tsx` 检测到状态变化，渲染 `PairingConfirmStep` 显示 short code

**问题**: SetupPage 如何感知后端状态异步变化？

- 当前只有 `runAction` 等待 API 返回新状态
- `selectJoinPeer` 返回的是 `ProcessingJoinSpace`，不是 `JoinSpaceConfirmPeer`
- **需要轮询或事件监听**来获取后续状态变化！

## Technical Decisions

1. **发起方状态同步**: Tauri 事件监听 - 后端 Setup 状态变化时 emit `setup-state-changed` 事件，前端 SetupPage listen 并更新
2. **接收方全局通知**: 独立的 `PairingNotificationProvider` 组件 - 挂载在 App 顶层，专门处理配对请求 toast 通知
3. **toast 库**: 使用已有的 sonner（项目已集成）

## Open Questions

1. SetupPage 如何感知后端异步状态变化？目前只在 API 调用时更新状态
2. 接收方全局 toast 应该在哪里实现？App.tsx 级别？
3. toast 跳转到 DevicesPage 时，是否需要携带 session_id 参数？
4. 发起方在 ProcessingJoinSpace loading 期间，是否需要超时处理？

## Scope Boundaries

- INCLUDE:
  - 发起方点击设备后进入 loading → 收到 short code → 显示确认
  - 接收方全局 toast → 跳转 device page → accept → 显示 short code
- EXCLUDE:
  - passphrase 输入步骤（已实现）
  - space access 协议（已实现）
  - 配对拒绝/取消的边缘情况（现有逻辑已处理）
