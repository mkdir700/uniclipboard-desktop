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

    /// 本地落盘路径（加密后的 payload）
    pub store_path: Option<String>,

    /// 内容大小（字节）
    pub size: Option<i32>,

    /// MIME 类型
    pub mime: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = t_clipboard_item)]
pub struct NewClipboardItemRow<'a> {
    pub id: &'a str,
    pub record_id: &'a str,
    pub index_in_record: i32,
    pub content_type: &'a str,
    pub content_hash: &'a str,
    pub store_path: Option<&'a str>,
    pub size: Option<i32>,
    pub mime: Option<&'a str>,
}
