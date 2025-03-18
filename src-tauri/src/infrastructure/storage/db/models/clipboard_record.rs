use diesel::prelude::*;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::infrastructure::storage::db::schema::clipboard_records)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DbClipboardRecord {
    pub id: String,
    pub device_id: String,
    pub local_file_path: Option<String>,
    pub remote_record_id: Option<String>,
    pub content_type: String,
    pub is_favorited: bool,
    pub created_at: i32,
    pub updated_at: i32,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::infrastructure::storage::db::schema::clipboard_records)]
pub struct NewClipboardRecord {
    pub id: String,
    pub device_id: String,
    pub local_file_path: Option<String>,
    pub remote_record_id: Option<String>,
    pub content_type: String,
    pub is_favorited: bool,
    pub created_at: i32,
    pub updated_at: i32,
}
