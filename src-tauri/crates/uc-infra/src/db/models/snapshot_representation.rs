#[derive(Queryable, Insertable)]
#[diesel(table_name = clipboard_snapshot_representation)]
pub struct SnapshotRepresentationRow {
    pub id: String,
    pub event_id: String,
    pub format_id: String,
    pub mime_type: Option<String>,
    pub size_bytes: i64,
    pub inline_data: Option<Vec<u8>>,
    pub blob_id: Option<String>,
}
