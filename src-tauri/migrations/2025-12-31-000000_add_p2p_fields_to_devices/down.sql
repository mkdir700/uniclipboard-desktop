-- Rollback P2P fields from devices table

ALTER TABLE devices DROP COLUMN last_seen;
ALTER TABLE devices DROP COLUMN is_paired;
ALTER TABLE devices DROP COLUMN device_name;
ALTER TABLE devices DROP COLUMN peer_id;
