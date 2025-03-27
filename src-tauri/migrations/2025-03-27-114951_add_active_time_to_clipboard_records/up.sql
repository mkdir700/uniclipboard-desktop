-- Your SQL goes here
-- 添加 active_time 列到 clipboard_records 表， 默认值为当前时间戳
ALTER TABLE clipboard_records ADD COLUMN active_time INTEGER NOT NULL DEFAULT (strftime('%s', 'now'));

-- 为 active_time 列创建索引以提高查询性能
CREATE INDEX idx_clipboard_records_active_time ON clipboard_records(active_time);

-- 确保所有现有记录都有 active_time 值, 为当前时间戳
UPDATE clipboard_records SET active_time = strftime('%s', 'now'); 