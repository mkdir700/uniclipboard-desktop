-- 这个文件应该撤销 up.sql 中的操作

-- 删除为 content_hash 列创建的索引
DROP INDEX IF EXISTS idx_clipboard_records_content_hash;

-- 从 clipboard_records 表中删除 content_hash 列
-- SQLite 不直接支持 DROP COLUMN，需要通过创建新表并复制数据来实现
CREATE TABLE clipboard_records_new (
    id TEXT PRIMARY KEY NOT NULL,
    device_id TEXT NOT NULL,
    local_file_path TEXT NULL,
    remote_record_id TEXT NULL,
    content_type TEXT NOT NULL,
    is_favorited BOOLEAN NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- 复制数据到新表，不包括 content_hash 列
INSERT INTO clipboard_records_new 
SELECT id, device_id, local_file_path, remote_record_id, content_type, is_favorited, created_at, updated_at
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
