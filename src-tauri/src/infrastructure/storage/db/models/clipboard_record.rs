use diesel::prelude::*;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::infrastructure::storage::db::schema::clipboard_records)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DbClipboardRecord {
    pub id: String,
    pub device_id: String,
    pub remote_file_url: Option<String>,
    pub local_file_url: Option<String>,
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
    pub remote_file_url: Option<String>,
    pub local_file_url: Option<String>,
    pub content_type: String,
    pub is_favorited: bool,
    pub created_at: i32,
    pub updated_at: i32,
}

#[derive(AsChangeset)]
#[diesel(table_name = crate::infrastructure::storage::db::schema::clipboard_records)]
pub struct UpdateClipboardRecord {
    pub device_id: String,
    pub remote_file_url: Option<String>,
    pub local_file_url: Option<String>,
    pub content_type: String,
    pub is_favorited: bool,
    pub updated_at: i32,
}