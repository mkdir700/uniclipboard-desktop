use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
