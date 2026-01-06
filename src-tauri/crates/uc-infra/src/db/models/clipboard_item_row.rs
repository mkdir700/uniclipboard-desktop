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
pub struct NewClipboardItemRow<'a> {
    pub id: &'a str,
    pub record_id: &'a str,
    pub index_in_record: i32,
    pub content_type: &'a str,
    pub content_hash: &'a str,
    pub blob_id: Option<&'a str>,
    pub size: Option<i64>,
    pub mime: Option<&'a str>,
}

/// Owned version of NewClipboardItemRow for easier construction
#[derive(Debug, Clone)]
pub struct NewClipboardItemRowOwned {
    pub id: String,
    pub record_id: String,
    pub index_in_record: i32,
    pub content_type: String,
    pub content_hash: String,
    pub blob_id: Option<String>,
    pub size: Option<i64>,
    pub mime: Option<String>,
}

impl<'a> From<&'a NewClipboardItemRowOwned> for NewClipboardItemRow<'a> {
    /// Create a `NewClipboardItemRow` that borrows string data from the given owned instance.
    ///
    /// `blob_id` and `mime` are converted from `Option<String>` to `Option<&str>`.
    ///
    /// # Examples
    ///
    /// ```
    /// let owned = NewClipboardItemRowOwned {
    ///     id: "id".to_string(),
    ///     record_id: "rec".to_string(),
    ///     index_in_record: 0,
    ///     content_type: "text".to_string(),
    ///     content_hash: "hash".to_string(),
    ///     blob_id: Some("blob".to_string()),
    ///     size: Some(42),
    ///     mime: Some("text/plain".to_string()),
    /// };
    ///
    /// let borrowed: NewClipboardItemRow = (&owned).into();
    /// assert_eq!(borrowed.id, owned.id.as_str());
    /// assert_eq!(borrowed.blob_id, owned.blob_id.as_deref());
    /// assert_eq!(borrowed.mime, owned.mime.as_deref());
    /// assert_eq!(borrowed.size, owned.size);
    /// ```
    fn from(owned: &'a NewClipboardItemRowOwned) -> Self {
        NewClipboardItemRow {
            id: &owned.id,
            record_id: &owned.record_id,
            index_in_record: owned.index_in_record,
            content_type: &owned.content_type,
            content_hash: &owned.content_hash,
            blob_id: owned.blob_id.as_deref(),
            size: owned.size,
            mime: owned.mime.as_deref(),
        }
    }
}
