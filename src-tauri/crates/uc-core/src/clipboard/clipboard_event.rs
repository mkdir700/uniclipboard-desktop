pub struct NewClipboardEvent {
    pub event_id: String,
    pub captured_at_ms: i64,
    pub source_device: String,
    pub snapshot_hash: String,
}

pub struct NewSnapshotRepresentation {
    pub id: String,
    pub format_id: String,
    pub mime_type: Option<String>,
    pub size_bytes: i64,
    pub inline_data: Option<Vec<u8>>,
    pub blob_id: Option<String>,
}

impl NewClipboardEvent {
    pub fn new(
        event_id: String,
        captured_at_ms: i64,
        source_device: String,
        snapshot_hash: String,
    ) -> Self {
        Self {
            event_id,
            captured_at_ms,
            source_device,
            snapshot_hash,
        }
    }
}
