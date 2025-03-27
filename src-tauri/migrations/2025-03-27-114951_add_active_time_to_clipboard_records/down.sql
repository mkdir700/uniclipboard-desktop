-- This file should undo anything in `up.sql`

-- 删除 active_time 列的索引
DROP INDEX idx_clipboard_records_active_time;

-- 删除 active_time 列
-- SQLite 不直接支持 DROP COLUMN，需要通过创建新表并复制数据来实现
CREATE TABLE clipboard_records_new (
    id TEXT PRIMARY KEY NOT NULL,
    device_id TEXT NOT NULL,
    local_file_path TEXT NULL,
    remote_record_id TEXT NULL,
    content_type TEXT NOT NULL,
    content_hash TEXT DEFAULT 'legacy_content',
    is_favorited BOOLEAN NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- 复制数据到新表，不包括 active_time 列
INSERT INTO clipboard_records_new 
SELECT id, device_id, local_file_path, remote_record_id, content_type, content_hash, is_favorited, created_at, updated_at
FROM clipboard_records;

-- 删除旧表
DROP TABLE clipboard_records;

-- 重命名新表
ALTER TABLE clipboard_records_new RENAME TO clipboard_records;

-- 重新创建触发器
CREATE TRIGGER update_clipboard_records_updated_at
AFTER UPDATE ON clipboard_records
FOR EACH ROW
BEGIN
    UPDATE clipboard_records SET updated_at = strftime('%s', 'now')
    WHERE id = NEW.id;
END;

-- 重新创建content_hash索引
CREATE INDEX idx_clipboard_records_content_hash ON clipboard_records(content_hash); 