use std::{fmt::Display, path::Path};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use twox_hash::xxh3::hash64;

use crate::message::Payload;

/// 剪贴板内容元数据
/// 
/// 用于网络传输，不包含实际内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClipboardMetadata {
    Text(TextMetadata),
    Image(ImageMetadata),
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
        }
    }

    /// 获取设备ID
    pub fn get_device_id(&self) -> &str {
        match self {
            ClipboardMetadata::Text(text) => &text.device_id,
            ClipboardMetadata::Image(image) => &image.device_id,
        }
    }

    /// 获取时间戳
    pub fn get_timestamp(&self) -> DateTime<Utc> {
        match self {
            ClipboardMetadata::Text(text) => text.timestamp,
            ClipboardMetadata::Image(image) => image.timestamp,
        }
    }

    /// 获取内容类型
    pub fn get_content_type(&self) -> &str {
        match self {
            ClipboardMetadata::Text(_) => "text",
            ClipboardMetadata::Image(_) => "image",
        }
    }

    /// 获取内容哈希值
    pub fn get_content_hash(&self) -> u64 {
        match self {
            ClipboardMetadata::Text(text) => text.content_hash,
            ClipboardMetadata::Image(image) => image.content_hash,
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
        }
    }

    /// 获取存储路径
    pub fn get_storage_path(&self) -> &str {
        match self {
            ClipboardMetadata::Text(text) => &text.storage_path,
            ClipboardMetadata::Image(image) => &image.storage_path,
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
        }
    }
}

impl Display for ClipboardTransferMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content_type = self.metadata.get_content_type();
        let timestamp = self.metadata.get_timestamp().format("%Y-%m-%d %H:%M:%S");
        
        write!(
            f, 
            "ClipboardTransfer[{}] - 类型: {}, 发送者: {}, 时间: {}", 
            self.message_id, 
            content_type, 
            self.sender_id, 
            timestamp
        )
    }
}