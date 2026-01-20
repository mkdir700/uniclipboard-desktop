# Clipboard Resource Protocol Design

## 背景与目标

- 需求：详情内容不要直接返回文本或 base64，使用自定义协议 `uc://blob/{blob_id}` 作为原始内容通道。
- 要求：语义化预览与原始内容通道严格分离。
- 列表页必须包含预览内容（文本或占位描述）。

## 非目标

- 不在此阶段实现加密鉴权或访问令牌（可作为后续增强）。
- 不改变捕获/写盘（spool/worker）现有流程。

## 架构与分层

- **uc-app**：新增用例生成资源元信息与 URL，不返回 bytes。
- **uc-core**：新增 Port 以根据 `blob_id` 获取表示元信息（mime/size）。
- **uc-infra**：实现 Port，通过 SQLite 查询 `clipboard_snapshot_representation`。
- **uc-tauri**：注册自定义协议 `uc://` 处理器，返回 bytes + MIME。
- **前端**：列表 API 用于预览；详情使用 `uc://blob/{blob_id}` 加载原始内容。

## API 设计

### 1) list entries（保持预览）

- 返回 `preview` 文本（从 `inline_data` 读取或占位）。
- 不返回资源 URL 或 bytes。

### 2) get_clipboard_entry_resource（新增）

- 输入：`entry_id`
- 输出：`{ blob_id, mime, size_bytes, url }`
- 用例流程：
  1. entry → selection → preview_rep
  2. `blob_id` 必须存在，否则返回错误
  3. `url = "uc://blob/{blob_id}"`

## 自定义协议 `uc://blob/{blob_id}`

- 仅返回 **bytes + MIME**。
- 处理器流程：
  1. 解析 `blob_id`
  2. 查询表示元信息（mime/size）
  3. `BlobStorePort::get(blob_id)` 读取 bytes
  4. 返回响应（MIME + bytes）
- 失败场景必须记录日志并返回错误状态（不静默失败）。

## 新增 Port

### ClipboardRepresentationRepositoryPort

- 新增方法：
  - `get_representation_by_blob_id(blob_id) -> Option<PersistedClipboardRepresentation>`
- 仅用于查询 mime/size 等元信息，不读取 bytes。

## 错误处理

- `get_clipboard_entry_resource`：
  - entry/selection/representation 缺失 → 明确错误
  - `blob_id` 为空 → 明确错误
- 协议 handler：
  - `blob_id` 无效或缺失 → 404 类错误
  - BlobStore 读取失败 → 500 类错误
- 全部错误必须写日志。

## 测试策略

- uc-app 用例测试：资源元信息返回、缺失 blob_id 错误。
- uc-infra 仓储测试：按 `blob_id` 查询 representation。
- 协议 handler 集成测试（若已有框架支持）。

## 前端改动

- 列表仍使用 `get_clipboard_entries` 的 `preview` 字段。
- 详情页面改为请求 `get_clipboard_entry_resource`，再用 `uc://blob/{blob_id}` 加载内容。
