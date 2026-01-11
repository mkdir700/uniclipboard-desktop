use crate::db::models::clipboard_entry::{ClipboardEntryRow, NewClipboardEntryRow};
use crate::db::ports::{InsertMapper, RowMapper};
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

impl RowMapper<ClipboardEntryRow, ClipboardEntry> for ClipboardEntryRowMapper {
    fn to_domain(&self, row: &ClipboardEntryRow) -> Result<ClipboardEntry> {
        Ok(ClipboardEntry::new(
            row.entry_id.clone().into(),
            row.event_id.clone().into(),
            row.created_at_ms,
            row.title.clone(),
            row.total_size,
        ))
    }
}
