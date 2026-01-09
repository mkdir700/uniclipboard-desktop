use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::clipboard::ClipboardOrigin;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClipboardRecordId(pub String);

impl From<String> for ClipboardRecordId {
    /// Wraps an owned `String` as a `ClipboardRecordId`.
    ///
    /// # Examples
    ///
    /// ```
    /// let id = ClipboardRecordId::from("abc123".to_string());
    /// assert_eq!(id, ClipboardRecordId("abc123".to_string()));
    /// ```
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ClipboardRecordId {
    /// Creates a ClipboardRecordId from a string slice.
    ///
    /// # Examples
    ///
    /// ```
    /// let id = ClipboardRecordId::from("abc");
    /// assert_eq!(id.0, "abc");
    /// ```
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardContentView {
    /// Record 主键（UUID / ULID）
    pub id: ClipboardRecordId,

    /// 产生该剪切板记录的设备 ID
    pub source_device_id: String,

    /// 来源：local / remote
    pub origin: ClipboardOrigin,

    /// 本次复制事件的整体 hash
    pub record_hash: String,

    /// 本次复制事件包含的 item 数量
    pub item_count: i32,

    pub items: Vec<ClipboardItemView>,

    /// 创建时间
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItemView {
    /// MIME（仅用于展示 / icon / 预览判断）
    pub mime: Option<String>,

    /// 大小提示（用于 UI 展示，不保证精确），None 表示大小未知
    pub size: Option<u64>,
}
