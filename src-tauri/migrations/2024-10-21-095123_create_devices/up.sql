-- Your SQL goes here
CREATE TABLE devices (
    id TEXT PRIMARY KEY NOT NULL,
    ip TEXT NULL,
    port INTEGER NULL,
    server_port INTEGER NULL,
    status INTEGER NOT NULL,
    self_device BOOLEAN NOT NULL,
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE TRIGGER update_devices_updated_at
AFTER UPDATE ON devices
FOR EACH ROW
BEGIN
    UPDATE devices SET updated_at = strftime('%s', 'now')
    WHERE id = NEW.id;
END;