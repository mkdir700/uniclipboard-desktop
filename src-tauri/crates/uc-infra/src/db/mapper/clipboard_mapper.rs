use crate::db::models::{ClipboardItemRow, ClipboardRecordRow, NewClipboardItemRow};
use std::hash::{Hash, Hasher};
use uc_core::clipboard::{ClipboardContent, ClipboardItem, MimeType};
use uuid::Uuid;

impl From<(&ClipboardItem, &str, i32)> for NewClipboardItemRow {
    fn from((item, record_id, index): (&ClipboardItem, &str, i32)) -> Self {
        let size = item.size_bytes().map(|v| v as i32);

        NewClipboardItemRow {
            id: Uuid::new_v4().to_string(),
            record_id: record_id.to_string(),
            index_in_record: index,
            content_type: item.mime.to_string(),
            content_hash: &item_hash(item),
            store_path: None, // 由 blob/storage 层后置填充
            size: size,
            mime: Some(item.mime.to_string()),
        }
    }
}

fn item_hash(item: &ClipboardItem) -> String {
    use std::collections::hash_map::DefaultHasher;

    let mut h = DefaultHasher::new();
    item.hash(&mut h);
    format!("{:x}", h.finish())
}
