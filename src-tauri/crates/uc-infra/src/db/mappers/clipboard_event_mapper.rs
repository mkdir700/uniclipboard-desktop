use crate::db::models::clipboard_event::NewClipboardEventRow;
use crate::db::ports::Mapper;
use uc_core::clipboard::ClipboardEvent;

pub struct ClipboardEventRowMapper;

impl Mapper<ClipboardEvent, NewClipboardEventRow> for ClipboardEventRowMapper {
    fn to_row(&self, domain: &ClipboardEvent) -> NewClipboardEventRow {
        NewClipboardEventRow {
            event_id: domain.event_id.clone().into(),
            captured_at_ms: domain.captured_at_ms,
            source_device: domain.source_device.clone(),
            snapshot_hash: domain.snapshot_hash.clone(),
        }
    }
}
