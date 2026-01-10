use crate::ids::EventId;

pub struct NewClipboardEvent {
    pub event_id: EventId,
    pub captured_at_ms: i64,
    pub source_device: String,
    pub snapshot_hash: String,
}

impl NewClipboardEvent {
    pub fn new(
        event_id: EventId,
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
