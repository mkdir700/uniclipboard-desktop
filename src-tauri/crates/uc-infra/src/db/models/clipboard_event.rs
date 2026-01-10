use crate::db::{models::ClipboardRecordRow, schema::t_clipboard_item};
use diesel::prelude::*;

#[derive(Queryable, Insertable)]
#[diesel(table_name = clipboard_event)]
pub struct ClipboardEventRow {
    pub event_id: String,
    pub captured_at_ms: i64,
    pub source_device: String,
    pub snapshot_hash: String,
}
