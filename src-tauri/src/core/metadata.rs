use anyhow::Result;
use chrono::Utc;
use std::path::Path;

use crate::message::Payload;
use crate::core::transfer::ClipboardMetadata;

/// 元数据生成器
/// 
/// 负责为剪贴板内容生成元数据，如时间戳、设备ID等
pub struct MetadataGenerator {
    device_id: String,
}

impl MetadataGenerator {
    pub fn new(device_id: String) -> Self {
        Self { device_id }
    }
    
    /// 从 Payload 生成用于网络传输的元数据
    pub fn generate_metadata(&self, payload: &Payload, storage_path: &Path) -> ClipboardMetadata {
        ClipboardMetadata::from_payload(payload, storage_path)
    }
}
