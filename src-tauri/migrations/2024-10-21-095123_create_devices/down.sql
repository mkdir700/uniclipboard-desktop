-- This file should undo anything in `up.sql`
DROP TABLE devices;
DROP TRIGGER IF EXISTS update_devices_updated_at;