use crate::db::models::{
    ClipboardItemRow, ClipboardRecordRow, NewClipboardItemRowOwned, NewClipboardRecordRowOwned,
};
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use uc_core::clipboard::meta_keys;
use uc_core::clipboard::{ClipboardContent, ClipboardData, ClipboardItem, MimeType};
use uuid::Uuid;

impl From<(&ClipboardItem, &str, &str, i32)> for NewClipboardItemRowOwned {
    /// Create a NewClipboardItemRowOwned from a ClipboardItem, a record ID, a blob ID, and an index.
    ///
    /// The resulting record has a freshly generated `id`, `record_id` and `index_in_record` taken from the inputs, `content_type` and `mime` derived from the item's MIME type, `content_hash` computed for the item, `blob_id` set to the provided blob identifier, and `size` set when `item.size_bytes()` returns a value.
    ///
    /// # Examples
    ///
    /// ```
    /// // assumes `ClipboardItem` and `NewClipboardItemRowOwned` are in scope and constructible
    /// let item = ClipboardItem::new("text/plain", b"hello"); // example constructor
    /// let row = NewClipboardItemRowOwned::from((&item, "record-1", "blob-1", 0));
    /// assert_eq!(row.record_id, "record-1");
    /// assert_eq!(row.blob_id.as_deref(), Some("blob-1"));
    /// ```
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
    /// Construct a NewClipboardRecordRowOwned from a ClipboardContent, deriving device and origin with `"unknown"` fallbacks.
    ///
    /// The created record uses a new UUID for `id`, `source_device_id` and `origin` taken from the content (or `"unknown"` if missing), `record_hash` from `content.content_hash()`, `item_count` equal to the number of items, `created_at` from `content.ts_ms`, and `deleted_at` set to `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// // Build or obtain a ClipboardContent named `content`
    /// let row = NewClipboardRecordRowOwned::from(&content);
    /// assert_eq!(row.record_hash, content.content_hash());
    /// assert_eq!(row.item_count as usize, content.items.len());
    /// ```
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

/// Build a ClipboardItem from a database row and externally provided clipboard data.
///
/// The function uses metadata from `row` (size and blob id) and the supplied `data` to
/// construct a ClipboardItem; it does not access any external store itself.
///
/// # Examples
///
/// ```ignore
/// // Prepare a ClipboardItemRow (fields omitted for brevity) and ClipboardData,
/// // then convert them into a ClipboardItem:
/// let row = ClipboardItemRow { size: Some(123), blob_id: Some("blob-uuid".into()), mime: Some("text/plain".into()), content_type: "text/plain".into(), ..Default::default() };
/// let data = ClipboardData::Bytes(b"hello".to_vec());
/// let item = map_item_row_to_item(&row, data);
/// assert_eq!(item.mime.0, "text/plain");
/// assert_eq!(item.meta.get(&meta_keys::sys::SIZE_BYTES.to_string()).unwrap(), "123");
/// ```
/// This is a helper function rather than a `From` impl because the actual
/// clipboard data is stored externally (referenced by `blob_id`), and
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