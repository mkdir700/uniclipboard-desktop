//! Tests for view models: [`ClipboardOrigin`], [`ClipboardRecordId`],
//! [`ClipboardContentView`], [`ClipboardItemView`].

use crate::clipboard::view::*;
use chrono::{DateTime, Utc};
use serde_json;

#[test]
fn test_clipboard_origin_from_str_local() {
    let origin: ClipboardOrigin = "local".into();
    assert!(matches!(origin, ClipboardOrigin::Local));
}

#[test]
fn test_clipboard_origin_from_str_remote() {
    let origin: ClipboardOrigin = "remote".into();
    assert!(matches!(origin, ClipboardOrigin::Remote));
}

#[test]
fn test_clipboard_origin_from_str_unknown() {
    let origin: ClipboardOrigin = "unknown".into();
    assert!(matches!(origin, ClipboardOrigin::Local), "Unknown should default to Local");
}

#[test]
fn test_clipboard_origin_from_str_empty() {
    let origin: ClipboardOrigin = "".into();
    assert!(matches!(origin, ClipboardOrigin::Local), "Empty should default to Local");
}

#[test]
fn test_clipboard_origin_from_string_local() {
    let origin: ClipboardOrigin = "local".to_string().into();
    assert!(matches!(origin, ClipboardOrigin::Local));
}

#[test]
fn test_clipboard_origin_from_string_remote() {
    let origin: ClipboardOrigin = "remote".to_string().into();
    assert!(matches!(origin, ClipboardOrigin::Remote));
}

#[test]
fn test_clipboard_origin_serialization_local() {
    let origin = ClipboardOrigin::Local;
    let json = serde_json::to_string(&origin).unwrap();
    // Default enum serialization uses capitalized variant names
    assert_eq!(json, "\"Local\"");
}

#[test]
fn test_clipboard_origin_serialization_remote() {
    let origin = ClipboardOrigin::Remote;
    let json = serde_json::to_string(&origin).unwrap();
    // Default enum serialization uses capitalized variant names
    assert_eq!(json, "\"Remote\"");
}

#[test]
fn test_clipboard_origin_deserialization_local() {
    let json = "\"Local\"";
    let origin: ClipboardOrigin = serde_json::from_str(json).unwrap();
    assert!(matches!(origin, ClipboardOrigin::Local));
}

#[test]
fn test_clipboard_origin_deserialization_remote() {
    let json = "\"Remote\"";
    let origin: ClipboardOrigin = serde_json::from_str(json).unwrap();
    assert!(matches!(origin, ClipboardOrigin::Remote));
}

#[test]
fn test_clipboard_origin_roundtrip() {
    let origins = vec![ClipboardOrigin::Local, ClipboardOrigin::Remote];
    for origin in origins {
        let json = serde_json::to_string(&origin).unwrap();
        let deserialized: ClipboardOrigin = serde_json::from_str(&json).unwrap();
        assert_eq!(origin, deserialized);
    }
}

#[test]
fn test_clipboard_record_id_from_string() {
    let id = ClipboardRecordId::from("test-id".to_string());
    assert_eq!(id.0, "test-id");
}

#[test]
fn test_clipboard_record_id_from_str() {
    let id: ClipboardRecordId = "test-id".into();
    assert_eq!(id.0, "test-id");
}

#[test]
fn test_clipboard_record_id_clone() {
    let id1 = ClipboardRecordId("test-id".to_string());
    let id2 = id1.clone();
    assert_eq!(id1.0, id2.0);
}

#[test]
fn test_clipboard_item_view_new() {
    let view = ClipboardItemView {
        mime: Some("text/plain".to_string()),
        size: 100,
    };
    assert_eq!(view.mime, Some("text/plain".to_string()));
    assert_eq!(view.size, 100);
}

#[test]
fn test_clipboard_item_view_without_mime() {
    let view = ClipboardItemView {
        mime: None,
        size: 0,
    };
    assert_eq!(view.mime, None);
    assert_eq!(view.size, 0);
}

#[test]
fn test_clipboard_item_view_serialization() {
    let view = ClipboardItemView {
        mime: Some("text/plain".to_string()),
        size: 100,
    };
    let json = serde_json::to_string(&view).unwrap();
    assert!(json.contains("\"mime\":\"text/plain\""));
    assert!(json.contains("\"size\":100"));
}

#[test]
fn test_clipboard_content_view_new() {
    let created_at: DateTime<Utc> = "2024-01-01T00:00:00Z".parse().unwrap();
    let view = ClipboardContentView {
        id: ClipboardRecordId("test-id".to_string()),
        source_device_id: "device-123".to_string(),
        origin: ClipboardOrigin::Local,
        record_hash: "hash-123".to_string(),
        item_count: 2,
        items: vec![
            ClipboardItemView {
                mime: Some("text/plain".to_string()),
                size: 100,
            },
        ],
        created_at,
    };
    assert_eq!(view.id.0, "test-id");
    assert_eq!(view.source_device_id, "device-123");
    assert!(matches!(view.origin, ClipboardOrigin::Local));
    assert_eq!(view.record_hash, "hash-123");
    assert_eq!(view.item_count, 2);
    assert_eq!(view.items.len(), 1);
}

#[test]
fn test_clipboard_content_view_serialization() {
    let created_at: DateTime<Utc> = "2024-01-01T00:00:00Z".parse().unwrap();
    let view = ClipboardContentView {
        id: ClipboardRecordId("test-id".to_string()),
        source_device_id: "device-456".to_string(),
        origin: ClipboardOrigin::Remote,
        record_hash: "hash-456".to_string(),
        item_count: 1,
        items: vec![],
        created_at,
    };
    let json = serde_json::to_string(&view).unwrap();
    assert!(json.contains("\"id\":\"test-id\""));
    assert!(json.contains("\"source_device_id\":\"device-456\""));
    assert!(json.contains("\"origin\":\"Remote\""));
    assert!(json.contains("\"record_hash\":\"hash-456\""));
    assert!(json.contains("\"item_count\":1"));
}
