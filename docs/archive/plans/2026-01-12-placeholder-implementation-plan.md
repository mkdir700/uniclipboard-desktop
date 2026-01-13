# Placeholder 实现计划

**日期**: 2026-01-12
**状态**: 待实施
**范围**: Phase 1 (基础设施层) + Phase 2 (核心业务层)

## 概述

本文档规划了 UniClipboard Desktop 项目中所有 placeholder 的实现计划。根据依赖层次分析，将实现分为两个阶段，专注于构建一个**本地可用**的剪贴板历史和管理系统。

### 目标

- ✅ 捕获剪贴板变化
- ✅ 存储剪贴板历史
- ✅ 物化和展示剪贴板内容
- ✅ 设备管理（本地设备信息）
- ❌ 设备间同步（留待未来）

### Placeholder 统计

| 类型                                            | 数量     |
| ----------------------------------------------- | -------- |
| 核心功能占位符 (`unimplemented!()` / `todo!()`) | 5 个文件 |
| 占位符适配器 (Placeholder Adapters)             | 10 个    |

---

## Phase 1 - 基础设施层

### 目标

实现最底层的依赖，为上层功能提供支撑。完成后可以：

- 剪贴板内容能够被捕获和存储
- 设备信息可以被持久化
- 依赖注入系统可以正常工作

### 功能清单

| #   | 功能模块           | 文件位置                                               | 涉及的 placeholder                                |
| --- | ------------------ | ------------------------------------------------------ | ------------------------------------------------- |
| 1   | **依赖注入连接**   | `uc-tauri/src/bootstrap/wiring.rs`                     | 替换所有 `todo!()` 注入点                         |
| 2   | **设备仓库**       | `uc-infra/src/db/repositories/device_repo.rs`          | `find_by_id`, `save`, `delete`, `list_all`        |
| 3   | **剪贴板事件仓库** | `uc-infra/src/db/repositories/clipboard_event_repo.rs` | `PlaceholderClipboardEventRepository`             |
| 4   | **剪贴板运行时**   | `uc-platform/src/runtime/runtime.rs`                   | `handle_event`, `ReadClipboard`, `WriteClipboard` |
| 5   | **Blob 存储**      | `uc-platform/src/adapters/blob.rs`                     | `PlaceholderBlobStorePort`                        |

### 依赖关系

```
┌─────────────────────────────────────────────────────┐
│  wiring.rs (依赖注入)                                │
│  └── 连接所有端口和实现                              │
└──────────────────┬──────────────────────────────────┘
                   │
       ┌───────────┴───────────┬──────────────┐
       ▼                       ▼              ▼
┌──────────────┐    ┌──────────────────┐  ┌──────────┐
│ device_repo  │    │ clipboard_event  │  │   Blob   │
│              │    │     _repo        │  │  Store   │
└──────────────┘    └──────────────────┘  └──────────┘
       │                       │
       └───────────┬───────────┘
                   ▼
         ┌──────────────────┐
         │   runtime.rs     │
         │ (剪贴板运行时)     │
         └──────────────────┘
```

### 实现要点

#### 1. 依赖注入连接 (`wiring.rs`)

**位置**: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs:252-256, 700-706`

**当前状态**: 包含多个 `todo!()` 占位符

**实现内容**:

- `ClipboardEventRepositoryPort` 注入
- `blob_repository` 和 `blob_materializer` 注入
- 恢复已注释的 `unimplemented!()` 代码块

#### 2. 设备仓库 (`device_repo.rs`)

**位置**: `src-tauri/crates/uc-infra/src/db/repositories/device_repo.rs:32-64`

**当前状态**: 所有方法都是 `unimplemented!()`，但有注释的实现代码

**实现内容**:

```rust
async fn find_by_id(&self, device_id: &DeviceId) -> Result<Option<Device>>;
async fn save(&self, device: Device) -> Result<()>;
async fn delete(&self, device_id: &DeviceId) -> Result<()>;
async fn list_all(&self) -> Result<Vec<Device>>;
```

**依赖**: `DeviceRow` 模型已定义在 `uc-infra/src/db/models/`

#### 3. 剪贴板事件仓库 (`clipboard_event_repo.rs`)

**位置**: `src-tauri/crates/uc-infra/src/db/repositories/clipboard_event_repo.rs:185-197`

**当前状态**: `PlaceholderClipboardEventRepository` 占位符

**实现内容**:

- `get_representation()` - 查询剪贴板表示
- 数据库连接和查询逻辑
- 与 `representation_repo` 协作

#### 4. 剪贴板运行时 (`runtime.rs`)

**位置**: `src-tauri/crates/uc-platform/src/runtime/runtime.rs:99-111`

**当前状态**: 事件和命令处理是 `todo!()`

**实现内容**:

```rust
async fn handle_event(&self, event: PlatformEvent);
async fn handle_command(&mut self, command: PlatformCommand);
// - PlatformCommand::ReadClipboard
// - PlatformCommand::WriteClipboard { content }
```

**已有基础**: `LocalClipboard` 和 `ClipboardWatcher` 已实现

#### 5. Blob 存储 (`adapters/blob.rs`)

**位置**: `src-tauri/crates/uc-platform/src/adapters/blob.rs`

**当前状态**: `PlaceholderBlobStorePort`

**实现内容**:

```rust
async fn store(&self, blob_id: &str, bytes: &[u8]) -> Result<()>;
async fn retrieve(&self, blob_id: &str) -> Result<Option<Vec<u8>>>;
```

**建议**: 使用文件系统存储，参考 `uc-infra/src/blob/`

---

## Phase 2 - 核心业务层

### 目标

实现剪贴板同步的核心业务逻辑。完成后可以：

- 捕获本地剪贴板变化
- 选择最佳表示格式
- 物化剪贴板内容（包括大文件）
- 将内容投影到前端展示

### 功能清单

| #   | 功能模块           | 文件位置                                                          | 涉及的 placeholder                                   |
| --- | ------------------ | ----------------------------------------------------------------- | ---------------------------------------------------- |
| 6   | **剪贴板内容物化** | `uc-app/src/usecases/internal/materialize_clipboard_selection.rs` | `load_representation_bytes()`                        |
| 7   | **剪贴板投影查询** | `uc-infra/src/clipboard/projection.rs`                            | `get_projection()`, `list_projections()`             |
| 8   | **Blob 物化器**    | `uc-platform/src/adapters/blob.rs`                                | `PlaceholderBlobMaterializerPort`                    |
| 9   | **剪贴板表示物化** | `uc-platform/src/adapters/clipboard.rs`                           | `PlaceholderClipboardRepresentationMaterializerPort` |
| 10  | **加密会话**       | `uc-platform/src/adapters/encryption.rs`                          | `PlaceholderEncryptionSessionPort` 增强              |

### 依赖关系

```
                    ┌─────────────────────────────────┐
                    │     materialize_clipboard_       │
                    │        selection.rs              │
                    │    (剪贴板内容物化用例)            │
                    └────────────┬────────────────────┘
                                 │
                ┌────────────────┼────────────────┐
                ▼                ▼                ▼
         ┌─────────────┐  ┌─────────────┐  ┌──────────────┐
         │   Blob      │  │ Clipboard   │  │ Encryption   │
         │Materializer │  │ReprMaterial.│  │   Session    │
         └─────────────┘  └─────────────┘  └──────────────┘
                │                │
                └────────┬───────┘
                         ▼
                ┌─────────────────┐
                │   projection.rs │
                │   (投影查询)     │
                └─────────────────┘
                         │
                         ▼
                ┌─────────────────┐
                │   前端展示       │
                └─────────────────┘
```

### 实现要点

#### 6. 剪贴板内容物化 (`materialize_clipboard_selection.rs`)

**位置**: `src-tauri/crates/uc-app/src/usecases/internal/materialize_clipboard_selection.rs:113-118`

**当前状态**: `load_representation_bytes()` 是 `unimplemented!()`

**实现内容**:

- 从剪贴板系统读取原始字节
- 支持不同格式（文本、图片、文件等）
- 处理跨平台差异

**关键点**: 需要与 `LocalClipboard` 和平台特定剪贴板适配器协作

#### 7. 剪贴板投影查询 (`projection.rs`)

**位置**: `src-tauri/crates/uc-infra/src/clipboard/projection.rs:13-24`

**当前状态**: `get_projection()` 和 `list_projections()` 是 `todo!()`

**实现内容**:

```rust
async fn get_projection(&self, entry_id: &EntryId) -> Result<ClipboardEntryProjection>;
async fn list_projections(&self, limit: usize, offset: usize) -> Result<Vec<ClipboardEntryProjection>>;
```

**已有基础**: `ClipboardSelectionRepositoryPort` 已实现

#### 8. Blob 物化器 (`adapters/blob.rs`)

**位置**: `src-tauri/crates/uc-platform/src/adapters/blob.rs`

**当前状态**: `PlaceholderBlobMaterializerPort`

**实现内容**:

```rust
async fn materialize(&self, raw_bytes: &[u8], content_hash: &str) -> Result<MaterializedBlob>;
```

**功能**:

- 将原始字节转换为持久化 blob
- 生成内容哈希
- 调用 `BlobStorePort` 存储数据

#### 9. 剪贴板表示物化 (`adapters/clipboard.rs`)

**位置**: `src-tauri/crates/uc-platform/src/adapters/clipboard.rs`

**当前状态**: `PlaceholderClipboardRepresentationMaterializerPort`

**实现内容**:

```rust
async fn materialize_representation(&self, representation: &ClipboardRepresentation) -> Result<()>;
```

**功能**:

- 将剪贴板表示写入系统剪贴板
- 处理不同 MIME 类型的转换
- 调用 `SystemClipboardPort`

#### 10. 加密会话 (`adapters/encryption.rs`)

**位置**: `src-tauri/crates/uc-platform/src/adapters/encryption.rs`

**当前状态**: `PlaceholderEncryptionSessionPort` (有基本内存存储)

**实现内容**:

- 主密钥的安全存储（可能需要 Tauri Stronghold）
- 密钥零化（zeroization）处理
- 会话状态持久化

**注意**: 此模块已有基本结构，主要是增强安全性

---

## 暂不实现的功能 (Phase 3)

以下功能将保留为 placeholder，留待未来实现：

| 功能          | 文件                                    | 占位符                          |
| ------------- | --------------------------------------- | ------------------------------- |
| P2P 网络      | `uc-platform/src/adapters/network.rs`   | `PlaceholderNetworkPort`        |
| mDNS 设备发现 | `network.rs`                            | `discover_peers()`, `peer_id()` |
| 配对流程      | `network.rs`                            | PIN 交换相关方法                |
| UI 事件端口   | `uc-platform/src/adapters/ui.rs`        | `PlaceholderUiPort`             |
| 自启动管理    | `uc-platform/src/adapters/autostart.rs` | `PlaceholderAutostartPort`      |

---

## 完整依赖图

```
                    ┌─────────────────────────────────┐
                    │          Phase 2 - 核心业务层      │
                    │  ┌──────────────┐  ┌───────────┐ │
                    │  │ 内容物化     │  │ 投影查询   │ │
                    │  └──────────────┘  └───────────┘ │
                    │         │                │        │
                    │         ▼                ▼        │
                    │  ┌──────────────┐  ┌───────────┐ │
                    │  │ Blob 物化器   │  │ 表示物化   │ │
                    │  └──────────────┘  └───────────┘ │
                    │         │                │        │
                    │         └────────┬───────┘        │
                    │                  ▼                │
                    │         ┌──────────────────┐      │
                    │         │   加密会话        │      │
                    │         └──────────────────┘      │
                    └──────────────────┬───────────────┘
                                       │
┌──────────────────────────────────────┴──────────────────────────────┐
│                    Phase 1 - 基础设施层                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │
│  │ 依赖注入连接 │  │  设备仓库     │  │    剪贴板事件仓库          │  │
│  └──────────────┘  └──────────────┘  └──────────────────────────┘  │
│         │                   │                   │                   │
│         └───────────────────┼───────────────────┘                   │
│                             ▼                                       │
│                   ┌──────────────────┐                              │
│                   │  剪贴板运行时      │     Blob 存储                │
│                   └──────────────────┘  ┌─────────────────────────┐  │
│                                         │      文件系统存储         │  │
│                                         └─────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 完成后成果

Phase 1 + Phase 2 完成后，你将拥有一个可以工作的剪贴板历史和管理系统：

```
用户复制内容
    │
    ▼
剪贴板监听 ──────► 存储到数据库
    │                    │
    ▼                    ▼
物化内容 ◄───────── Blob 存储
    │
    ▼
投影查询 ◄───────── 前端展示
```

### 可用功能

- ✅ 剪贴板历史记录
- ✅ 内容搜索和过滤
- ✅ 剪贴板条目恢复
- ✅ 本地设备信息管理
- ✅ 加密内容存储

### 暂不可用

- ❌ 设备间同步
- ❌ mDNS 设备发现
- ❌ P2P 配对
- ❌ 自启动管理

---

## 时间估算

| 阶段     | 功能数 | 预计时间     |
| -------- | ------ | ------------ |
| Phase 1  | 5      | 5-7 天       |
| Phase 2  | 5      | 5-7 天       |
| **总计** | **10** | **10-14 天** |

---

## 相关文档

- [架构设计文档](../architecture/README.md)
- [Hexagonal 重构概述](../2026-01-12-bootstrap-architecture-design.md)
- [Phase 2 创建总结](../2026-01-12-bootstrap-phase2-summary.md)
