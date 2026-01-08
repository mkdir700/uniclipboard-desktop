use crate::db::models::{ClipboardRecordRow, NewClipboardItemRow, NewClipboardRecordRow};
use uc_core::clipboard::{ClipboardContent, ClipboardItem};
use uuid::Uuid;

impl From<(&ClipboardItem, &str, &str, i32)> for NewClipboardItemRow {
    fn from((item, record_id, blob_id, index): (&ClipboardItem, &str, &str, i32)) -> Self {
        NewClipboardItemRow {
            id: Uuid::new_v4().to_string(),
            record_id: record_id.to_string(),
            index_in_record: index,
            content_type: item.mime.to_string(),
            content_hash: item.hash().to_string(),
            blob_id: Some(blob_id.to_string()),
            size: item.size_bytes().and_then(|v| v.try_into().ok()),
            mime: Some(item.mime.to_string()),
        }
    }
}

impl From<(&ClipboardContent, &str)> for NewClipboardRecordRow {
    fn from((content, record_id): (&ClipboardContent, &str)) -> Self {
        let device_id = content.get_device_id().unwrap_or("unknown");
        let origin = content.get_origin().unwrap_or("unknown");
        NewClipboardRecordRow {
            id: record_id.to_string(),
            source_device_id: device_id.to_string(),
            origin: origin.to_string(),
            record_hash: content.content_hash().to_string(),
            version: content.v as i32,
            occurred_at: content.occurred_at.as_millis(),
            item_count: content.items.len() as i32,
        }
    }
}

impl NewClipboardRecordRow {
    pub fn from_content(content: &ClipboardContent, record_id: &str) -> Self {
        NewClipboardRecordRow::from((content, record_id))
    }
}
