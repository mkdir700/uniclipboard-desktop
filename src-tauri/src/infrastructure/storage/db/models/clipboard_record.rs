use crate::core::content_type::ContentType;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::infrastructure::storage::db::schema::clipboard_records)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DbClipboardRecord {
    pub id: String,
    pub device_id: String,
    pub local_file_path: Option<String>,
    pub remote_record_id: Option<String>,
    pub content_type: String,
    pub content_hash: Option<String>,
    pub is_favorited: bool,
    pub created_at: i32,
    pub updated_at: i32,
    pub active_time: i32,
}

impl DbClipboardRecord {
    /// 获取内容类型枚举
    pub fn get_content_type(&self) -> Option<ContentType> {
        ContentType::from_str(&self.content_type)
    }

    /// 获取更新记录
    pub fn get_update_record(&self) -> UpdateClipboardRecord {
        UpdateClipboardRecord {
            is_favorited: self.is_favorited,
            updated_at: self.updated_at,
            active_time: self.active_time,
        }
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
    pub content_hash: String,
    pub is_favorited: bool,
    pub created_at: i32,
    pub updated_at: i32,
}

#[derive(AsChangeset, Debug)]
#[diesel(table_name = crate::infrastructure::storage::db::schema::clipboard_records)]
pub struct UpdateClipboardRecord {
    pub is_favorited: bool,
    pub updated_at: i32,
    pub active_time: i32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OrderBy {
    #[serde(rename = "created_at_asc")]
    CreatedAtAsc,
    #[serde(rename = "created_at_desc")]
    CreatedAtDesc,
    #[serde(rename = "updated_at_asc")]
    UpdatedAtAsc,
    #[serde(rename = "updated_at_desc")]
    UpdatedAtDesc,
    #[serde(rename = "active_time_asc")]
    ActiveTimeAsc,
    #[serde(rename = "active_time_desc")]
    ActiveTimeDesc,
    #[serde(rename = "content_type_asc")]
    ContentTypeAsc,
    #[serde(rename = "content_type_desc")]
    ContentTypeDesc,
    #[serde(rename = "is_favorited_asc")]
    IsFavoritedAsc,
    #[serde(rename = "is_favorited_desc")]
    IsFavoritedDesc,
}

impl Default for OrderBy {
    fn default() -> Self {
        OrderBy::ActiveTimeDesc
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Filter {
    #[serde(rename = "all")]
    All,
    #[serde(rename = "favorited")]
    Favorited,
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "image")]
    Image,
    #[serde(rename = "link")]
    Link,
    #[serde(rename = "code")]
    Code,
    #[serde(rename = "file")]
    File,
}

impl Default for Filter {
    fn default() -> Self {
        Filter::All
    }
}
