use std::{fmt::Display, path::Path};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use twox_hash::xxh3::hash64;

use crate::message::Payload;

/// 剪贴板内容类型
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
    /// 获取内容类型的字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Text => "text",
            ContentType::Image => "image",
            ContentType::Link => "link",
            ContentType::File => "file",
            ContentType::CodeSnippet => "code_snippet",
            ContentType::RichText => "rich_text",
        }
    }
    
    /// 从字符串解析内容类型
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "text" => Some(ContentType::Text),
            "image" => Some(ContentType::Image),
            "link" => Some(ContentType::Link),
            "file" => Some(ContentType::File),
            "code_snippet" => Some(ContentType::CodeSnippet),
            "rich_text" => Some(ContentType::RichText),
            _ => None,
        }
    }
}

impl Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 剪贴板内容元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClipboardMetadata {
    Text(TextMetadata),
    Image(ImageMetadata),
    Link(TextMetadata),
    File(TextMetadata),
    CodeSnippet(TextMetadata),
    RichText(TextMetadata),
}

/// 文本内容元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextMetadata {
    /// 内容哈希值
    pub content_hash: u64,
    /// 设备ID
    pub device_id: String,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 文本长度
    pub length: usize,
    /// 存储路径
    pub storage_path: String,
}

/// 图片内容元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    /// 内容哈希值
    pub content_hash: u64,
    /// 设备ID
    pub device_id: String,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 宽度
    pub width: usize,
    /// 高度
    pub height: usize,
    /// 格式
    pub format: String,
    /// 大小
    pub size: usize,
    /// 存储路径
    pub storage_path: String,
}

impl ClipboardMetadata {
    /// 从 Payload 创建元数据
    pub fn from_payload(payload: &Payload, storage_path: &Path) -> Self {
        match payload {
            Payload::Text(text) => {
                ClipboardMetadata::Text(TextMetadata {
                    content_hash: hash64(&text.content),
                    device_id: text.device_id.clone(),
                    timestamp: text.timestamp,
                    length: text.content.len(),
                    storage_path: storage_path.to_string_lossy().to_string(),
                })
            }
            Payload::Image(image) => {
                ClipboardMetadata::Image(ImageMetadata {
                    content_hash: image.content_hash(),
                    device_id: image.device_id.clone(),
                    timestamp: image.timestamp,
                    width: image.width,
                    height: image.height,
                    format: image.format.clone(),
                    size: image.size,
                    storage_path: storage_path.to_string_lossy().to_string(),
                })
            }
            // Payload::Link(link) => {
            //     ClipboardMetadata::Link(TextMetadata {
            //         content_hash: hash64(&link.content),
            //         device_id: link.device_id.clone(),
            //         timestamp: link.timestamp,
            //         length: link.content.len(),
            //         storage_path: storage_path.to_string_lossy().to_string(),
            //     })
            // }
            // Payload::File(file) => {
            //     ClipboardMetadata::File(TextMetadata {
            //         content_hash: hash64(&file.content),
            //         device_id: file.device_id.clone(),
            //         timestamp: file.timestamp,
            //         length: file.content.len(),
            //         storage_path: storage_path.to_string_lossy().to_string(),
            //     })
            // }
            // Payload::CodeSnippet(code) => {
            //     ClipboardMetadata::CodeSnippet(TextMetadata {
            //         content_hash: hash64(&code.content),
            //         device_id: code.device_id.clone(),
            //         timestamp: code.timestamp,
            //         length: code.content.len(),
            //         storage_path: storage_path.to_string_lossy().to_string(),
            //     })
            // }
            // Payload::RichText(rich_text) => {
            //     ClipboardMetadata::RichText(TextMetadata {
            //         content_hash: hash64(&rich_text.content),
            //         device_id: rich_text.device_id.clone(),
            //         timestamp: rich_text.timestamp,
            //         length: rich_text.content.len(),
            //         storage_path: storage_path.to_string_lossy().to_string(),
            //     })
            // }
        }
    }

    /// 获取设备ID
    pub fn get_device_id(&self) -> &str {
        match self {
            ClipboardMetadata::Text(text) => &text.device_id,
            ClipboardMetadata::Image(image) => &image.device_id,
            ClipboardMetadata::Link(link) => &link.device_id,
            ClipboardMetadata::File(file) => &file.device_id,
            ClipboardMetadata::CodeSnippet(code) => &code.device_id,
            ClipboardMetadata::RichText(rich_text) => &rich_text.device_id,
        }
    }

    /// 获取时间戳
    pub fn get_timestamp(&self) -> DateTime<Utc> {
        match self {
            ClipboardMetadata::Text(text) => text.timestamp,
            ClipboardMetadata::Image(image) => image.timestamp,
            ClipboardMetadata::Link(link) => link.timestamp,
            ClipboardMetadata::File(file) => file.timestamp,
            ClipboardMetadata::CodeSnippet(code) => code.timestamp,
            ClipboardMetadata::RichText(rich_text) => rich_text.timestamp,
        }
    }

    /// 获取内容类型
    pub fn get_content_type(&self) -> ContentType {
        match self {
            ClipboardMetadata::Text(_) => ContentType::Text,
            ClipboardMetadata::Image(_) => ContentType::Image,
            ClipboardMetadata::Link(_) => ContentType::Link,
            ClipboardMetadata::File(_) => ContentType::File,
            ClipboardMetadata::CodeSnippet(_) => ContentType::CodeSnippet,
            ClipboardMetadata::RichText(_) => ContentType::RichText,
        }
    }

    /// 获取内容哈希值
    pub fn get_content_hash(&self) -> u64 {
        match self {
            ClipboardMetadata::Text(text) => text.content_hash,
            ClipboardMetadata::Image(image) => image.content_hash,
            ClipboardMetadata::Link(link) => link.content_hash,
            ClipboardMetadata::File(file) => file.content_hash,
            ClipboardMetadata::CodeSnippet(code) => code.content_hash,
            ClipboardMetadata::RichText(rich_text) => rich_text.content_hash,
        }
    }

    /// 获取唯一标识符
    pub fn get_key(&self) -> String {
        match self {
            ClipboardMetadata::Text(text) => {
                format!("{:016x}", text.content_hash)
            }
            ClipboardMetadata::Image(image) => {
                // 使用图片内容哈希 + 尺寸信息作为唯一标识符
                let size_info = format!("{}x{}", image.width, image.height);
                format!("img_{:016x}_{}", image.content_hash, size_info)
            }
            ClipboardMetadata::Link(link) => {
                format!("{:016x}", link.content_hash)
            }
            ClipboardMetadata::File(file) => {
                format!("{:016x}", file.content_hash)
            }
            ClipboardMetadata::CodeSnippet(code) => {
                format!("{:016x}", code.content_hash)
            }
            ClipboardMetadata::RichText(rich_text) => {
                format!("{:016x}", rich_text.content_hash)
            }
        }
    }

    /// 获取存储路径
    pub fn get_storage_path(&self) -> &str {
        match self {
            ClipboardMetadata::Text(text) => &text.storage_path,
            ClipboardMetadata::Image(image) => &image.storage_path,
            ClipboardMetadata::Link(link) => &link.storage_path,
            ClipboardMetadata::File(file) => &file.storage_path,
            ClipboardMetadata::CodeSnippet(code) => &code.storage_path,
            ClipboardMetadata::RichText(rich_text) => &rich_text.storage_path,
        }
    }

    /// 判断两个元数据是否表示相同的内容
    pub fn is_duplicate(&self, other: &ClipboardMetadata) -> bool {
        match (self, other) {
            (ClipboardMetadata::Text(t1), ClipboardMetadata::Text(t2)) => {
                t1.content_hash == t2.content_hash
            }
            (ClipboardMetadata::Image(i1), ClipboardMetadata::Image(i2)) => {
                i1.content_hash == i2.content_hash &&
                i1.width == i2.width &&
                i1.height == i2.height &&
                // 文件大小相差不超过10%
                (i1.size as f64 - i2.size as f64).abs() / (i1.size as f64) <= 0.1
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
            "ClipboardMetadata[{}] - 类型: {}, 设备ID: {}, 时间: {}", 
            self.get_key(),
            self.get_content_type(),
            self.get_device_id(),
            self.get_timestamp().format("%Y-%m-%d %H:%M:%S"),
        )
    }
}

/// 剪贴板传输消息
/// 
/// 用于网络传输剪贴板内容的元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardTransferMessage {
    /// 元数据
    pub metadata: ClipboardMetadata,
    /// 消息ID
    pub message_id: String,
    /// 发送者设备ID
    pub sender_id: String,
    /// 发送者的记录ID，用于下载文件
    pub record_id: String,
}

impl ClipboardTransferMessage {
    /// 创建新的传输消息
    pub fn new(metadata: ClipboardMetadata, sender_id: String, record_id: String) -> Self {
        let message_id = format!("{}_{}", metadata.get_key(), Utc::now().timestamp_millis());
        Self {
            metadata,
            message_id,
            sender_id,
            record_id,
        }
    }

    /// 从元数据创建传输消息
    pub fn from_metadata(metadata: ClipboardMetadata, sender_id: String, record_id: String) -> Self {
        let message_id = format!("{}_{}", metadata.get_key(), Utc::now().timestamp_millis());
        Self {
            metadata,
            message_id,
            sender_id,
            record_id,
        }
    }

    pub fn to_payload(&self, bytes: bytes::Bytes) -> Payload {
        match &self.metadata {
            ClipboardMetadata::Text(text) => {
                Payload::new_text(
                    bytes,
                    self.sender_id.clone(),
                    self.metadata.get_timestamp(),
                )
            }
            ClipboardMetadata::Image(image) => {
                Payload::new_image(
                    bytes,
                    self.sender_id.clone(),
                    self.metadata.get_timestamp(),
                    image.width,
                    image.height,
                    image.format.clone(),
                    image.size,
                )
            }
            _ => {
                unimplemented!()
            }
            // ClipboardMetadata::Link(link) => {
            //     Payload::new_link(
            //         bytes,
            //         self.sender_id.clone(),
            //         self.metadata.get_timestamp(),
            //     )
            // }
            // ClipboardMetadata::File(file) => {
            //     Payload::new_file(
            //         bytes,
            //         self.sender_id.clone(),
            //         self.metadata.get_timestamp(),
            //     )
            // }
            // ClipboardMetadata::CodeSnippet(code) => {
            //     Payload::new_code_snippet(
            //         bytes,
            //         self.sender_id.clone(),
            //         self.metadata.get_timestamp(),
            //     )
            // }
            // ClipboardMetadata::RichText(rich_text) => {
            //     Payload::new_rich_text(
            //         bytes,
            //         self.sender_id.clone(),
            //         self.metadata.get_timestamp(),
            //     )
            // }
        }
    }
}

impl Display for ClipboardTransferMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content_type = self.metadata.get_content_type();
        
        write!(
            f, 
            "ClipboardTransfer[{}] - 类型: {}, 发送者: {}, 时间: {}", 
            self.message_id, 
            content_type, 
            self.sender_id, 
            self.metadata.get_timestamp().format("%Y-%m-%d %H:%M:%S")
        )
    }
}