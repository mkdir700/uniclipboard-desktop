//! Clipboard representation materializer with owned config
//! 带有拥有所有权的配置的剪贴板表示物化器

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

use uc_core::clipboard::{MimeType, ObservedClipboardRepresentation, PersistedClipboardRepresentation};
use uc_core::ports::clipboard::ClipboardRepresentationMaterializerPort;
use crate::config::clipboard_storage_config::ClipboardStorageConfig;

const PREVIEW_LENGTH_CHARS: usize = 500;

/// Check if MIME type is text-based
/// 检查 MIME 类型是否为文本类型
pub(crate) fn is_text_mime_type(mime_type: &Option<MimeType>) -> bool {
    match mime_type {
        None => false,
        Some(mt) => {
            let mt_str = mt.as_str();
            mt_str.starts_with("text/") ||
            mt_str == "text/plain" ||
            mt_str.contains("json") ||
            mt_str.contains("xml") ||
            mt_str.contains("javascript") ||
            mt_str.contains("html") ||
            mt_str.contains("css")
        }
    }
}

/// UTF-8 safe truncation to first N characters
/// UTF-8 安全截断到前 N 个字符
pub(crate) fn truncate_to_preview(bytes: &[u8]) -> Vec<u8> {
    // UTF-8 safe truncation to first N characters
    std::str::from_utf8(bytes)
        .map(|text| {
            text.chars()
                .take(PREVIEW_LENGTH_CHARS)
                .collect::<String>()
                .into_bytes()
        })
        .unwrap_or_else(|_| {
            // Fallback for invalid UTF-8: truncate bytes
            bytes[..bytes.len().min(PREVIEW_LENGTH_CHARS)].to_vec()
        })
}

/// Clipboard representation materializer with owned config
/// 带有拥有所有权的配置的剪贴板表示物化器
///
/// Valid states:
/// 1. inline_data = Some, blob_id = None  -> inline payload
/// 2. inline_data = None, blob_id = Some  -> materialized blob
/// 3. inline_data = None, blob_id = None  -> lazy (metadata only)
/// 4. inline_data = Some, blob_id = Some  -> transitional / debugging
pub struct ClipboardRepresentationMaterializer {
    config: Arc<ClipboardStorageConfig>,
}

impl ClipboardRepresentationMaterializer {
    /// Create a new materializer with the given config
    /// 使用给定配置创建新物化器
    pub fn new(config: Arc<ClipboardStorageConfig>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ClipboardRepresentationMaterializerPort for ClipboardRepresentationMaterializer {
    async fn materialize(
        &self,
        observed: &ObservedClipboardRepresentation,
    ) -> Result<PersistedClipboardRepresentation> {
        let inline_threshold_bytes = self.config.inline_threshold_bytes;
        let size_bytes = observed.bytes.len() as i64;

        // Decision: inline or blob?
        // 决策：内联还是 blob？
        let inline_data = if size_bytes <= inline_threshold_bytes {
            Some(observed.bytes.clone())
        } else {
            None
        };

        Ok(PersistedClipboardRepresentation::new(
            observed.id.clone(),
            observed.format_id.clone(),
            observed.mime.clone(),
            size_bytes,
            inline_data,
            None, // blob_id will be set later by blob materializer
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_is_text_mime_type_with_text_plain() {
        assert!(is_text_mime_type(&Some(MimeType::text_plain())));
    }

    #[test]
    fn test_is_text_mime_type_with_json() {
        assert!(is_text_mime_type(&Some(MimeType::from_str("application/json").unwrap())));
    }

    #[test]
    fn test_is_text_mime_type_with_image() {
        assert!(!is_text_mime_type(&Some(MimeType::from_str("image/png").unwrap())));
    }

    #[test]
    fn test_is_text_mime_type_with_none() {
        assert!(!is_text_mime_type(&None));
    }

    // truncate_to_preview tests
    #[test]
    fn test_truncate_to_preview_ascii() {
        let input = b"h".repeat(5000); // 5000 bytes
        let result = truncate_to_preview(&input);
        assert_eq!(result.len(), 500); // 500 chars (ASCII)
        assert_eq!(String::from_utf8_lossy(&result), "h".repeat(500));
    }

    #[test]
    fn test_truncate_to_preview_utf8() {
        // Chinese characters are 3 bytes each in UTF-8
        let input = "你".repeat(1000).as_bytes().to_vec(); // 3000 bytes
        let result = truncate_to_preview(&input);
        assert_eq!(String::from_utf8_lossy(&result), "你".repeat(500)); // 500 chars = 500 * 3 = 1500 bytes
        assert_eq!(result.len(), 1500); // 500 chars * 3 bytes each
    }

    #[test]
    fn test_truncate_to_preview_shorter_than_limit() {
        let input = b"short";
        let result = truncate_to_preview(input);
        assert_eq!(result, b"short");
    }

    #[test]
    fn test_truncate_to_preview_invalid_utf8() {
        let input = vec![0xFF, 0xFE, 0xFD]; // Invalid UTF-8
        let result = truncate_to_preview(&input);
        // Fallback to byte truncation
        assert_eq!(result.len(), 3);
    }
}
