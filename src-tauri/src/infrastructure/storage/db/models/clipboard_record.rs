use diesel::prelude::*;
use crate::core::transfer::{ContentType, ClipboardMetadata};
use anyhow::Result;

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

impl DbClipboardRecord {
    /// 获取内容类型枚举
    pub fn get_content_type(&self) -> Option<ContentType> {
        ContentType::from_str(&self.content_type)
    }
}

/// 剪贴板项目元数据
/// 
/// 用于从DbClipboardRecord中提取显示信息
pub struct ClipboardItemMetadata {
    pub display_content: String,
    // 根据需要添加其他字段
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
