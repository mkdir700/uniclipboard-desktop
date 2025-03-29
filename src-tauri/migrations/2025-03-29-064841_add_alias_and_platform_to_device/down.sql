-- This file should undo anything in `up.sql`
ALTER TABLE devices DROP COLUMN alias;
ALTER TABLE devices DROP COLUMN platform;
