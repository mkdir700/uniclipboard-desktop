use crate::db::schema::t_clipboard_record;
use diesel::prelude::*;

#[derive(Debug, Clone, Queryable, Identifiable, Selectable)]
#[diesel(table_name = t_clipboard_record)]
pub struct ClipboardRecordRow {
    /// Record 主键（UUID / ULID）
    pub id: String,

    /// 产生该剪切板记录的设备 ID
    pub source_device_id: String,

    /// 来源：local / remote
    pub origin: String,

    /// 本次复制事件的整体 hash
    pub record_hash: String,

    /// 本次复制事件包含的 item 数量
    pub item_count: i32,

    /// 创建时间（Unix epoch 毫秒）
    pub created_at: i64,

    /// 软删除时间（NULL = 未删除）
    pub deleted_at: Option<i64>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = t_clipboard_record)]
pub struct NewClipboardRecordRow<'a> {
    pub id: &'a str,
    pub source_device_id: &'a str,
    pub origin: &'a str,
    pub record_hash: &'a str,
    pub item_count: i32,
    pub created_at: i64,
    pub deleted_at: Option<i64>,
}
