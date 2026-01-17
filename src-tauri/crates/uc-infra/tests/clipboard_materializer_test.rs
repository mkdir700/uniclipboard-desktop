//! Clipboard Representation Materializer Tests
//! 剪贴板表示物化器测试

use uc_core::clipboard::{ObservedClipboardRepresentation};
use uc_core::ids::{RepresentationId, FormatId};
use uc_core::MimeType;
use uc_core::ports::ClipboardRepresentationMaterializerPort;
use uc_infra::clipboard::ClipboardRepresentationMaterializer;
use uc_infra::config::ClipboardStorageConfig;
use std::sync::Arc;

#[tokio::test]
async fn test_materialize_small_data_inline() {
    let config = Arc::new(ClipboardStorageConfig {
        inline_threshold_bytes: 16 * 1024, // 16 KB
    });
    let materializer = ClipboardRepresentationMaterializer::new(config);

    let observed = ObservedClipboardRepresentation {
        id: RepresentationId::from("test-rep-1".to_string()),
        format_id: FormatId::from("public.utf8-plain-text".to_string()),
        mime: Some(MimeType("text/plain".to_string())),
        bytes: b"Hello, World!".to_vec(),
    };

    let result = materializer.materialize(&observed).await.unwrap();

    assert_eq!(result.id.to_string(), "test-rep-1");
    assert_eq!(result.size_bytes, 13);
    assert!(result.inline_data.is_some(), "Small data should be inline");
    assert_eq!(result.inline_data.unwrap(), b"Hello, World!".to_vec());
    assert!(result.blob_id.is_none(), "Small data should not have blob_id");
}

#[tokio::test]
async fn test_materialize_large_data_not_inline() {
    let config = Arc::new(ClipboardStorageConfig {
        inline_threshold_bytes: 1024, // 1 KB threshold
    });
    let materializer = ClipboardRepresentationMaterializer::new(config);

    // Create 2KB of data
    let large_data = vec![0u8; 2048];
    let observed = ObservedClipboardRepresentation {
        id: RepresentationId::from("test-rep-2".to_string()),
        format_id: FormatId::from("public.png".to_string()),
        mime: Some(MimeType("image/png".to_string())),
        bytes: large_data.clone(),
    };

    let result = materializer.materialize(&observed).await.unwrap();

    assert_eq!(result.id.to_string(), "test-rep-2");
    assert_eq!(result.size_bytes, 2048);
    assert!(result.inline_data.is_none(), "Large data should NOT be inline");
    assert!(result.blob_id.is_none(), "blob_id will be set by blob materializer later");
}

#[tokio::test]
async fn test_materialize_exactly_at_threshold() {
    let config = Arc::new(ClipboardStorageConfig {
        inline_threshold_bytes: 100, // exactly 100 bytes
    });
    let materializer = ClipboardRepresentationMaterializer::new(config);

    // Create exactly 100 bytes
    let exact_data = vec![42u8; 100];
    let observed = ObservedClipboardRepresentation {
        id: RepresentationId::from("exact-threshold".to_string()),
        format_id: FormatId::from("test.format".to_string()),
        mime: None,
        bytes: exact_data,
    };

    let result = materializer.materialize(&observed).await.unwrap();

    assert!(result.inline_data.is_some(), "Data at threshold should be inline");
    assert_eq!(result.inline_data.unwrap().len(), 100);
}

#[tokio::test]
async fn test_materialize_one_byte_over_threshold() {
    let config = Arc::new(ClipboardStorageConfig {
        inline_threshold_bytes: 100, // 100 bytes threshold
    });
    let materializer = ClipboardRepresentationMaterializer::new(config);

    // Create 101 bytes (one over threshold)
    let over_data = vec![42u8; 101];
    let observed = ObservedClipboardRepresentation {
        id: RepresentationId::from("over-threshold".to_string()),
        format_id: FormatId::from("test.format".to_string()),
        mime: None,
        bytes: over_data,
    };

    let result = materializer.materialize(&observed).await.unwrap();

    assert!(result.inline_data.is_none(), "Data over threshold should NOT be inline");
}

#[tokio::test]
async fn test_materialize_large_text_creates_inline_preview() {
    let config = Arc::new(ClipboardStorageConfig {
        inline_threshold_bytes: 16 * 1024, // 16 KB
    });
    let materializer = ClipboardRepresentationMaterializer::new(config);

    // Create text larger than inline threshold (16KB)
    let large_text = "x".repeat(20000);
    let observed = ObservedClipboardRepresentation {
        id: RepresentationId::from("test-rep-large-text".to_string()),
        format_id: FormatId::from("public.utf8-plain-text".to_string()),
        mime: Some(MimeType("text/plain".to_string())),
        bytes: large_text.as_bytes().to_vec(),
    };

    let result = materializer.materialize(&observed).await.unwrap();

    // Should have both inline preview AND no blob_id (blob_id set later)
    assert!(result.inline_data.is_some(), "Should have inline preview");
    assert!(result.blob_id.is_none(), "blob_id will be set by blob materializer later");

    // Inline should be truncated to 500 chars
    let inline_text = String::from_utf8(result.inline_data.unwrap()).unwrap();
    assert_eq!(inline_text.len(), 500);
    assert_eq!(inline_text, "x".repeat(500));
}

#[tokio::test]
async fn test_materialize_large_image_no_inline() {
    let config = Arc::new(ClipboardStorageConfig {
        inline_threshold_bytes: 16 * 1024, // 16 KB
    });
    let materializer = ClipboardRepresentationMaterializer::new(config);

    // Create image data larger than inline threshold
    let large_image = vec![0u8; 20000];
    let observed = ObservedClipboardRepresentation {
        id: RepresentationId::from("test-rep-large-image".to_string()),
        format_id: FormatId::from("public.png".to_string()),
        mime: Some(MimeType("image/png".to_string())),
        bytes: large_image.clone(),
    };

    let result = materializer.materialize(&observed).await.unwrap();

    // Should have NO inline data, only blob (blob_id set later)
    assert!(result.inline_data.is_none(), "Large images should not have inline data");
    assert!(result.blob_id.is_none(), "blob_id will be set by blob materializer later");
    assert_eq!(result.size_bytes, 20000);
}
