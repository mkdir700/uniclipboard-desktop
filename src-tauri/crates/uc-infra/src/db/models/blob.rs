CREATE TABLE blob (
    blob_id         TEXT PRIMARY KEY,
    storage_path    TEXT NOT NULL,
    size_bytes      INTEGER NOT NULL,

    content_hash    TEXT NOT NULL,
    encryption_algo TEXT,
    created_at_ms   INTEGER NOT NULL
);

CREATE INDEX idx_blob_hash
ON blob (content_hash);
