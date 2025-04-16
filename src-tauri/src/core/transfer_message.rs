use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::core::clipboard_metadata::ClipboardMetadata;

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

/// 从元数据、发送者ID和记录ID创建传输消息
impl From<(ClipboardMetadata, String, String)> for ClipboardTransferMessage {
    fn from((metadata, sender_id, record_id): (ClipboardMetadata, String, String)) -> Self {
        let message_id = format!("{}_{}", metadata.get_key(), Utc::now().timestamp_millis());
        Self {
            metadata,
            message_id,
            sender_id,
            record_id,
        }
    }
}
