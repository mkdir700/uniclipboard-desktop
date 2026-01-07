//! Test fixtures and helper functions for clipboard tests.

use crate::clipboard::*;
use crate::clipboard::meta_keys;
use std::collections::BTreeMap;

/// Creates a minimal ClipboardContent containing a single text item with the given string.
///
/// The returned content has a fixed version and timestamp and no additional metadata.
///
/// # Examples
///
/// ```
/// let content = create_text_clipboard("hello");
/// assert_eq!(content.items.len(), 1);
/// match &content.items[0].data {
///     ClipboardData::Text { text } => assert_eq!(text, "hello"),
///     _ => panic!("expected text data"),
/// }
/// ```
pub fn create_text_clipboard(text: &str) -> ClipboardContent {
    ClipboardContent {
        v: 1,
        ts_ms: 1000,
        items: vec![ClipboardItem {
            mime: MimeType::text_plain(),
            data: ClipboardData::Text { text: text.to_string() },
            meta: BTreeMap::new(),
        }],
        meta: BTreeMap::new(),
    }
}

/// Creates a ClipboardContent containing a single PNG image item.
///
/// The returned content has a fixed version and timestamp and one item with MIME type
/// "image/png" whose payload is the provided byte vector.
///
/// # Examples
///
/// ```
/// let bytes = vec![137, 80, 78]; // truncated PNG-like bytes
/// let content = create_image_clipboard(bytes.clone());
/// assert_eq!(content.items.len(), 1);
/// assert_eq!(content.items[0].mime.0, "image/png");
/// assert_eq!(content.items[0].data, ClipboardData::Bytes { bytes });
/// ```
pub fn create_image_clipboard(bytes: Vec<u8>) -> ClipboardContent {
    ClipboardContent {
        v: 1,
        ts_ms: 2000,
        items: vec![ClipboardItem {
            mime: MimeType("image/png".to_string()),
            data: ClipboardData::Bytes { bytes },
            meta: BTreeMap::new(),
        }],
        meta: BTreeMap::new(),
    }
}

/// Constructs a ClipboardContent containing two items: a plain-text item and an HTML text item.
///
/// # Examples
///
/// ```
/// let content = create_multi_item_clipboard();
/// assert_eq!(content.items.len(), 2);
/// assert!(matches!(content.items[0].data, ClipboardData::Text { .. }));
/// assert!(matches!(content.items[1].data, ClipboardData::Text { .. }));
/// ```
pub fn create_multi_item_clipboard() -> ClipboardContent {
    ClipboardContent {
        v: 1,
        ts_ms: 3000,
        items: vec![
            ClipboardItem {
                mime: MimeType::text_plain(),
                data: ClipboardData::Text { text: "Hello, world!".to_string() },
                meta: BTreeMap::new(),
            },
            ClipboardItem {
                mime: MimeType("text/html".to_string()),
                data: ClipboardData::Text { text: "<p>Hello, world!</p>".to_string() },
                meta: BTreeMap::new(),
            },
        ],
        meta: BTreeMap::new(),
    }
}

/// Creates a [`ClipboardDecisionSnapshot`] with the specified blobs_exist flag.
///
/// # Examples
///
/// ```
/// let _snapshot = create_snapshot(true);
/// ```
pub fn create_snapshot(blobs_exist: bool) -> ClipboardDecisionSnapshot {
    ClipboardDecisionSnapshot::new(blobs_exist)
}

/// Creates a `ClipboardContent` containing the provided metadata and no items.
///
/// The returned `ClipboardContent` uses a fixed version (`v = 1`) and timestamp (`ts_ms = 1000`),
/// and has an empty `items` list. The supplied `meta` map is stored as the content's metadata.
///
/// # Examples
///
/// ```
/// use std::collections::BTreeMap;
/// use uc_core::clipboard::ClipboardContent;
/// use uc_core::clipboard::tests::fixtures::create_content_with_meta;
///
/// let mut meta = BTreeMap::new();
/// meta.insert("origin".to_string(), "example".to_string());
///
/// let content = create_content_with_meta(meta.clone());
/// assert_eq!(content.v, 1);
/// assert_eq!(content.ts_ms, 1000);
/// assert!(content.items.is_empty());
/// assert_eq!(content.meta, meta);
/// ```
pub fn create_content_with_meta(meta: BTreeMap<String, String>) -> ClipboardContent {
    ClipboardContent {
        v: 1,
        ts_ms: 1000,
        items: vec![],
        meta,
    }
}

/// Create clipboard content with a device identifier in metadata.
///
/// The device identifier is stored under the key `meta_keys::sys::DEVICE_ID`.
///
/// # Examples
///
/// ```
/// let content = create_content_with_device_id("device-123");
/// assert_eq!(content.metadata.get(meta_keys::sys::DEVICE_ID).map(String::as_str), Some("device-123"));
/// ```
pub fn create_content_with_device_id(device_id: &str) -> ClipboardContent {
    let mut meta = BTreeMap::new();
    meta.insert(meta_keys::sys::DEVICE_ID.to_string(), device_id.to_string());
    create_content_with_meta(meta)
}

/// Creates a `ClipboardContent` whose metadata contains the provided origin.
///
/// The origin is stored under the key `meta_keys::sys::ORIGIN`.
///
/// # Examples
///
/// ```
/// let content = create_content_with_origin("https://example.com");
/// assert_eq!(
///     content.metadata.get(meta_keys::sys::ORIGIN).map(String::as_str),
///     Some("https://example.com")
/// );
/// ```
pub fn create_content_with_origin(origin: &str) -> ClipboardContent {
    let mut meta = BTreeMap::new();
    meta.insert(meta_keys::sys::ORIGIN.to_string(), origin.to_string());
    create_content_with_meta(meta)
}

/// Creates a `ClipboardItem` with the given MIME type and a zeroed data payload of the specified size in bytes.
///
/// The item's metadata will include the key `"sys.size_bytes"` with the decimal string representation of `size`.
///
/// # Examples
///
/// ```
/// let item = create_item_with_size(MimeType::from("text/plain"), 4);
/// if let ClipboardData::Bytes { bytes } = item.data {
///     assert_eq!(bytes.len(), 4);
/// } else {
///     panic!("expected bytes data");
/// }
/// assert_eq!(item.meta.get("sys.size_bytes").map(|s| s.as_str()), Some("4"));
/// ```
pub fn create_item_with_size(mime: MimeType, size: u64) -> ClipboardItem {
    let mut meta = BTreeMap::new();
    meta.insert("sys.size_bytes".to_string(), size.to_string());
    ClipboardItem {
        mime,
        data: ClipboardData::Bytes { bytes: vec![0; size as usize] },
        meta,
    }
}

/// Produces a byte vector containing a minimal PNG image (1×1 red pixel).
///
/// The returned data is a valid PNG file encoded as bytes and suitable for use
/// in tests that need a small binary image.
///
/// # Examples
///
/// ```
/// let png = sample_png_data();
/// // PNG files start with the 8-byte signature 0x89 0x50 0x4E 0x47 0x0D 0x0A 0x1A 0x0A
/// assert!(png.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]));
/// assert!(png.len() >= 16);
/// ```
pub fn sample_png_data() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, // IHDR length
        0x49, 0x48, 0x44, 0x52, // IHDR type
        0x00, 0x00, 0x00, 0x01, // width: 1
        0x00, 0x00, 0x00, 0x01, // height: 1
        0x08, 0x02, 0x00, 0x00, 0x00, // bit depth: 8, color type: 2 (RGB)
        0x90, 0x77, 0x53, 0xDE, // CRC
    ]
}