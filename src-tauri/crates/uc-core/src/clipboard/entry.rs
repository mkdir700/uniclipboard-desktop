use crate::ids::{EntryId, EventId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardEntry {
    pub entry_id: EntryId,
    pub event_id: EventId,
    pub created_at_ms: i64,
    pub title: Option<String>,
    pub total_size: i64,
}

impl ClipboardEntry {
    pub fn new(
        entry_id: EntryId,
        event_id: EventId,
        created_at_ms: i64,
        title: Option<String>,
        total_size: i64,
    ) -> Self {
        Self {
            entry_id,
            event_id,
            created_at_ms,
            title,
            total_size,
        }
    }
}
