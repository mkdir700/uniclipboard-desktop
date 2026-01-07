-- ============================================================
-- 剪切板记录（Record）与条目（Item）表
--
-- 设计要点：
-- - Record 表：表示“一次复制操作”（一次剪切板事件），用于 UI 列表展示 / 同步幂等 / 软删除等
-- - Item 表：表示该次复制操作中的“一个元素”（文件/图片/文本等），用于保存具体条目的元数据与落盘路径
-- - 多文件复制：1 条 record + N 条 item，通过 index_in_record 保序
-- ============================================================


-- ============================================================
-- t_clipboard_record：一次剪切板复制事件（Record）
-- ============================================================
CREATE TABLE t_clipboard_record (
  -- Record 主键：建议用 UUID/ULID
  id               TEXT PRIMARY KEY NOT NULL,

  -- 产生该记录的设备 ID（用于同步、过滤、审计）
  source_device_id TEXT NOT NULL,

  -- 记录来源：通常用于区分本地复制/远端同步写入
  -- 建议值：'local' | 'remote'
  origin           TEXT NOT NULL,

  -- 本次复制事件的整体 hash（对 items 的结构化摘要做 hash）
  -- 用于去重、幂等写入、快速比较“是否同一次复制”
  record_hash      TEXT NOT NULL,

  -- 本次复制事件包含多少个 item（例如复制 3 个文件 => item_count=3）
  -- 用于 UI 快速展示（“3 个文件”）与完整性校验
  item_count       INTEGER NOT NULL,

  -- 创建时间（建议统一用 Unix epoch 毫秒）
  -- 注意：SQLite 的 BIGINT 语义上等价于 INTEGER，但这里保留 BIGINT 便于表达
  created_at       BIGINT NOT NULL,

  -- 软删除时间（NULL 表示未删除；非 NULL 表示删除时刻）
  deleted_at       BIGINT
);

-- 按时间倒序查询历史记录的索引（UI 最近列表 / 分页）
CREATE INDEX idx_clipboard_record_created
ON t_clipboard_record(created_at DESC);



-- ============================================================
-- t_clipboard_item：一次复制事件内的具体条目（Item）
-- ============================================================
CREATE TABLE t_clipboard_item (
  -- Item 主键：建议用 UUID/ULID
  id               TEXT PRIMARY KEY NOT NULL,

  -- 外键：归属到哪条 record（一次复制事件）
  record_id        TEXT NOT NULL,

  -- 该 item 在 record 中的顺序（从 0 或 1 开始均可，但要全局一致）
  -- 多文件粘贴/回放时需要严格保序
  index_in_record  INTEGER NOT NULL,

  -- 内容类型：例如 'text' | 'image' | 'file' | 'rich_text' 等
  -- 建议与 uc-core 的 ContentType Display/序列化一致
  content_type     TEXT NOT NULL,

  -- 该 item 的内容 hash（用于 item 级去重、快速比较、以及 watcher 防回环）
  content_hash     TEXT NOT NULL,

  -- 内容存储的 Blob ID：用于从 Blob 存储系统获取内容
  blob_id          TEXT,

  -- 条目大小（字节数），用于清理策略与传输策略（例如大文件走 chunk）
  size             BIGINT NULL,

  -- MIME 类型：例如 'text/plain'、'image/png'、'application/octet-stream'
  mime             TEXT,

  -- 外键约束：item 必须属于某条 record
  FOREIGN KEY(record_id) REFERENCES t_clipboard_record(id)
);

-- 保证同一个 record 内的 item 顺序唯一
-- 防止同一 record 出现两个 index=0 的 item
CREATE UNIQUE INDEX ux_clipboard_item_order
ON t_clipboard_item(record_id, index_in_record);
