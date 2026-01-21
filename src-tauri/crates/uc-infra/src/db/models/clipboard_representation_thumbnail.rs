use diesel::prelude::*;

use crate::db::schema::clipboard_representation_thumbnail;

#[derive(Debug, Clone, Queryable)]
#[diesel(table_name = clipboard_representation_thumbnail)]
pub struct ClipboardRepresentationThumbnailRow {
    pub representation_id: String,
    pub thumbnail_blob_id: String,
    pub thumbnail_mime_type: String,
    pub width: i32,
    pub height: i32,
    pub size_bytes: i64,
    pub created_at_ms: Option<i64>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = clipboard_representation_thumbnail)]
pub struct NewClipboardRepresentationThumbnailRow<'a> {
    pub representation_id: &'a str,
    pub thumbnail_blob_id: &'a str,
    pub thumbnail_mime_type: &'a str,
    pub width: i32,
    pub height: i32,
    pub size_bytes: i64,
    pub created_at_ms: Option<i64>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_thumbnail_table_exists() {
        let _ = crate::db::schema::clipboard_representation_thumbnail::table;
    }
}
