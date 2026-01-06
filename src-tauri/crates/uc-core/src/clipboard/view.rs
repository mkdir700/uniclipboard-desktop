use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClipboardOrigin {
    Local,
    Remote,
}

impl From<&str> for ClipboardOrigin {
    fn from(s: &str) -> Self {
        match s {
            "local" => ClipboardOrigin::Local,
            "remote" => ClipboardOrigin::Remote,
            _ => ClipboardOrigin::Local, // Default to Local for unknown values
        }
    }
}

impl From<String> for ClipboardOrigin {
    fn from(s: String) -> Self {
        match s.as_str() {
            "local" => ClipboardOrigin::Local,
            "remote" => ClipboardOrigin::Remote,
            _ => ClipboardOrigin::Local, // Default to Local for unknown values
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClipboardRecordId(pub String);

impl From<String> for ClipboardRecordId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ClipboardRecordId {
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
pub enum ClipboardItemKind {
    Text,
    Image,
    File,
    Link,
    CodeSnippet,
    RichText,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItemView {
    /// 业务语义上的内容类型
    // pub kind: ClipboardItemKind,

    /// MIME（仅用于展示 / icon / 预览判断）
    pub mime: Option<String>,

    /// 大小提示（用于 UI 展示，不保证精确）
    pub size: u64,
}
