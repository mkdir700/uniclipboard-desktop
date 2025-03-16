CREATE TABLE clipboard_records (
    id TEXT PRIMARY KEY NOT NULL,
    device_id TEXT NOT NULL,
    local_file_url TEXT NULL,
    remote_file_url TEXT NULL,
    content_type TEXT NOT NULL,
    is_favorited BOOLEAN NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE TRIGGER update_clipboard_records_updated_at
AFTER UPDATE ON clipboard_records
FOR EACH ROW
BEGIN
    UPDATE clipboard_records SET updated_at = strftime('%s', 'now')
    WHERE id = NEW.id;
END;