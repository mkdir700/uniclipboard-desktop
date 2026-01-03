//! Clipboard data models
//!
//! Pure data structures for clipboard content without infrastructure dependencies.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// Clipboard content type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    Text,
    Image,
    Link,
    File,
    CodeSnippet,
    RichText,
}

impl ContentType {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Text => "text",
            ContentType::Image => "image",
            ContentType::Link => "link",
            ContentType::File => "file",
            ContentType::CodeSnippet => "code",
            ContentType::RichText => "rich_text",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "text" => Some(ContentType::Text),
            "image" => Some(ContentType::Image),
            "link" => Some(ContentType::Link),
            "file" => Some(ContentType::File),
            "code" => Some(ContentType::CodeSnippet),
            "rich_text" => Some(ContentType::RichText),
            _ => None,
        }
    }
}

impl Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Text clipboard item
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextItem {
    /// Display text content
    pub display_text: String,
    /// Whether the text was truncated
    pub is_truncated: bool,
    /// Content size in bytes
    pub size: usize,
}

/// Image clipboard item
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageItem {
    /// Base64-encoded thumbnail image
    pub thumbnail: String,
    /// Content size in bytes
    pub size: usize,
    /// Image width in pixels
    pub width: usize,
    /// Image height in pixels
    pub height: usize,
}

/// File clipboard item
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileItem {
    /// File names
    pub file_names: Vec<String>,
    /// File sizes in bytes
    pub file_sizes: Vec<usize>,
}

impl FileItem {
    /// Get total size of all files
    pub fn total_size(&self) -> usize {
        self.file_sizes.iter().sum()
    }

    /// Get number of files
    pub fn file_count(&self) -> usize {
        self.file_names.len()
    }
}

/// Link clipboard item
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LinkItem {
    /// URL
    pub url: String,
}

/// Code snippet clipboard item
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CodeItem {
    /// Code content
    pub code: String,
}

/// Clipboard item (content variant)
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ClipboardItem {
    Text(TextItem),
    Image(ImageItem),
    File(FileItem),
    Link(LinkItem),
    Code(CodeItem),
}

impl ClipboardItem {
    /// Get the content type of this item
    pub fn content_type(&self) -> ContentType {
        match self {
            ClipboardItem::Text(_) => ContentType::Text,
            ClipboardItem::Image(_) => ContentType::Image,
            ClipboardItem::File(_) => ContentType::File,
            ClipboardItem::Link(_) => ContentType::Link,
            ClipboardItem::Code(_) => ContentType::CodeSnippet,
        }
    }

    /// Get the size of this item in bytes
    pub fn size(&self) -> usize {
        match self {
            ClipboardItem::Text(item) => item.size,
            ClipboardItem::Image(item) => item.size,
            ClipboardItem::File(item) => item.total_size(),
            ClipboardItem::Link(item) => item.url.len(),
            ClipboardItem::Code(item) => item.code.len(),
        }
    }
}

/// Clipboard metadata (stored in database)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClipboardMetadata {
    Text(TextMetadata),
    Image(ImageMetadata),
    Link(LinkMetadata),
    File(FileMetadata),
    CodeSnippet(CodeMetadata),
    RichText(RichTextMetadata),
}

impl ClipboardMetadata {
    /// Get device ID
    pub fn device_id(&self) -> &str {
        match self {
            ClipboardMetadata::Text(meta) => &meta.device_id,
            ClipboardMetadata::Image(meta) => &meta.device_id,
            ClipboardMetadata::Link(meta) => &meta.device_id,
            ClipboardMetadata::File(meta) => &meta.device_id,
            ClipboardMetadata::CodeSnippet(meta) => &meta.device_id,
            ClipboardMetadata::RichText(meta) => &meta.device_id,
        }
    }

    /// Get timestamp
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            ClipboardMetadata::Text(meta) => meta.timestamp,
            ClipboardMetadata::Image(meta) => meta.timestamp,
            ClipboardMetadata::Link(meta) => meta.timestamp,
            ClipboardMetadata::File(meta) => meta.timestamp,
            ClipboardMetadata::CodeSnippet(meta) => meta.timestamp,
            ClipboardMetadata::RichText(meta) => meta.timestamp,
        }
    }

    /// Get content type
    pub fn content_type(&self) -> ContentType {
        match self {
            ClipboardMetadata::Text(_) => ContentType::Text,
            ClipboardMetadata::Image(_) => ContentType::Image,
            ClipboardMetadata::Link(_) => ContentType::Link,
            ClipboardMetadata::File(_) => ContentType::File,
            ClipboardMetadata::CodeSnippet(_) => ContentType::CodeSnippet,
            ClipboardMetadata::RichText(_) => ContentType::RichText,
        }
    }

    /// Get content hash
    pub fn content_hash(&self) -> u64 {
        match self {
            ClipboardMetadata::Text(meta) => meta.content_hash,
            ClipboardMetadata::Image(meta) => meta.content_hash,
            ClipboardMetadata::Link(meta) => meta.content_hash,
            ClipboardMetadata::File(meta) => meta.content_hash,
            ClipboardMetadata::CodeSnippet(meta) => meta.content_hash,
            ClipboardMetadata::RichText(meta) => meta.content_hash,
        }
    }

    /// Get content size
    pub fn size(&self) -> usize {
        match self {
            ClipboardMetadata::Text(meta) => meta.size,
            ClipboardMetadata::Image(meta) => meta.size,
            ClipboardMetadata::Link(meta) => meta.size,
            ClipboardMetadata::File(meta) => meta.total_size(),
            ClipboardMetadata::CodeSnippet(meta) => meta.size,
            ClipboardMetadata::RichText(meta) => meta.size,
        }
    }

    /// Get storage path
    pub fn storage_path(&self) -> &str {
        match self {
            ClipboardMetadata::Text(meta) => &meta.storage_path,
            ClipboardMetadata::Image(meta) => &meta.storage_path,
            ClipboardMetadata::Link(meta) => &meta.storage_path,
            ClipboardMetadata::File(meta) => &meta.storage_path,
            ClipboardMetadata::CodeSnippet(meta) => &meta.storage_path,
            ClipboardMetadata::RichText(meta) => &meta.storage_path,
        }
    }

    /// Get unique key for this metadata
    pub fn key(&self) -> String {
        match self {
            ClipboardMetadata::Text(meta) => format!("{:016x}", meta.content_hash),
            ClipboardMetadata::Image(meta) => {
                let size_info = format!("{}x{}", meta.width, meta.height);
                format!("img_{:016x}_{}", meta.content_hash, size_info)
            }
            ClipboardMetadata::Link(meta) => format!("{:016x}", meta.content_hash),
            ClipboardMetadata::File(meta) => format!("{:016x}", meta.content_hash),
            ClipboardMetadata::CodeSnippet(meta) => format!("{:016x}", meta.content_hash),
            ClipboardMetadata::RichText(meta) => format!("{:016x}", meta.content_hash),
        }
    }

    /// Check if two metadata represent the same content
    pub fn is_duplicate(&self, other: &ClipboardMetadata) -> bool {
        match (self, other) {
            (ClipboardMetadata::Text(t1), ClipboardMetadata::Text(t2)) => {
                t1.content_hash == t2.content_hash
            }
            (ClipboardMetadata::Image(i1), ClipboardMetadata::Image(i2)) => {
                i1.content_hash == i2.content_hash
                    && i1.width == i2.width
                    && i1.height == i2.height
                    && (i1.size as f64 - i2.size as f64).abs() / (i1.size as f64) <= 0.1
            }
            (ClipboardMetadata::Link(l1), ClipboardMetadata::Link(l2)) => {
                l1.content_hash == l2.content_hash
            }
            (ClipboardMetadata::File(f1), ClipboardMetadata::File(f2)) => {
                f1.content_hash == f2.content_hash
            }
            (ClipboardMetadata::CodeSnippet(c1), ClipboardMetadata::CodeSnippet(c2)) => {
                c1.content_hash == c2.content_hash
            }
            (ClipboardMetadata::RichText(r1), ClipboardMetadata::RichText(r2)) => {
                r1.content_hash == r2.content_hash
            }
            _ => false,
        }
    }
}

impl Display for ClipboardMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClipboardMetadata[key={}, type={}, device={}, time={}]",
            self.key(),
            self.content_type(),
            self.device_id(),
            self.timestamp().format("%Y-%m-%d %H:%M:%S")
        )
    }
}

/// Text metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextMetadata {
    pub content_hash: u64,
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
    pub length: usize,
    pub size: usize,
    pub storage_path: String,
}

/// Image metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub content_hash: u64,
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
    pub width: usize,
    pub height: usize,
    pub format: String,
    pub size: usize,
    pub storage_path: String,
}

/// File metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub content_hash: u64,
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
    pub file_names: Vec<String>,
    pub file_sizes: Vec<usize>,
    pub storage_path: String,
}

impl FileMetadata {
    pub fn total_size(&self) -> usize {
        self.file_sizes.iter().sum()
    }

    pub fn file_count(&self) -> usize {
        self.file_names.len()
    }
}

/// Link metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkMetadata {
    pub content_hash: u64,
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
    pub length: usize,
    pub size: usize,
    pub storage_path: String,
}

/// Code snippet metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMetadata {
    pub content_hash: u64,
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
    pub length: usize,
    pub size: usize,
    pub storage_path: String,
}

/// Rich text metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichTextMetadata {
    pub content_hash: u64,
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
    pub length: usize,
    pub size: usize,
    pub storage_path: String,
}

/// Clipboard statistics
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClipboardStats {
    pub total_items: usize,
    pub total_size: usize,
}

impl ClipboardStats {
    /// Create new stats
    pub fn new(total_items: usize, total_size: usize) -> Self {
        Self {
            total_items,
            total_size,
        }
    }

    /// Calculate average item size
    pub fn average_size(&self) -> Option<usize> {
        if self.total_items > 0 {
            Some(self.total_size / self.total_items)
        } else {
            None
        }
    }
}

/// Clipboard item response (with metadata)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClipboardItemResponse {
    pub id: String,
    pub device_id: String,
    pub is_downloaded: bool,
    pub is_favorited: bool,
    pub created_at: i32,
    pub updated_at: i32,
    pub active_time: i32,
    pub item: ClipboardItem,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_conversion() {
        assert_eq!(ContentType::Text.as_str(), "text");
        assert_eq!(ContentType::from_str("image"), Some(ContentType::Image));
        assert_eq!(ContentType::from_str("invalid"), None);
    }

    #[test]
    fn test_clipboard_item_size() {
        let text_item = ClipboardItem::Text(TextItem {
            display_text: "Hello".to_string(),
            is_truncated: false,
            size: 5,
        });
        assert_eq!(text_item.size(), 5);

        let file_item = ClipboardItem::File(FileItem {
            file_names: vec!["a.txt".to_string(), "b.txt".to_string()],
            file_sizes: vec![100, 200],
        });
        assert_eq!(file_item.size(), 300);
    }

    #[test]
    fn test_clipboard_metadata_key() {
        let text_meta = TextMetadata {
            content_hash: 0x1234,
            device_id: "device1".to_string(),
            timestamp: Utc::now(),
            length: 10,
            size: 10,
            storage_path: "/path/to/file".to_string(),
        };

        let metadata = ClipboardMetadata::Text(text_meta);
        assert_eq!(metadata.key(), "0000000000001234");
    }

    #[test]
    fn test_clipboard_metadata_duplicate() {
        let meta1 = ClipboardMetadata::Text(TextMetadata {
            content_hash: 0x1234,
            device_id: "device1".to_string(),
            timestamp: Utc::now(),
            length: 10,
            size: 10,
            storage_path: "/path/1".to_string(),
        });

        let meta2 = ClipboardMetadata::Text(TextMetadata {
            content_hash: 0x1234,
            device_id: "device1".to_string(),
            timestamp: Utc::now(),
            length: 10,
            size: 10,
            storage_path: "/path/2".to_string(),
        });

        assert!(meta1.is_duplicate(&meta2));
    }

    #[test]
    fn test_clipboard_stats_average() {
        let stats = ClipboardStats::new(10, 1000);
        assert_eq!(stats.average_size(), Some(100));

        let empty = ClipboardStats::new(0, 0);
        assert_eq!(empty.average_size(), None);
    }

    #[test]
    fn test_serialization() {
        let item = ClipboardItem::Text(TextItem {
            display_text: "Hello, World!".to_string(),
            is_truncated: false,
            size: 13,
        });

        let json = serde_json::to_string(&item).unwrap();
        let deserialized: ClipboardItem = serde_json::from_str(&json).unwrap();

        match deserialized {
            ClipboardItem::Text(text) => {
                assert_eq!(text.display_text, "Hello, World!");
                assert!(!text.is_truncated);
                assert_eq!(text.size, 13);
            }
            _ => panic!("Wrong type"),
        }
    }
}
