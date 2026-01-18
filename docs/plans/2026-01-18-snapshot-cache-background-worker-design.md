# Snapshot Cache + Spool + Background Blob Worker（Final Design, Round 2）

- Created: 2026-01-18（基于 2025-01-18 草案迭代）
- Last Updated: 2026-01-18
- Status: Design Proposal（Round 2 Converged / Ready for Implementation）
- Scope: Large payload durability + non-blocking capture + restart recovery + idempotent blob writes
- Non-Goals: Blob 写入不在 capture path；Resolver 不直接写 blob

---

## 1. 问题陈述

在 clipboard materialization 重构中，捕获大内容（例如图片）时，normalize 阶段会丢失原始 bytes，导致后续 ClipboardPayloadResolver.load_raw_bytes() 失败。核心原因是旧实现用 inline_data: Some(vec![]) 作为“占位”，但既不保留 snapshot 原始 bytes，也未及时落 blob。

用户要求：
• capture path 不写 blob（解耦，避免卡顿）
• 后台 worker 异步写 blob
• 最近数据可快速访问（cache）
• 降级可用：用户访问内容不应因为后台延迟而失败（只要 cache/spool 还在）

工程约束（最终版）：
• cache key 必须是 representation_id
• memory cache 必须 bounded（max_entries + max_bytes）
• 必须有 disk spool（崩溃/重启恢复）
• capture path 不允许任何磁盘 I/O（包括 async fs write）
• 并发写必须真正幂等（不能靠 write+delete 清理）
• 状态机必须显式（无空 vec 占位）
• cache 锁模型一致（内部 tokio::sync::Mutex）
• 表状态更新必须事务化/CAS（避免 blob_id 与 state 不一致）

---

## 2. 最终架构总览（唯一架构）

### 2.1 三层可用性

    1.	In-Memory RepresentationCache（bounded）：最近内容快速返回
    2.	Disk Spool（best-effort）：崩溃/重启恢复（明文临时文件 + 强权限）
    3.	Blob Store（最终加密存储）：加密落盘 + 去重

### 2.2 核心原则

    •	Capture 只做：DB 写入元数据 + cache.put + try_send 到队列
    •	Spool 写入只由 SpoolerTask 执行（后台）
    •	Blob 写入只由 BackgroundBlobWorker 执行（后台）
    •	Resolver 只读：Inline/BlobReady 直接返回；Staged 尝试 cache/spool 返回并 re-queue

---

## 3. Domain Model：显式状态机（单一事实来源）

关键点：PayloadAvailability 不携带 bytes/blob_id，仅表达状态。
数据载体分别是 inline_data 与 blob_id，避免“双来源”。

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum PayloadAvailability {
    Inline,       // inline_data=Some, blob_id=None
    BlobReady,    // inline_data=None, blob_id=Some
    Staged,       // inline_data=None, blob_id=None (expect cache/spool)
    Processing,   // optional but recommended
    Failed,       // optional: last_error=Some
    Lost,         // inline_data=None, blob_id=None (cache/spool missing & ttl/attempt exhausted)
}

pub struct PersistedClipboardRepresentation {
    pub id: RepresentationId,
    pub format: String,
    pub inline_data: Option<Vec<u8>>,
    pub blob_id: Option<BlobId>,
    pub payload_state: PayloadAvailability,
    pub last_error: Option<String>, // recommended
    // ... other fields
}
```

### 3.1 不变量（实现必须 enforce）

| payload_state | inline_data | blob_id | 备注                            |
| ------------- | ----------- | ------- | ------------------------------- |
| Inline        | Some        | None    | 小内容                          |
| BlobReady     | None        | Some    | blob 可读                       |
| Staged        | None        | None    | 等 worker；cache/spool 预期存在 |
| Processing    | None        | None    | worker in-flight                |
| Failed        | None        | None    | last_error 必填（建议）         |
| Lost          | None        | None    | 仅在“确定不可恢复”时落入        |

建议：resolver 不要轻易把状态写成 Lost。Lost 更适合由 worker（或后台清理策略）在重试/TTL 结束后判定。

---

## 4. Infrastructure Components

### 4.1 RepresentationCache（内部锁 + &self API）

Location: uc-infra/src/clipboard/representation_cache.rs

```rust
use tokio::sync::Mutex;

pub struct RepresentationCache {
    inner: Mutex<Inner>,
}

struct Inner {
    entries: std::collections::HashMap<RepresentationId, CacheEntry>,
    queue: std::collections::VecDeque<RepresentationId>,
    max_entries: usize,
    max_bytes: usize,
    current_bytes: usize,
}

struct CacheEntry {
    raw_bytes: Vec<u8>,
    status: CacheEntryStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CacheEntryStatus {
    Pending,
    Processing,
    Completed,
}
```

行为契约：
• bounded：max_entries + max_bytes
• eviction：优先淘汰 Completed 的 oldest；必要时可淘汰更旧 Pending（需明确定义策略）
• 所有方法 &self，内部 tokio::sync::Mutex，严禁持锁 await

---

### 4.2 SpoolManager（磁盘明文临时存储 + 强权限 + 清理策略）

Location: uc-infra/src/clipboard/spool_manager.rs

强制安全策略：
• spool_dir 必须位于 OS cache dir（避免备份/同步）
• dir 权限：0700（尽可能）
• file 权限：0600（Unix）；Windows 使用用户 profile 目录默认 ACL（文档需说明）
• 明确威胁模型：不对抗 root/管理员；目标是最小化同机其他用户/进程读取风险

```rust
pub struct SpoolManager {
    spool_dir: std::path::PathBuf,
    max_bytes: usize,
}
pub struct SpoolEntry {
    pub representation_id: RepresentationId,
    pub file_path: std::path::PathBuf,
    pub size: usize,
}
```

清理策略（建议写入实现约束）：
• 成功落 blob：立即删除 spool
• 失败：保留 N 天（配置），超过 TTL 清理
• 超上限：按最旧优先删除（LRU/mtime）

“secure delete（覆盖写）”在 SSD 上不可保证，建议作为可选项，不要在文档里承诺“安全擦除”。

---

### 4.3 SpoolerTask（关键：保证 capture path 非阻塞）

Location: uc-infra/src/clipboard/spooler_task.rs

```rust
pub struct SpoolRequest {
    pub rep_id: RepresentationId,
    pub bytes: Vec<u8>,
}

pub struct SpoolerTask {
    spool_rx: tokio::sync::mpsc::Receiver<SpoolRequest>,
    spool_manager: std::sync::Arc<SpoolManager>,
}
```

契约：
• Capture 只能 spool_tx.try_send(req)，绝不 await。
• 队列满：允许降级为 cache-only（必须 warn + metric）
• 队列关闭：视为严重错误（任务死亡）

---

### 4.4 BackgroundBlobWorker（唯一 blob 写入路径）

Location: uc-infra/src/clipboard/background_blob_worker.rs

Message type: RepresentationId

核心依赖：
• cache（Arc）
• spool（Arc）
• repo（ClipboardRepresentationRepositoryPort）
• blob writer（BlobWriterPort）
• encryption（EncryptionPort）
• hasher（ContentHashPort）
• retry/backoff 配置

---

## 5. Ports / Repository Contracts（必须按语义实现）

> 约定：所有 Port trait 定义在 `uc-core`（或至少在 core 侧），`uc-infra`/`uc-platform` 仅提供实现。

### 5.1 BlobWriterPort：幂等必须在存储层原子化

```rust
#[async_trait::async_trait]
pub trait BlobWriterPort: Send + Sync {
    /// 原子语义：若 content_id 已存在，返回既有 BlobId；否则写入并返回新 BlobId
    async fn write_if_absent(
        &self,
        content_id: &ContentHash,
        encrypted_bytes: &[u8],
    ) -> anyhow::Result<BlobId>;
}
```

关键说明：
• 不允许上层做 write() + delete() 清理竞争者（风险极高）
• content_id 建议默认使用 keyed hash（避免内容指纹泄露）

---

### 5.2 ClipboardRepresentationRepositoryPort：事务化 + CAS 更新 blob_id 与状态

```rust
#[async_trait::async_trait]
pub trait ClipboardRepresentationRepositoryPort: Send + Sync {
    /// 原子更新：仅当当前状态属于 expected_states 时，更新 blob_id + payload_state (+ last_error)
    async fn update_processing_result(
        &self,
        rep_id: &RepresentationId,
        expected_states: &[PayloadAvailability], // e.g. [Staged, Processing]
        blob_id: Option<&BlobId>,                // success Some, failure None
        new_state: PayloadAvailability,          // BlobReady / Failed / Lost
        last_error: Option<&str>,
    ) -> anyhow::Result<PersistedClipboardRepresentation>;
}
```

建议实现（SQLite）语义：
• 单 SQL UPDATE 设置 blob_id, payload_state, last_error, updated_at
• WHERE 限定 payload_state IN expected_states
• 返回 updated row，若 0 row 更新则说明状态已被其他线程推进（正常并发语义）

---

## 6. Data Flows（Final）

### 6.1 Capture Flow（严格非阻塞）

    1.	normalize → 生成 PersistedClipboardRepresentation（大 payload：payload_state=Staged, inline_data=None, blob_id=None）
    2.	写 DB（事件 + representations 元信息）
    3.	对每个大 payload：
    •	cache.put(rep_id, bytes)（内存）
    •	spool_tx.try_send(SpoolRequest{rep_id, bytes})（best-effort）
    •	worker_tx.try_send(rep_id)（best-effort）

capture 不允许执行任何 fs write，不允许 await “等队列腾位置”。

---

### 6.2 Worker Flow（cache→spool fallback）

对每个 rep_id：1.（可选）repo CAS：Staged -> Processing（提高可观测性）2. bytes 获取：
• 先 cache.get
• cache miss 再 spool.read（基于 rep_id 定位 entry）

    3.	计算 content_id = hasher.hash_keyed(raw_bytes)（建议）
    4.	encrypted = encryption.encrypt(raw_bytes, aad(rep_id))
    5.	blob_id = blob_writer.write_if_absent(content_id, encrypted)
    6.	repo CAS 事务更新：(Staged|Processing) -> BlobReady，同时写入 blob_id
    7.	删除 spool entry（best-effort），并 cache.mark_completed / cache.remove（按策略）

失败路径：
• transient：按 backoff retry（次数上限配置）
• fatal：repo 更新为 Failed（写 last_error），保留 spool（可配置 TTL）

---

### 6.3 Resolver Flow（只读 + re-queue，不写 blob）

Location: uc-infra/src/clipboard/payload_resolver.rs

行为：
• Inline：直接返回 inline_data
• BlobReady：读 blob → decrypt → 返回
• Staged/Processing/Failed：尝试 cache/spool 读取 bytes，若拿到则立刻返回，并 worker_tx.try_send(rep_id)（优先提示）；如果拿不到则返回错误（或根据策略映射为 Lost）
• Lost：返回“不可恢复”错误

Resolver 不包含 blob_writer/hasher 依赖，不重复实现 worker 写入逻辑。

---

## 7. Backpressure（行为契约，必须一致）

| 队列      | Full（队列满）                           | Closed（任务死亡） | 影响               |
| --------- | ---------------------------------------- | ------------------ | ------------------ |
| spool_tx  | 跳过 spool（cache-only），warn + metric  | error（严重）      | 影响重启恢复能力   |
| worker_tx | 允许降级：不立即排队（用户访问时再触发） | error（严重）      | 影响 blob 最终落盘 |

---

## 8. Spool 安全政策（必须写入实现）

    •	位置：OS cache dir（macOS ~/Library/Caches，Linux ~/.cache，Windows %LOCALAPPDATA% 之类）
    •	权限：dir 0700，file 0600（尽可能；Windows 说明依赖 ACL）
    •	内容：明文（可选 future：轻量加密）
    •	生命周期：
    •	成功：立刻删
    •	失败：保留 TTL（默认 7 天）
    •	超额：按最旧删除

---

## 9. Testing Strategy（与最终架构一致）

必须覆盖：
• capture 不阻塞：压力下不出现明显延迟（可用基准/统计）
• spooler backpressure：队列满时降级策略符合表格契约
• restart recovery：spool 扫描 + worker 能补齐落 blob
• 幂等：并发 worker + 多次 re-queue 下只产生一个 blob_id（由 write_if_absent 保证）
• 状态一致性：任何时刻不出现 blob_id=Some 但 payload_state != BlobReady 的非法组合

---

## 10. Implementation Checklist（Final）

Phase 0（必须先做，避免跑偏）
• 统一 PayloadAvailability 为 state-only enum（无 bytes/blob_id）
• PersistedClipboardRepresentation 增加 last_error（建议）
• RepresentationCache 改为内部 Mutex + &self API
• 引入 SpoolerTask，capture 只 try_send
• BlobWriterPort::write_if_absent（原子语义）
• repo 方法统一为 update_processing_result（事务化 + CAS）

Phase 1-…（按需拆）
• Worker：cache→spool→write_if_absent→CAS 更新→cleanup
• Resolver：只读 + re-queue
• 启动恢复：扫描 spool entries 并 enqueue 到 worker
• 完整测试矩阵 + stress test（100 张大图 burst）

---

## 11. Bootstrap / Wiring（Tauri & Runtime）

原则：
• 所有后台任务（SpoolerTask / BackgroundBlobWorker）必须在应用启动阶段完成 wiring，并持有可观测的生命周期句柄（JoinHandle + ShutdownToken）。
• 任何可能被 `tauri::State<T>` 访问的类型必须在启动前通过 `.manage()` 注册（避免运行期拿不到状态）。
• 后台任务不得“静默死亡”：panic/错误必须 `tracing::error!` 并上报到上层（例如 PlatformEvent / HealthState）。

建议落点：
• `uc-tauri`：在 bootstrap/wiring 里创建并 `.manage()` 共享组件（cache、spool_manager、worker_tx 等），在 runtime 启动时 spawn 任务。
• `uc-platform`：负责把 clipboard watcher、worker、spooler 的事件流接入平台运行时（统一 cancel/shutdown）。

---

## 12. Config Knobs（建议显式化）

建议作为配置项（可先 hardcode，但必须集中在一处，便于后续迁移到 config）：
• RepresentationCache：`max_entries`、`max_bytes`、eviction 策略
• SpoolManager：`spool_dir`（默认 OS cache dir）、`max_bytes`、`ttl_days`
• SpoolerTask：队列容量（`spool_queue_capacity`）
• BackgroundBlobWorker：队列容量（`worker_queue_capacity`）、并发度、retry 次数与 backoff
