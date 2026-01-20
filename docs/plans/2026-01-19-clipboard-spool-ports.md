# Clipboard Cache/Spool Ports Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将表示缓存与假脱机队列抽象为 `uc-core` 端口，消除 `uc-app` 对 `uc-infra` 的依赖，同时采用可等待的背压队列语义。

**Architecture:** 在 `uc-core` 新增 `RepresentationCachePort` 与 `SpoolQueuePort`（含 `SpoolRequest`），`uc-infra` 提供实现（内存缓存 + mpsc 队列适配器）。`uc-app` 仅依赖端口并在用例中等待入队，错误日志与上抛保证可观测性。

**Tech Stack:** Rust, async-trait, tokio mpsc, Hexagonal Architecture

---

### Task 1: 在 uc-core 定义新的端口与请求类型

**Files:**

- Create: `src-tauri/crates/uc-core/src/ports/clipboard/representation_cache.rs`
- Create: `src-tauri/crates/uc-core/src/ports/clipboard/spool_queue.rs`
- Modify: `src-tauri/crates/uc-core/src/ports/clipboard/mod.rs`

**Step 1: 写一个失败的编译性测试（引用新类型）**

在 `src-tauri/crates/uc-core/src/ports/clipboard/spool_queue.rs` 里添加：

```rust
#[cfg(test)]
mod tests {
    use super::SpoolRequest;
    use crate::ids::RepresentationId;

    #[test]
    fn spool_request_is_clone() {
        let req = SpoolRequest {
            rep_id: RepresentationId::new(),
            bytes: vec![1, 2, 3],
        };
        let _clone = req.clone();
    }
}
```

**Step 2: 运行测试确认失败**

Run: `cargo test -p uc-core spool_request_is_clone`
Expected: FAIL（类型/模块未定义）

**Step 3: 添加端口与请求类型**

在 `representation_cache.rs` 定义 `RepresentationCachePort`（async trait，方法：`put/get/mark_completed/mark_spooling/remove`）。
在 `spool_queue.rs` 定义：

```rust
#[derive(Debug, Clone)]
pub struct SpoolRequest {
    pub rep_id: RepresentationId,
    pub bytes: Vec<u8>,
}

#[async_trait::async_trait]
pub trait SpoolQueuePort: Send + Sync {
    async fn enqueue(&self, request: SpoolRequest) -> anyhow::Result<()>;
}
```

在 `ports/clipboard/mod.rs` 中 `mod` 并 `pub use` 新模块。

**Step 4: 运行测试确认通过**

Run: `cargo test -p uc-core spool_request_is_clone`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-core/src/ports/clipboard/representation_cache.rs \
        src-tauri/crates/uc-core/src/ports/clipboard/spool_queue.rs \
        src-tauri/crates/uc-core/src/ports/clipboard/mod.rs

git commit -m "feat(core): add clipboard cache and spool queue ports"
```

---

### Task 2: 在 uc-infra 实现端口（缓存 + 队列适配器）

**Files:**

- Modify: `src-tauri/crates/uc-infra/src/clipboard/representation_cache.rs`
- Create: `src-tauri/crates/uc-infra/src/clipboard/spool_queue.rs`
- Modify: `src-tauri/crates/uc-infra/src/clipboard/mod.rs`

**Step 1: 写一个失败的单元测试（队列适配器）**

在 `spool_queue.rs` 添加：

```rust
#[cfg(test)]
mod tests {
    use super::MpscSpoolQueue;
    use tokio::sync::mpsc;
    use uc_core::ids::RepresentationId;
    use uc_core::ports::clipboard::{SpoolQueuePort, SpoolRequest};

    #[tokio::test]
    async fn enqueues_request() {
        let (tx, mut rx) = mpsc::channel(1);
        let queue = MpscSpoolQueue::new(tx);
        let req = SpoolRequest {
            rep_id: RepresentationId::new(),
            bytes: vec![1, 2, 3],
        };

        queue.enqueue(req.clone()).await.expect("enqueue");
        let received = rx.recv().await.expect("recv");
        assert_eq!(received.rep_id, req.rep_id);
        assert_eq!(received.bytes, req.bytes);
    }
}
```

**Step 2: 运行测试确认失败**

Run: `cargo test -p uc-infra enqueues_request`
Expected: FAIL（类型/实现未定义）

**Step 3: 实现端口**

- 在 `representation_cache.rs` 为 `RepresentationCache` 实现 `RepresentationCachePort`（使用 async-trait，直接委托现有方法）。
- 在 `spool_queue.rs` 实现 `MpscSpoolQueue`：

```rust
pub struct MpscSpoolQueue {
    sender: mpsc::Sender<SpoolRequest>,
}

impl MpscSpoolQueue {
    pub fn new(sender: mpsc::Sender<SpoolRequest>) -> Self { Self { sender } }
}

#[async_trait::async_trait]
impl SpoolQueuePort for MpscSpoolQueue {
    async fn enqueue(&self, request: SpoolRequest) -> anyhow::Result<()> {
        self.sender
            .send(request)
            .await
            .map_err(|err| anyhow::anyhow!("spool queue closed: {err}"))
    }
}
```

- 在 `clipboard/mod.rs` 导出 `MpscSpoolQueue`。

**Step 4: 运行测试确认通过**

Run: `cargo test -p uc-infra enqueues_request`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-infra/src/clipboard/representation_cache.rs \
        src-tauri/crates/uc-infra/src/clipboard/spool_queue.rs \
        src-tauri/crates/uc-infra/src/clipboard/mod.rs

git commit -m "feat(infra): implement clipboard cache and spool queue ports"
```

---

### Task 3: 更新 uc-infra spooler 与 worker 使用新的 SpoolRequest

**Files:**

- Modify: `src-tauri/crates/uc-infra/src/clipboard/spooler_task.rs`

**Step 1: 运行现有测试确认失败**

Run: `cargo test -p uc-infra spooler_task`
Expected: FAIL（类型引用变更）

**Step 2: 更新实现与测试引用**

- `SpoolRequest` 改为 `uc_core::ports::clipboard::SpoolRequest`
- 其余逻辑不变

**Step 3: 运行测试确认通过**

Run: `cargo test -p uc-infra spooler_task`
Expected: PASS

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-infra/src/clipboard/spooler_task.rs

git commit -m "refactor(infra): use core spool request in spooler task"
```

---

### Task 4: 更新 uc-app 依赖为端口（用例与依赖组）

**Files:**

- Modify: `src-tauri/crates/uc-app/src/deps.rs`
- Modify: `src-tauri/crates/uc-app/src/usecases/internal/capture_clipboard.rs`

**Step 1: 运行 uc-app 测试确认失败**

Run: `cargo test -p uc-app capture_clipboard`
Expected: FAIL（依赖类型不匹配）

**Step 2: 更新 AppDeps 与用例签名**

- `AppDeps`：
  - `representation_cache: Arc<dyn RepresentationCachePort>`
  - `spool_queue: Arc<dyn SpoolQueuePort>`
  - 移除 `mpsc::Sender<SpoolRequest>`
- `CaptureClipboardUseCase`：
  - 使用 `RepresentationCachePort` 与 `SpoolQueuePort`
  - 入队改为 `enqueue(...).await`
  - 若入队失败：`warn!` 记录并返回 `Err`（可观测且向上层传播）

**Step 3: 运行测试确认通过**

Run: `cargo test -p uc-app capture_clipboard`
Expected: PASS

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-app/src/deps.rs \
        src-tauri/crates/uc-app/src/usecases/internal/capture_clipboard.rs

git commit -m "refactor(app): depend on cache/spool ports"
```

---

### Task 5: 更新 wiring/runtime 注入与 background 通道

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`

**Step 1: 运行相关构建测试确认失败**

Run: `cargo test -p uc-tauri wiring`
Expected: FAIL（依赖字段变更）

**Step 2: 更新注入**

- 创建 `RepresentationCache` 后上转型为 `Arc<dyn RepresentationCachePort>`
- 创建 mpsc 通道后用 `MpscSpoolQueue` 包装 `spool_tx`
- `AppDeps` 注入 `spool_queue`
- runtime 中 `CaptureClipboardUseCase::new` 传入 `spool_queue`

**Step 3: 运行测试确认通过**

Run: `cargo test -p uc-tauri wiring`
Expected: PASS

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs \
        src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs

git commit -m "refactor(tauri): wire cache/spool ports"
```

---

### Task 6: 更新 uc-app 测试与依赖声明

**Files:**

- Modify: `src-tauri/crates/uc-app/tests/snapshot_cache_integration_test.rs`
- Modify: `src-tauri/crates/uc-app/tests/stress_test.rs`
- Modify: `src-tauri/crates/uc-app/Cargo.toml`

**Step 1: 运行现有测试确认失败**

Run: `cargo test -p uc-app snapshot_cache_integration_test`
Expected: FAIL（类型/构造器变更）

**Step 2: 更新测试构造**

- 使用 `Arc<dyn RepresentationCachePort>`
- 使用 `MpscSpoolQueue` 或测试内自定义 `SpoolQueuePort` 适配器
- `SpoolRequest` 改用 `uc_core::ports::clipboard::SpoolRequest`

**Step 3: 移动依赖**

- 将 `uc-infra` 从 `[dependencies]` 移至 `[dev-dependencies]`

**Step 4: 运行测试确认通过**

Run: `cargo test -p uc-app snapshot_cache_integration_test`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-app/tests/snapshot_cache_integration_test.rs \
        src-tauri/crates/uc-app/tests/stress_test.rs \
        src-tauri/crates/uc-app/Cargo.toml

git commit -m "test(app): update tests for cache/spool ports"
```

---

### Task 7: 全量回归验证

**Step 1: 运行核心测试**

Run: `cargo test -p uc-core`
Expected: PASS

**Step 2: 运行基础设施测试**

Run: `cargo test -p uc-infra`
Expected: PASS

**Step 3: 运行应用测试**

Run: `cargo test -p uc-app`
Expected: PASS

**Step 4: Commit（如有遗漏修复）**

```bash
git status -sb
# 若有修复，补提交
```
