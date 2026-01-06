//! Tests for [`ClipboardContent`], [`ClipboardItem`], and [`ClipboardData`].

use super::fixtures::*;
use crate::clipboard::*;
use std::collections::BTreeMap;

#[test]
fn test_content_hash_deterministic() {
    let content = create_text_clipboard("hello");
    let hash1 = content.content_hash();
    let hash2 = content.content_hash();
    assert_eq!(hash1, hash2, "Hash should be deterministic");
}

#[test]
fn test_content_hash_different_items() {
    let content1 = create_text_clipboard("hello");
    let content2 = create_text_clipboard("world");
    assert_ne!(
        content1.content_hash(),
        content2.content_hash(),
        "Different content should produce different hashes"
    );
}

#[test]
fn test_content_hash_ignores_ts_ms() {
    let mut content1 = create_text_clipboard("test");
    let mut content2 = content1.clone();
    content2.ts_ms = 9999;
    assert_eq!(
        content1.content_hash(),
        content2.content_hash(),
        "Hash should ignore timestamp"
    );
}

#[test]
fn test_content_hash_ignores_meta() {
    let content1 = create_text_clipboard("test");
    let mut content2 = create_text_clipboard("test");
    content2.meta.insert("key".to_string(), "value".to_string());
    assert_eq!(
        content1.content_hash(),
        content2.content_hash(),
        "Hash should ignore metadata"
    );
}

#[test]
fn test_content_hash_ignores_empty_items() {
    let content1 = ClipboardContent {
        v: 1,
        ts_ms: 1000,
        items: vec![],
        meta: BTreeMap::new(),
    };
    let content2 = ClipboardContent {
        v: 1,
        ts_ms: 2000,
        items: vec![],
        meta: {
            let mut m = BTreeMap::new();
            m.insert("x".to_string(), "y".to_string());
            m
        },
    };
    assert_eq!(
        content1.content_hash(),
        content2.content_hash(),
        "Hash should be same for empty items regardless of meta/ts"
    );
}

#[test]
fn test_get_device_id_present() {
    let content = create_content_with_device_id("device-123");
    assert_eq!(content.get_device_id(), Some("device-123"));
}

#[test]
fn test_get_device_id_missing() {
    let content = create_text_clipboard("test");
    assert_eq!(content.get_device_id(), None);
}

#[test]
fn test_get_origin_present() {
    let content = create_content_with_origin("remote-device");
    assert_eq!(content.get_origin(), Some("remote-device"));
}

#[test]
fn test_get_origin_missing() {
    let content = create_text_clipboard("test");
    assert_eq!(content.get_origin(), None);
}

#[test]
fn test_item_data_len_text() {
    let item = ClipboardItem {
        mime: MimeType::text_plain(),
        data: ClipboardData::Text { text: "hello".to_string() },
        meta: BTreeMap::new(),
    };
    assert_eq!(item.data_len(), 5);
}

#[test]
fn test_item_data_len_text_unicode() {
    let item = ClipboardItem {
        mime: MimeType::text_plain(),
        data: ClipboardData::Text { text: "你好".to_string() },
        meta: BTreeMap::new(),
    };
    // UTF-8: 你 = 3 bytes, 好 = 3 bytes
    assert_eq!(item.data_len(), 6);
}

#[test]
fn test_item_data_len_text_empty() {
    let item = ClipboardItem {
        mime: MimeType::text_plain(),
        data: ClipboardData::Text { text: "".to_string() },
        meta: BTreeMap::new(),
    };
    assert_eq!(item.data_len(), 0);
}

#[test]
fn test_item_data_len_bytes() {
    let item = ClipboardItem {
        mime: MimeType("image/png".to_string()),
        data: ClipboardData::Bytes { bytes: vec![1, 2, 3, 4, 5] },
        meta: BTreeMap::new(),
    };
    assert_eq!(item.data_len(), 5);
}

#[test]
fn test_item_data_len_bytes_empty() {
    let item = ClipboardItem {
        mime: MimeType("image/png".to_string()),
        data: ClipboardData::Bytes { bytes: vec![] },
        meta: BTreeMap::new(),
    };
    assert_eq!(item.data_len(), 0);
}

#[test]
fn test_item_size_bytes_from_meta() {
    let item = create_item_with_size(MimeType("image/png".to_string()), 1024);
    assert_eq!(item.size_bytes(), Some(1024));
}

#[test]
fn test_item_size_bytes_missing() {
    let item = ClipboardItem {
        mime: MimeType::text_plain(),
        data: ClipboardData::Text { text: "test".to_string() },
        meta: BTreeMap::new(),
    };
    assert_eq!(item.size_bytes(), None);
}

#[test]
fn test_item_size_bytes_invalid_meta() {
    let mut meta = BTreeMap::new();
    meta.insert("sys.size_bytes".to_string(), "not-a-number".to_string());
    let item = ClipboardItem {
        mime: MimeType::text_plain(),
        data: ClipboardData::Text { text: "test".to_string() },
        meta,
    };
    assert_eq!(item.size_bytes(), None);
}

#[test]
fn test_multi_item_clipboard() {
    let content = create_multi_item_clipboard();
    assert_eq!(content.items.len(), 2);
    assert_eq!(content.items[0].mime.0, "text/plain");
    assert_eq!(content.items[1].mime.0, "text/html");
}

#[test]
fn test_image_clipboard() {
    let png_data = sample_png_data();
    let content = create_image_clipboard(png_data.clone());
    assert_eq!(content.items.len(), 1);
    assert_eq!(content.items[0].mime.0, "image/png");
    match &content.items[0].data {
        ClipboardData::Bytes { bytes } => assert_eq!(bytes, &png_data),
        ClipboardData::Text { .. } => panic!("Expected Bytes"),
    }
}

#[test]
fn test_clipboard_data_equality_text() {
    let data1 = ClipboardData::Text { text: "hello".to_string() };
    let data2 = ClipboardData::Text { text: "hello".to_string() };
    assert_eq!(data1, data2);
}

#[test]
fn test_clipboard_data_equality_bytes() {
    let data1 = ClipboardData::Bytes { bytes: vec![1, 2, 3] };
    let data2 = ClipboardData::Bytes { bytes: vec![1, 2, 3] };
    assert_eq!(data1, data2);
}

#[test]
fn test_clipboard_data_inequality() {
    let data1 = ClipboardData::Text { text: "hello".to_string() };
    let data2 = ClipboardData::Bytes { bytes: vec![104, 101, 108, 108, 111] };
    assert_ne!(data1, data2);
}

#[test]
fn test_clipboard_item_equality() {
    let item1 = ClipboardItem {
        mime: MimeType::text_plain(),
        data: ClipboardData::Text { text: "test".to_string() },
        meta: BTreeMap::new(),
    };
    let item2 = ClipboardItem {
        mime: MimeType::text_plain(),
        data: ClipboardData::Text { text: "test".to_string() },
        meta: BTreeMap::new(),
    };
    assert_eq!(item1, item2);
}

#[test]
fn test_clipboard_content_equality() {
    let content1 = create_text_clipboard("equal");
    let content2 = create_text_clipboard("equal");
    assert_eq!(content1, content2);
}
