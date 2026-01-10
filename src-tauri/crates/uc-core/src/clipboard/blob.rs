pub struct NewBlob {
    pub blob_id: String,
    pub storage_path: String,
    pub size_bytes: i64,
    pub content_hash: String,
    pub encryption_algo: Option<String>,
    pub created_at_ms: i64,
}

impl NewBlob {
    pub fn new(
        blob_id: String,
        storage_path: String,
        size_bytes: i64,
        content_hash: String,
        encryption_algo: Option<String>,
        created_at_ms: i64,
    ) -> Self {
        Self {
            blob_id,
            storage_path,
            size_bytes,
            content_hash,
            encryption_algo,
            created_at_ms,
        }
    }
}
