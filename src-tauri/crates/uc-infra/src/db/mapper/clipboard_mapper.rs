use crate::db::models::{
    ClipboardItemRow, ClipboardRecordRow, NewClipboardItemRowOwned, NewClipboardRecordRowOwned,
};
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use uc_core::clipboard::meta_keys;
use uc_core::clipboard::{ClipboardContent, ClipboardData, ClipboardItem, MimeType};
use uuid::Uuid;

impl From<(&ClipboardItem, &str, &str, i32)> for NewClipboardItemRowOwned {
    fn from((item, record_id, blob_id, index): (&ClipboardItem, &str, &str, i32)) -> Self {
        let size = item.size_bytes().and_then(|v| v.try_into().ok());

        NewClipboardItemRowOwned {
            id: Uuid::new_v4().to_string(),
            record_id: record_id.to_string(),
            index_in_record: index,
            content_type: item.mime.to_string(),
            content_hash: item_hash(item),
            blob_id: Some(blob_id.to_string()),
            size,
            mime: Some(item.mime.to_string()),
        }
    }
}

impl From<&ClipboardContent> for NewClipboardRecordRowOwned {
    fn from(content: &ClipboardContent) -> Self {
        let device_id = content.get_device_id().unwrap_or("unknown");
        let origin = content.get_origin().unwrap_or("unknown");
        NewClipboardRecordRowOwned {
            id: Uuid::new_v4().to_string(),
            source_device_id: device_id.to_string(),
            origin: origin.to_string(),
            record_hash: content.content_hash(),
            item_count: content.items.len() as i32,
            created_at: content.ts_ms,
            deleted_at: None,
        }
    }
}

/// Maps a database row to a ClipboardItem.
///
/// This is a helper function rather than a `From` impl because the actual
/// clipboard data is stored externally (referenced by `store_path`), and
/// must be provided separately.
pub fn map_item_row_to_item(row: &ClipboardItemRow, data: ClipboardData) -> ClipboardItem {
    let mut meta = BTreeMap::new();
    if let Some(size) = row.size {
        meta.insert(meta_keys::sys::SIZE_BYTES.to_string(), size.to_string());
    }
    if let Some(blob_id) = &row.blob_id {
        meta.insert(meta_keys::sys::BLOB_ID.to_string(), blob_id.clone());
    }

    ClipboardItem {
        mime: MimeType(row.mime.clone().unwrap_or_else(|| row.content_type.clone())),
        data,
        meta,
    }
}

/// Creates a ClipboardContent from database rows.
///
/// This is a helper function that creates the structure from metadata,
/// but the actual ClipboardItem data must be loaded separately using
/// `map_item_row_to_item`.
pub fn map_record_row_to_content(record_row: &ClipboardRecordRow) -> ClipboardContent {
    ClipboardContent {
        v: 1,
        ts_ms: record_row.created_at,
        items: Vec::new(), // Items must be loaded separately
        meta: BTreeMap::new(),
    }
}

fn item_hash(item: &ClipboardItem) -> String {
    use std::collections::hash_map::DefaultHasher;

    let mut h = DefaultHasher::new();
    item.hash(&mut h);
    format!("{:x}", h.finish())
}
