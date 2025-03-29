use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::Path};

use crate::core::content_detector::ContentDetector;
use crate::core::content_type::ContentType;
use crate::core::metadata_models::{ImageMetadata, TextMetadata};
use crate::message::Payload;

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

impl ClipboardMetadata {
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

    /// 获取内容大小
    pub fn get_size(&self) -> usize {
        match self {
            ClipboardMetadata::Text(text) => text.size,
            ClipboardMetadata::Image(image) => image.size,
            ClipboardMetadata::Link(link) => link.size,
            ClipboardMetadata::File(file) => file.size,
            ClipboardMetadata::CodeSnippet(code) => code.size,
            ClipboardMetadata::RichText(rich_text) => rich_text.size,
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
        write!(
            f,
            "ClipboardMetadata[{}] - 类型: {}, 设备ID: {}, 时间: {}",
            self.get_key(),
            self.get_content_type(),
            self.get_device_id(),
            self.get_timestamp().format("%Y-%m-%d %H:%M:%S"),
        )
    }
}

/// 从 Payload 和本地存储路径创建 ClipboardMetadata
impl<P: AsRef<Path>> From<(&Payload, P)> for ClipboardMetadata {
    fn from((payload, storage_path): (&Payload, P)) -> Self {
        let path = storage_path.as_ref();
        match payload {
            Payload::Text(text) => ContentDetector::create_text_metadata(
                &text.content,
                text.device_id.clone(),
                text.timestamp,
                path.to_string_lossy().to_string(),
            ),
            Payload::Image(image) => ClipboardMetadata::Image(ImageMetadata {
                content_hash: image.content_hash(),
                device_id: image.device_id.clone(),
                timestamp: image.timestamp,
                width: image.width,
                height: image.height,
                format: image.format.clone(),
                size: image.size,
                storage_path: path.to_string_lossy().to_string(),
            }),
            Payload::File(file) => ClipboardMetadata::File(TextMetadata {
                content_hash: file.content_hash,
                device_id: file.device_id.clone(),
                timestamp: file.timestamp,
                length: file.file_size as usize,
                size: file.file_size as usize,
                storage_path: path.to_string_lossy().to_string(),
            }),
        }
    }
}
