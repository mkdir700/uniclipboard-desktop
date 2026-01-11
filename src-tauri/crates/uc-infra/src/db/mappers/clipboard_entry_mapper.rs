use crate::db::models::clipboard_entry::NewClipboardEntryRow;
use crate::db::ports::InsertMapper;
use anyhow::Result;
use uc_core::clipboard::ClipboardEntry;

pub struct ClipboardEntryRowMapper;

impl InsertMapper<ClipboardEntry, NewClipboardEntryRow> for ClipboardEntryRowMapper {
    fn to_row(&self, domain: &ClipboardEntry) -> Result<NewClipboardEntryRow> {
        Ok(NewClipboardEntryRow {
            entry_id: domain.entry_id.clone().into(),
            event_id: domain.event_id.clone().into(),
            created_at_ms: domain.created_at_ms,
            title: domain.title.clone(),
            total_size: domain.total_size,
            pinned: false, // TODO: implement
        })
    }
}
