//! Test fixtures and helper functions for clipboard tests.

use crate::clipboard::*;
use crate::clipboard::meta_keys;
use std::collections::BTreeMap;

/// Creates a minimal [`ClipboardContent`] with a single text item.
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

/// Creates a [`ClipboardContent`] with an image item (PNG).
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

/// Creates a [`ClipboardContent`] with multiple items.
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
pub fn create_snapshot(blobs_exist: bool) -> ClipboardDecisionSnapshot {
    ClipboardDecisionSnapshot::new(blobs_exist)
}

/// Creates a [`ClipboardContent`] with custom metadata.
pub fn create_content_with_meta(meta: BTreeMap<String, String>) -> ClipboardContent {
    ClipboardContent {
        v: 1,
        ts_ms: 1000,
        items: vec![],
        meta,
    }
}

/// Creates a [`ClipboardContent`] with device ID in metadata.
pub fn create_content_with_device_id(device_id: &str) -> ClipboardContent {
    let mut meta = BTreeMap::new();
    meta.insert(meta_keys::sys::DEVICE_ID.to_string(), device_id.to_string());
    create_content_with_meta(meta)
}

/// Creates a [`ClipboardContent`] with origin in metadata.
pub fn create_content_with_origin(origin: &str) -> ClipboardContent {
    let mut meta = BTreeMap::new();
    meta.insert(meta_keys::sys::ORIGIN.to_string(), origin.to_string());
    create_content_with_meta(meta)
}

/// Creates a [`ClipboardItem`] with size metadata.
pub fn create_item_with_size(mime: MimeType, size: u64) -> ClipboardItem {
    let mut meta = BTreeMap::new();
    meta.insert("sys.size_bytes".to_string(), size.to_string());
    ClipboardItem {
        mime,
        data: ClipboardData::Bytes { bytes: vec![0; size as usize] },
        meta,
    }
}

/// Creates a sample PNG image data (1x1 red pixel).
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
