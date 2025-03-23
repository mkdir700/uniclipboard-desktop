-- Your SQL goes here

-- 添加 content_hash 列到 clipboard_records 表
ALTER TABLE clipboard_records ADD COLUMN content_hash TEXT DEFAULT 'legacy_content';

-- 为 content_hash 列创建索引以提高查询性能
CREATE INDEX idx_clipboard_records_content_hash ON clipboard_records(content_hash);

-- 确保所有现有记录都有content_hash值
UPDATE clipboard_records SET content_hash = 'legacy_content' WHERE content_hash IS NULL;
