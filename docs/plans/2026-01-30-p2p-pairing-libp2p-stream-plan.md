# P2P Pairing over libp2p-stream (Best-Practices) Plan

> 背景：当前新架构的 pairing 仍走 `libp2p::request_response`，并依赖 `session_id -> ResponseChannel` 映射来“伪造会话”。这造成实现复杂、可维护性差、以及行为被 request/response 模型反向约束。
>
> 本计划的目标是：**完全删除 request_response pairing**，改为 **libp2p-stream + 有界 framing + 会话驱动** 的实现，并把网络层对 pairing 的语义收敛成“一个 session 一条 stream”。

## Goals

- 彻底移除 pairing 的 `request_response`（不做旧客户端兼容）。
- 统一网络层通信模型：配对、业务协议均基于 `libp2p_stream`。
- pairing 采用 **session-based stream**：一条 stream 对应一个 `session_id`，在同一条 stream 上双向收发多条消息，完成后关闭。
- 引入 **有界并发/背压策略**（避免无限 spawn、避免 accept backlog 导致 inbound stream 被 drop）。
- 采用 **length-delimited framing**（而不是 JSON Lines），以便设置最大帧大小、防 DoS、避免换行/超长行等边界问题。
- 保持 Hexagonal Architecture 边界：`uc-core` 不依赖 `libp2p`，stream 细节只存在于 `uc-platform`。

## Non-Goals

- 不兼容 legacy pairing（旧版本客户端无法与新版本进行配对）。
- 不在本计划中重做 UI/事件命名（仍通过现有新架构事件边界向上派发）。
- 不在本计划中引入新的加密层（仍使用现有 identity key / short code / pin 等机制）。

## Architecture Constraints (Must Not Violate)

- Respect Hexagonal Architecture: `uc-app -> uc-core <- uc-infra/uc-platform`。
- `uc-core` 不得引入 `libp2p`、`tokio`、`StreamProtocol` 等外部实现细节。
- 不允许 silent failures：任何 stream 错误必须可观测（log + 映射到上层事件/状态机）。
- 不使用 `unwrap()` / `expect()` 于生产代码。
- Rust 相关命令必须在 `src-tauri/` 目录运行。

## Current State (Evidence)

- 新架构 pairing 的 request_response 逻辑与 `ResponseChannel` 映射：
  - `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs`
    - `pairing: request_response::json::Behaviour<PairingMessage, PairingMessage>`
    - `pairing_response_channels: HashMap<session_id, ResponseChannel<...>>`
    - `handle_pairing_event()` 写入 channel；`run_swarm()` 出站时 remove 并优先 `send_response`
- 新架构已经存在 stream 用法（business）：
  - 同文件内 `stream: libp2p_stream::Behaviour`，并使用 `control.accept(...)` + `open_stream(...)`
- 协议 ID 权威来源：
  - `src-tauri/crates/uc-core/src/network/protocol_ids.rs`
    - `ProtocolId::Pairing => "/uc-pairing/1.0.0"`
    - `ProtocolId::Business => "/uc-business/1.0.0"`
- legacy 中已有 length-prefixed framing 的工程化实现可参考（但不直接复用 legacy 模块结构）：
  - `src-tauri/src-legacy/infrastructure/p2p/swarm.rs`（`read_len_prefixed` / `write_len_prefixed`）

## Target Architecture Overview

### High-Level

- 删除 `request_response`。
- 保留并强化 `libp2p_stream::Behaviour`。
- 在 `uc-platform` 内引入一个“Stream Router / Protocol Services”层：
  - 常驻 accept loop：每个协议一个 `IncomingStreams`，持续 poll。
  - 将 `(peer, Stream)` 派发给对应的服务（PairingService / BusinessService / ...）。

### Pairing = Session-Based Stream

- 一条 pairing stream 对应一个 `session_id`。
- 第一帧必须是 `PairingMessage::Request`（用于确定 `session_id` + 绑定 peer）。
- 后续 `Challenge/Response/Confirm/Reject/Cancel/Busy` 都在同一 stream 内往返。
- 进入终态（Paired/Failed/Cancelled）后：
  - 关闭 stream
  - 清理 session 表
  - 取消所有 timers

## Protocol & Framing

### Protocol ID

新增一个明确的 stream 协议 ID，并和现有 `ProtocolId::Pairing`（旧 request_response）区分。

- 新增：`ProtocolId::PairingStream => "/uniclipboard/pairing-stream/1.0.0"`
- 统一命名空间：将 `Business` 也迁移到 `/uniclipboard/business/1.0.0`

**Rationale**：统一使用 `/uniclipboard/*` 命名空间，避免与旧的 `/uc-*` 混淆，便于未来维护和协议发现。

### Frame Encoding (Recommended)

使用 length-delimited framing（u32 big-endian）+ JSON payload。

- Header: `len: u32 BE`
- Payload: `serde_json` 序列化后的 bytes（单帧 = 单条 `PairingMessage`）
- Hard limits:
  - `MAX_PAIRING_FRAME_BYTES = 16KB`（pairing 消息实际 < 1KB，留足余量同时防止 DoS）
  - 若超限：记录 warn + 关闭 stream + 上报 transport error

理由：

- 可以严格限制单条消息大小，避免 JSON Lines 的超长行攻击。
- 允许同一条 stream 传多条消息，不依赖 close 作为边界。
- 更容易做一致的超时与 backpressure 策略。

## Port/API Redesign (Session-Driven)

### Why

若继续保留 `send_pairing_message(peer_id, message)` 的“逐条消息发送”接口，网络层仍需猜测：何时 open/close、同一 session 的后续消息走哪条 stream；这会重新引入 session 粘合逻辑。

### Proposed NetworkPort Changes (uc-core)

将 pairing 发送与生命周期显式化为 session API（接口形态可根据代码风格微调）：

```rust
/// uc-core: src-tauri/crates/uc-core/src/ports/network.rs
#[async_trait::async_trait]
pub trait NetworkPort {
    async fn open_pairing_session(&self, peer_id: String, session_id: String) -> Result<()>;
    async fn send_pairing_on_session(&self, session_id: String, message: PairingMessage) -> Result<()>;
    async fn close_pairing_session(&self, session_id: String, reason: Option<String>) -> Result<()>;

    // 其他协议不在此计划内列出
}
```

### Orchestrator Changes (uc-app)

- `PairingAction::Send { peer_id, message }` 执行时：
  - 从 `message.session_id()` 获取 session_id
  - 若 session 未打开：根据 state/role（Initiator 首包）显式调用 `open_pairing_session(peer, session_id)`
  - 再调用 `send_pairing_on_session(session_id, message)`
- 在进入终态或 timeout 时：调用 `close_pairing_session`（确保资源释放）

**Error Handling**：

- `open_pairing_session` 失败：
  - 记录 error 日志（含 peer_id, session_id）
  - 转换为 `PairingEvent::TransportError`
  - 状态机进入 `Failed` 状态
- `send_pairing_on_session` 失败：
  - 记录 warn 日志
  - 可选重试（由 Orchestrator 状态机决定）
  - 超过重试次数：触发 `close_pairing_session` + 状态机失败
- `close_pairing_session` 失败：
  - 记录 warn（Best-effort cleanup）
  - 不影响状态机终态转换

## uc-platform Implementation Plan

### 1) Behaviour Composition

在 `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs`：

- 从 `Libp2pBehaviour` 删除 pairing 的 request_response behaviour。
- 保留 `stream: libp2p_stream::Behaviour`。

### 2) Stream Router & Services

引入（或在现有 adapter 内组织）如下职责：

- `StreamRouter`：
  - 拿到 `stream::Control`
  - 为每个协议启动常驻 accept loop
  - 将 incoming stream 交给对应 service
- `PairingService`：
  - inbound: 解析 framing，第一帧绑定 session，之后派发消息到 `NetworkEvent::PairingMessageReceived`
  - outbound: 提供 `open_session/send/close` 的具体实现，内部维护 session 表

### 3) Concurrency & Backpressure

必须实现有界并发，原因：

- `libp2p_stream::accept()` 内部零缓冲，若不及时 poll/处理，inbound stream 会被 drop。
- 官方示例也警告“每条 stream 都 spawn 等价于无界缓冲，可能 OOM”。

建议：

- 全局 pairing 并发：`Semaphore(MAX_PAIRING_CONCURRENCY = 16)`（保守初始值，可通过 metrics 监控后调整）
- 每 peer 并发：`HashMap<PeerId, Semaphore(2)>`（允许重试时不阻塞）
- accept loop 只做"接收 + 申请 permit + 交给会话任务"，避免在 accept loop 内长时间 await。
- 增加 metrics 监控：active_sessions, pending_permits, dropped_streams

### 4) Session Task Model

每个 session 一个 task，task 内 split read/write：

- writer 单点写：通过 `mpsc::Receiver<PairingMessage>` mailbox 发送
- reader loop：`read_frame -> decode -> event_tx.send(NetworkEvent::PairingMessageReceived{...})`
- 统一 `tokio::select!` 驱动读写与超时
- 终态/异常：close stream + 从 session 表移除 + 释放 permits

**Timeout Strategy**：

```rust
const FRAME_READ_TIMEOUT: Duration = Duration::from_secs(30);  // 单帧读取超时
const FRAME_WRITE_TIMEOUT: Duration = Duration::from_secs(10); // 单帧写入超时
const SESSION_IDLE_TIMEOUT: Duration = Duration::from_secs(300); // 5分钟无消息交互
```

**Session Cleanup Triggers**：

1. **Graceful Shutdown**：收到终态消息（Paired/Failed/Cancelled）
2. **Idle Timeout**：超过 SESSION_IDLE_TIMEOUT 无消息交互
3. **Forced Cleanup**：
   - Stream 错误（decode 失败、协议违规）
   - 对端关闭 stream
   - Frame 读写超时

### 5) Error Handling & Observability

- 建议每个 session 建立 span：
  - `pairing.session` fields: `peer_id`, `session_id`
- 错误必须：
  - `warn!` 记录（含 peer/session）
  - 映射到上层（至少走 `PairingEvent::TransportError` 的通路）
- Frame decode/encode、超限、session_id mismatch、unexpected first message 都视为协议违规。

## Step-by-Step Execution Plan (TDD-Oriented)

### Phase 0: Preparations

1. 增加新协议 ID
   - Modify: `src-tauri/crates/uc-core/src/network/protocol_ids.rs`
   - 新增 `PairingStream => "/uniclipboard/pairing-stream/1.0.0"`
   - 更新 `Business => "/uniclipboard/business/1.0.0"`
2. 明确 framing 常量与 helper
   - Add: `src-tauri/crates/uc-platform/src/adapters/pairing_stream/framing.rs`
   - 实现 `read_length_prefixed` / `write_length_prefixed`
   - 定义常量：`MAX_PAIRING_FRAME_BYTES`, `FRAME_READ_TIMEOUT`, `FRAME_WRITE_TIMEOUT`
   - 单测 framing（正常帧、超长帧、分片读取、超时）
3. 风险缓解：先在 Business 协议验证 framing
   - 将 framing helper 先集成到现有 Business 协议的测试代码
   - 确认 framing 稳定后再用于 Pairing（降低风险）

### Phase 1: Port / Orchestrator API reshape

1. 调整 `NetworkPort` 接口（session-driven）
   - Modify: `src-tauri/crates/uc-core/src/ports/network.rs`
2. 调整 orchestrator 执行 PairingAction 的逻辑
   - Modify: `src-tauri/crates/uc-app/src/usecases/pairing/orchestrator.rs`
   - 增加错误处理：open/send/close 失败时的状态机转换
3. 调整 uc-tauri wiring（如存在直接调用旧 send_pairing_message）
   - Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`
4. 新增/更新单测：保证 action -> port 调用序列正确
   - 测试用例：
     - `test_open_send_close_sequence`：验证 open -> send -> close 调用序列
     - `test_open_failure_handling`：验证 open 失败时的状态机行为
     - `test_send_failure_handling`：验证 send 失败时的重试/失败逻辑
     - `test_close_on_terminal_state`：验证终态时自动调用 close

### Phase 2: uc-platform pairing stream service

1. 删除 request_response pairing behaviour
   - Modify: `src-tauri/crates/uc-platform/src/adapters/libp2p_network.rs`
2. 实现 accept loop（pairing protocol）
3. 实现 session 表与会话任务（包含超时策略）
4. 实现 outbound：open_pairing_session / send_pairing_on_session / close_pairing_session
5. 集成测试：双 swarm 完整握手
   - 测试用例：
     - `test_pairing_e2e_success`：完整 Request/Challenge/Response/Confirm 流程
     - `test_pairing_timeout`：模拟对方不响应，验证 SESSION_IDLE_TIMEOUT
     - `test_pairing_protocol_violation`：发送非法帧，验证错误处理
     - `test_pairing_concurrent_sessions`：验证并发限制（全局 + per-peer）
     - `test_pairing_graceful_shutdown`：验证 session 清理
6. 性能测试
   - 测试并发限制配置的合理性（16 全局，2 per-peer）
   - 测量平均配对延迟，与 request_response 版本对比
7. 错误注入测试
   - 网络异常模拟（中途断开、高延迟）
   - 资源耗尽模拟（permits 耗尽、channel 满）

### Phase 3: Cleanups & Observability

- 删除 `pairing_response_channels`、相关 command、以及不再使用的 request_response 依赖。
- 更新文档与注释（标记 breaking change）。
- 增加 Monitoring/Metrics：
  - `pairing_active_sessions`（gauge）
  - `pairing_frame_size_bytes`（histogram）
  - `pairing_session_duration_seconds`（histogram）
  - `pairing_errors_total`（counter，按错误类型 label）

## Verification

所有 Rust 命令在 `src-tauri/` 执行：

```bash
cargo test -p uc-core
cargo test -p uc-app
cargo test -p uc-platform
cargo test --workspace
```

（如有 libp2p 相关集成测试超时，需要明确区分：是本次改动引入，还是既有不稳定。）

## Acceptance Criteria

- Pairing 完整流程在两端成功：短码/指纹展示，用户确认后双方持久化 Trusted。
- 配对完成后，pairing session 资源清理：session 表为空、任务退出、timers 取消。
- 协议违规/解析失败/超时：
  - 有明确日志（含 peer_id + session_id）
  - 上层可观测到失败（状态机进入 Failed/Cancelled）
- `uc-core` 无 `libp2p`/stream 依赖；`uc-platform` 承担所有 transport 细节。

## Migration Guide

### API Changes

**旧 API (request_response)**：

```rust
// NetworkPort
async fn send_pairing_message(&self, peer_id: String, message: PairingMessage) -> Result<()>;

// Orchestrator
self.network_port.send_pairing_message(peer_id, message).await?;
```

**新 API (session-driven)**：

```rust
// NetworkPort
async fn open_pairing_session(&self, peer_id: String, session_id: String) -> Result<()>;
async fn send_pairing_on_session(&self, session_id: String, message: PairingMessage) -> Result<()>;
async fn close_pairing_session(&self, session_id: String, reason: Option<String>) -> Result<()>;

// Orchestrator
// 首包（Initiator）
self.network_port.open_pairing_session(peer_id, session_id).await?;
self.network_port.send_pairing_on_session(session_id, message).await?;

// 后续消息
self.network_port.send_pairing_on_session(session_id, message).await?;

// 终态或超时
self.network_port.close_pairing_session(session_id, Some("timeout")).await?;
```

### Breaking Changes

- 旧客户端无法与新客户端进行配对（协议 ID 变更）
- `NetworkPort::send_pairing_message` 方法删除
- `Libp2pBehaviour::pairing` request_response behaviour 删除
- `pairing_response_channels` HashMap 删除

### State Machine Changes

状态机调用模式变化：

- **旧模式**：每条消息独立发送，网络层隐式管理 session
- **新模式**：显式 open/send/close，Orchestrator 管理 session 生命周期

**关键点**：Orchestrator 必须在进入终态时调用 `close_pairing_session`，否则资源泄漏。

## Observability

### Span Naming Conventions

所有 pairing session 相关操作必须使用统一的 span 命名：

```rust
// Session-level span
info_span!("pairing.session", peer_id = %peer_id, session_id = %session_id)

// Operation-level spans (nested)
info_span!("pairing.session.read_frame")
info_span!("pairing.session.write_frame")
info_span!("pairing.session.decode")
```

### Required Log Events

**必须记录的错误**：

```rust
// Frame 超限
warn!("Pairing frame size exceeds limit",
      peer_id = %peer_id,
      session_id = %session_id,
      frame_size = %size,
      max_size = MAX_PAIRING_FRAME_BYTES);

// 协议违规
warn!("Pairing protocol violation",
      peer_id = %peer_id,
      session_id = %session_id,
      error = %error_type);

// Session 超时
warn!("Pairing session timeout",
      peer_id = %peer_id,
      session_id = %session_id,
      timeout_type = %timeout_type); // "idle" | "read" | "write"
```

### Metrics Definitions

| Metric                             | Type      | Labels                                               | Description                     |
| ---------------------------------- | --------- | ---------------------------------------------------- | ------------------------------- |
| `pairing_active_sessions`          | Gauge     | -                                                    | 当前活跃的 pairing session 数量 |
| `pairing_frame_size_bytes`         | Histogram | `direction=inbound/outbound`                         | Pairing 消息帧大小分布          |
| `pairing_session_duration_seconds` | Histogram | `outcome=success/failed/cancelled`                   | Session 持续时间分布            |
| `pairing_errors_total`             | Counter   | `error_type=timeout/protocol_violation/decode_error` | 错误计数                        |
| `pairing_permits_pending`          | Gauge     | `scope=global/peer`                                  | 等待 permit 的请求数            |

## Security Considerations

### DoS Protection Measures

1. **Frame Size Limit**：`MAX_PAIRING_FRAME_BYTES = 16KB`
   - 防止单帧攻击
   - 实际 pairing 消息 < 1KB，16KB 留有足够余量

2. **Concurrency Limits**：
   - 全局：16 并发 session（防止资源耗尽）
   - Per-peer：2 并发 session（防止单个恶意 peer 占用所有资源）

3. **Timeout Protection**：
   - Frame read timeout: 30s（防止慢速攻击）
   - Frame write timeout: 10s（防止写入阻塞）
   - Session idle timeout: 5min（防止僵尸 session）

4. **Protocol Validation**：
   - 第一帧必须是 `PairingMessage::Request`
   - 后续帧的 `session_id` 必须匹配
   - 任何违规立即关闭 stream + 记录 warn

### Resource Limits Summary

| Resource             | Limit | Rationale                         |
| -------------------- | ----- | --------------------------------- |
| Frame size           | 16KB  | 实际消息 < 1KB，防止单帧 DoS      |
| Global concurrency   | 16    | 正常使用场景罕见超过 5 个同时配对 |
| Per-peer concurrency | 2     | 允许重试，防止单 peer 滥用        |
| Session idle timeout | 5min  | 正常配对 < 1min，5min 足够宽松    |
| Frame read timeout   | 30s   | 正常网络 < 5s，30s 应对高延迟     |
| Frame write timeout  | 10s   | 写入通常即时，10s 应对背压        |

## Pre-Implementation Checklist

在开始 Phase 0 前，必须完成以下准备：

- [ ] **确认 Business 协议迁移计划**
  - 决定是否同步将 Business 协议迁移到统一 framing
  - 如果同步迁移，调整 Phase 0 增加 Business framing 测试
  - 如果分阶段迁移，确保两套 framing 实现可共存

- [ ] **Review NetworkPort API 设计**
  - 与团队讨论 session-driven API vs RAII pattern 的权衡
  - 确认 Orchestrator 层是否有能力管理 session 生命周期
  - 评估是否需要增加 `PairingSession` handle 抽象

- [ ] **准备测试环境**
  - 确认集成测试环境支持双 swarm 测试
  - 准备网络模拟工具（延迟、丢包、断线）
  - 设置 CI 超时策略（libp2p 测试可能较慢）

- [ ] **确认依赖版本**
  - 检查 `libp2p-stream` 版本，确认 API 稳定性
  - 确认 `tokio` 版本支持所需的 timeout/select 特性

- [ ] **设置 Metrics 基础设施**
  - 如果项目尚无 metrics，决定使用的库（prometheus-client / metrics crate）
  - 如果已有，确认如何集成 pairing metrics

本改动为 breaking change（不兼容旧客户端）。回滚策略：

- Git revert 回到 request_response pairing 版本。
- 不建议在同一版本内保留“双栈”代码路径（会拖累测试矩阵并隐藏真实问题）。
