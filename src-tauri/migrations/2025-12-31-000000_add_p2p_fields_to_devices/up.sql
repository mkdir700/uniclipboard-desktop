-- Add P2P-related fields to devices table
-- This migration adds support for libp2p peer identification and pairing

-- Add peer_id column (libp2p PeerId)
ALTER TABLE devices ADD COLUMN peer_id TEXT;

-- Add device_name column (human-readable device name)
ALTER TABLE devices ADD COLUMN device_name TEXT;

-- Add is_paired column (whether device has completed pairing)
ALTER TABLE devices ADD COLUMN is_paired BOOLEAN DEFAULT FALSE NOT NULL;

-- Add last_seen column (timestamp of last contact)
ALTER TABLE devices ADD COLUMN last_seen INTEGER;
