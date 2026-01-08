use crate::db::{models::ClipboardRecordRow, schema::t_clipboard_item};
use diesel::prelude::*;

#[derive(Debug, Clone, Queryable, Identifiable, Selectable, Associations)]
#[diesel(table_name = t_clipboard_item)]
#[diesel(belongs_to(ClipboardRecordRow, foreign_key = record_id))]
pub struct ClipboardItemRow {
    /// Item 主键
    pub id: String,

    /// 所属 record ID
    pub record_id: String,

    /// 在 record 中的顺序（必须保序）
    pub index_in_record: i32,

    /// 内容类型：text / image / file ...
    pub content_type: String,

    /// item 级内容 hash
    pub content_hash: String,

    /// blob 存储 ID
    pub blob_id: Option<String>,

    /// 内容大小（字节），NULL 表示大小未知
    pub size: Option<i64>,

    /// MIME 类型
    pub mime: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = t_clipboard_item)]
pub struct NewClipboardItemRow {
    pub id: String,
    pub record_id: String,
    pub index_in_record: i32,
    pub content_type: String,
    pub content_hash: String,
    pub blob_id: Option<String>,
    pub size: Option<i64>,
    pub mime: Option<String>,
}
