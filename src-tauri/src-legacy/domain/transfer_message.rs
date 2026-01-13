use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::domain::clipboard_metadata::ClipboardMetadata;

/// 剪贴板传输消息
///
/// 用于 P2P 网络传输剪贴板内容（包含完整内容）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardTransferMessage {
    /// 元数据
    pub metadata: ClipboardMetadata,
    /// 消息ID
    pub message_id: String,
    /// 发送者设备ID
    pub sender_id: String,
    /// 实际内容（Text 或 Image 的字节数据）
    pub content: Vec<u8>,
}

impl ClipboardTransferMessage {
    /// 创建新的传输消息
    pub fn new(metadata: ClipboardMetadata, sender_id: String, content: Vec<u8>) -> Self {
        let message_id = format!("{}_{}", metadata.get_key(), Utc::now().timestamp_millis());
        Self {
            metadata,
            message_id,
            sender_id,
            content,
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

/// 从元数据、发送者ID和内容创建传输消息
impl From<(ClipboardMetadata, String, Vec<u8>)> for ClipboardTransferMessage {
    fn from((metadata, sender_id, content): (ClipboardMetadata, String, Vec<u8>)) -> Self {
        let message_id = format!("{}_{}", metadata.get_key(), Utc::now().timestamp_millis());
        Self {
            metadata,
            message_id,
            sender_id,
            content,
        }
    }
}
