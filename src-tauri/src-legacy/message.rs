use anyhow::Result;
use base64::Engine;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json;
use std::{fmt, fs};
use tokio_tungstenite::tungstenite::Message;
use twox_hash::xxh3::hash64;

use crate::application::file_service::ContentProcessorService;
use crate::domain::clipboard_metadata::ClipboardMetadata;
use crate::domain::device::{Device, DeviceStatus};
use crate::domain::network::{ConnectionRequestMessage, ConnectionResponseMessage};
use crate::domain::transfer_message::ClipboardTransferMessage;
use crate::infrastructure::storage::db::models::clipboard_record::DbClipboardRecord;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Payload {
    Text(TextPayload),
    Image(ImagePayload),
    File(FilePayload),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TextPayload {
    #[serde(
        serialize_with = "serialize_bytes",
        deserialize_with = "deserialize_bytes"
    )]
    content: Bytes,
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
}

impl TextPayload {
    pub fn get_content(&self) -> Bytes {
        self.content.clone()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ImagePayload {
    #[serde(
        serialize_with = "serialize_bytes",
        deserialize_with = "deserialize_bytes"
    )]
    content: Bytes,
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
    pub width: usize,
    pub height: usize,
    pub format: String,
    pub size: usize,
}

impl ImagePayload {
    // 计算图片内容的哈希值（采样哈希，只哈希前 64KB 以优化性能）
    pub fn content_hash(&self) -> u64 {
        const SAMPLE_SIZE: usize = 64 * 1024; // 64KB
        let sample = if self.content.len() <= SAMPLE_SIZE {
            self.content.as_ref()
        } else {
            &self.content[..SAMPLE_SIZE]
        };
        hash64(sample)
    }

    pub fn get_content(&self) -> Bytes {
        self.content.clone()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileInfo {
    pub file_name: String,
    pub file_size: u64,
    pub file_path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FilePayload {
    pub content_hash: u64,
    pub file_infos: Vec<FileInfo>,
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
}

impl FilePayload {
    pub fn new(file_infos: Vec<FileInfo>, device_id: String, timestamp: DateTime<Utc>) -> Self {
        let content_hash = hash64(
            &file_infos
                .iter()
                .map(|f| f.file_path.as_bytes())
                .collect::<Vec<_>>()
                .concat(),
        );
        Self {
            content_hash,
            file_infos,
            device_id,
            timestamp,
        }
    }

    /// 获取文件路径
    pub fn get_file_paths(&self) -> Vec<String> {
        self.file_infos
            .iter()
            .map(|f| f.file_path.clone())
            .collect()
    }
}

fn serialize_bytes<S>(bytes: &Bytes, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let base64_string = base64::engine::general_purpose::STANDARD.encode(bytes);
    serializer.serialize_str(&base64_string)
}

fn deserialize_bytes<'de, D>(deserializer: D) -> Result<Bytes, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let base64_string = String::deserialize(deserializer)?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&base64_string)
        .map_err(|e| serde::de::Error::custom(e.to_string()))?;
    Ok(Bytes::from(bytes))
}

impl Payload {
    pub fn new_text(content: Bytes, device_id: String, timestamp: DateTime<Utc>) -> Self {
        Payload::Text(TextPayload {
            content,
            device_id,
            timestamp,
        })
    }

    pub fn new_image(
        content: Bytes,
        device_id: String,
        timestamp: DateTime<Utc>,
        width: usize,
        height: usize,
        format: String,
        size: usize,
    ) -> Self {
        Payload::Image(ImagePayload {
            content,
            device_id,
            timestamp,
            width,
            height,
            format,
            size,
        })
    }

    pub fn new_file(
        file_infos: Vec<FileInfo>,
        device_id: String,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Payload::File(FilePayload::new(file_infos, device_id, timestamp))
    }

    pub fn new_file_from_path(
        file_paths: Vec<String>,
        device_id: String,
        timestamp: DateTime<Utc>,
    ) -> Result<Self> {
        let file_infos = file_paths
            .iter()
            .map(|path| {
                let file_name = path.split("/").last().unwrap_or("unknown");
                let file_size = fs::metadata(path)
                    .map_err(|e| anyhow::anyhow!("Failed to get file size: {}", e))?
                    .len();
                Ok(FileInfo {
                    file_name: file_name.to_string(),
                    file_size,
                    file_path: path.to_string(),
                })
            })
            .collect::<Result<Vec<FileInfo>>>()?;
        Ok(Payload::File(FilePayload::new(
            file_infos, device_id, timestamp,
        )))
    }

    #[allow(dead_code)]
    pub fn get_timestamp(&self) -> DateTime<Utc> {
        match self {
            Payload::Text(p) => p.timestamp,
            Payload::Image(p) => p.timestamp,
            Payload::File(p) => p.timestamp,
        }
    }

    #[allow(dead_code)]
    pub fn is_image(&self) -> bool {
        matches!(self, Payload::Image(_))
    }

    #[allow(dead_code)]
    pub fn as_image(&self) -> Option<&ImagePayload> {
        if let Payload::Image(image) = self {
            Some(image)
        } else {
            None
        }
    }

    pub fn get_device_id(&self) -> &str {
        match self {
            Payload::Text(p) => &p.device_id,
            Payload::Image(p) => &p.device_id,
            Payload::File(p) => &p.device_id,
        }
    }

    /// 获取 Payload 的唯一标识符
    pub fn get_key(&self) -> String {
        match self {
            Payload::Text(p) => {
                format!("{:016x}", hash64(p.content.as_ref()))
            }
            Payload::Image(p) => {
                // 使用图片内容哈希 + 尺寸信息作为唯一标识符
                let content_hash = p.content_hash();
                let size_info = format!("{}x{}", p.width, p.height);
                format!("img_{:016x}_{}", content_hash, size_info)
            }
            Payload::File(p) => {
                format!("file_{:016x}", p.content_hash)
            }
        }
    }

    /// 判断两个 Payload 是否相同
    ///
    /// 文本消息只比较内容是否相同
    /// 图片消息比较内容哈希、尺寸和文件大小
    pub fn is_duplicate(&self, other: &Payload) -> bool {
        match (self, other) {
            (Payload::Text(t1), Payload::Text(t2)) => t1.content == t2.content,
            (Payload::Image(i1), Payload::Image(i2)) => {
                // size=0 的特殊处理：避免除零错误
                let size_match = if i1.size == 0 && i2.size == 0 {
                    true
                } else if i1.size == 0 || i2.size == 0 {
                    false
                } else {
                    (i1.size as f64 - i2.size as f64).abs() / (i1.size as f64) <= 0.1
                };

                i1.content_hash() == i2.content_hash()
                    && i1.width == i2.width
                    && i1.height == i2.height
                    && size_match
            }
            _ => false,
        }
    }

    #[allow(dead_code)]
    pub fn eq(&self, other: &Payload) -> bool {
        self.get_key() == other.get_key()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn get_content_type(&self) -> &str {
        match self {
            Payload::Text(_) => "text",
            Payload::Image(_) => "image",
            Payload::File(_) => "file",
        }
    }

    /// 将 Payload 序列化为字节数组（用于 P2P 传输）
    pub fn to_bytes(&self) -> Result<Vec<u8>, anyhow::Error> {
        match self {
            Payload::Text(p) => Ok(p.content.as_ref().to_vec()),
            Payload::Image(p) => Ok(p.content.as_ref().to_vec()),
            Payload::File(_) => {
                anyhow::bail!("File type serialization not implemented yet")
            }
        }
    }
}

// 友好的展示大小
fn friendly_size(size: usize) -> String {
    if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{} KB", size / 1024)
    } else {
        format!("{} MB", size / 1024 / 1024)
    }
}

impl fmt::Display for Payload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Payload::Text(text) => write!(
                f,
                "文本消息 - KEY: {}, 设备: {}, 时间: {}, 内容长度: {}",
                self.get_key(),
                text.device_id,
                text.timestamp,
                friendly_size(text.content.len())
            ),
            Payload::Image(image) => write!(
                f,
                "图片消息 - KEY: {}, 设备: {}, 时间: {}, 尺寸: {}x{}, 格式: {}, 大小: {}",
                self.get_key(),
                image.device_id,
                image.timestamp,
                image.width,
                image.height,
                image.format,
                friendly_size(image.size)
            ),
            Payload::File(file) => write!(
                f,
                "文件消息 - KEY: {}, 设备: {}, 时间: {}, 文件数量: {}, 哈希: {:016x}",
                self.get_key(),
                file.device_id,
                file.timestamp,
                file.file_infos.len(),
                file.content_hash
            ),
        }
    }
}

impl PartialEq for Payload {
    fn eq(&self, other: &Self) -> bool {
        self.get_key() == other.get_key()
    }
}

impl Eq for Payload {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegisterDeviceMessage {
    pub id: String,
    pub ip: Option<String>,
    pub server_port: Option<u16>,
}

impl RegisterDeviceMessage {
    pub fn new(id: String, ip: Option<String>, server_port: Option<u16>) -> Self {
        Self {
            id,
            ip,
            server_port,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum WebSocketMessage {
    ClipboardSync(ClipboardTransferMessage),
    DeviceListSync(DevicesSyncMessage),
    Register(RegisterDeviceMessage),
    Unregister(String),
    ConnectionRequest(ConnectionRequestMessage),
    ConnectionResponse(ConnectionResponseMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSyncInfo {
    /// 设备ID
    pub id: String,
    /// 设备IP
    pub ip: Option<String>,
    /// 连接端口
    pub port: Option<u16>,
    /// 设备服务端口
    pub server_port: Option<u16>,
    /// 设备状态
    pub status: DeviceStatus,
    /// 设备更新时间(时间戳)
    pub updated_at: Option<i32>,
}

impl From<&Device> for DeviceSyncInfo {
    fn from(device: &Device) -> Self {
        Self {
            id: device.id.clone(),
            ip: device.ip.clone(),
            port: device.port,
            server_port: device.server_port,
            status: device.status,
            updated_at: device.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DevicesSyncMessage {
    pub devices: Vec<DeviceSyncInfo>,
    // 经转发过的设备ID列表
    pub replay_device_ids: Vec<String>,
}

impl DevicesSyncMessage {
    pub fn new(devices: Vec<DeviceSyncInfo>, replay_device_ids: Vec<String>) -> Self {
        Self {
            devices,
            replay_device_ids,
        }
    }
}

impl WebSocketMessage {
    pub fn to_tungstenite_message(&self) -> Message {
        Message::text(serde_json::to_string(self).unwrap())
    }

    pub fn to_json(&self) -> Result<String> {
        match serde_json::to_string(self) {
            Ok(json) => Ok(json),
            Err(e) => {
                anyhow::bail!("Failed to serialize WebSocketMessage: {}", e)
            }
        }
    }
}

/// 从 DbClipboardRecord 创建 Payload
impl TryFrom<DbClipboardRecord> for Payload {
    type Error = anyhow::Error;

    fn try_from(record: DbClipboardRecord) -> Result<Self, Self::Error> {
        ContentProcessorService::create_payload_from_record(&record)
    }
}

/// 从 ClipboardTransferMessage 和 Bytes 创建 Payload
impl From<(&ClipboardTransferMessage, Bytes)> for Payload {
    fn from((message, bytes): (&ClipboardTransferMessage, Bytes)) -> Self {
        match &message.metadata {
            ClipboardMetadata::Text(_) => Payload::new_text(
                bytes,
                message.sender_id.clone(),
                message.metadata.get_timestamp(),
            ),
            ClipboardMetadata::Image(image) => Payload::new_image(
                bytes,
                message.sender_id.clone(),
                message.metadata.get_timestamp(),
                image.width,
                image.height,
                image.format.clone(),
                image.size,
            ),
            _ => {
                unimplemented!()
            }
        }
    }
}

/// 从 ClipboardTransferMessage 直接创建 Payload（P2P 接收，消息已包含内容）
impl TryFrom<&ClipboardTransferMessage> for Payload {
    type Error = anyhow::Error;

    fn try_from(msg: &ClipboardTransferMessage) -> Result<Self, Self::Error> {
        let bytes = Bytes::from(msg.content.clone());
        match &msg.metadata {
            ClipboardMetadata::Text(_) => Ok(Payload::new_text(
                bytes,
                msg.sender_id.clone(),
                msg.metadata.get_timestamp(),
            )),
            ClipboardMetadata::Image(image) => Ok(Payload::new_image(
                bytes,
                msg.sender_id.clone(),
                msg.metadata.get_timestamp(),
                image.width,
                image.height,
                image.format.clone(),
                image.size,
            )),
            ClipboardMetadata::Link(_) => {
                // Link 类型作为文本处理
                Ok(Payload::new_text(
                    bytes,
                    msg.sender_id.clone(),
                    msg.metadata.get_timestamp(),
                ))
            }
            ClipboardMetadata::CodeSnippet(_) => {
                // CodeSnippet 类型作为文本处理
                Ok(Payload::new_text(
                    bytes,
                    msg.sender_id.clone(),
                    msg.metadata.get_timestamp(),
                ))
            }
            ClipboardMetadata::RichText(_) => {
                // RichText 类型暂时作为文本处理
                Ok(Payload::new_text(
                    bytes,
                    msg.sender_id.clone(),
                    msg.metadata.get_timestamp(),
                ))
            }
            ClipboardMetadata::File(_) => {
                anyhow::bail!("File type not supported in P2P mode yet")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 创建测试用的 ImagePayload
    fn create_test_image_payload(
        content: Vec<u8>,
        width: usize,
        height: usize,
        size: usize,
    ) -> Payload {
        Payload::Image(ImagePayload {
            content: Bytes::from(content),
            device_id: "test-device".to_string(),
            timestamp: Utc::now(),
            width,
            height,
            format: "png".to_string(),
            size,
        })
    }

    /// 创建测试用的 TextPayload
    fn create_test_text_payload(content: &str) -> Payload {
        Payload::Text(TextPayload {
            content: Bytes::from(content.to_string()),
            device_id: "test-device".to_string(),
            timestamp: Utc::now(),
        })
    }

    /// 创建测试用的 FilePayload
    fn create_test_file_payload(file_infos: Vec<FileInfo>) -> Payload {
        Payload::File(FilePayload {
            content_hash: 12345,
            file_infos,
            device_id: "test-device".to_string(),
            timestamp: Utc::now(),
        })
    }

    /// 生成大于 64KB 的测试数据
    fn create_large_test_bytes(size: usize) -> Vec<u8> {
        vec![0xAB; size]
    }

    // ========== 基础相同/不同情况测试 ==========

    #[test]
    fn test_image_identical() {
        let content = b"test-image-content".to_vec();
        let img1 = create_test_image_payload(content.clone(), 1920, 1080, 1024);
        let img2 = create_test_image_payload(content, 1920, 1080, 1024);

        assert!(img1.is_duplicate(&img2));
    }

    #[test]
    fn test_image_different_content() {
        let img1 = create_test_image_payload(b"content-a".to_vec(), 1920, 1080, 1024);
        let img2 = create_test_image_payload(b"content-b".to_vec(), 1920, 1080, 1024);

        assert!(!img1.is_duplicate(&img2));
    }

    // ========== Size 边界用例测试 ==========

    #[test]
    fn test_image_zero_size() {
        // 两个 size=0 的图片，其他条件相同
        let content = b"test-content".to_vec();
        let img1 = create_test_image_payload(content.clone(), 100, 100, 0);
        let img2 = create_test_image_payload(content, 100, 100, 0);

        // size 相同且为 0，应该判定为重复
        assert!(img1.is_duplicate(&img2));
    }

    #[test]
    fn test_image_same_content_different_size() {
        // size 差异 > 10% 的情况
        let content = b"test-content".to_vec();
        let img1 = create_test_image_payload(content.clone(), 1920, 1080, 1000);
        let img2 = create_test_image_payload(content, 1920, 1080, 1200); // 20% 差异

        assert!(!img1.is_duplicate(&img2));
    }

    #[test]
    fn test_image_same_content_size_within_tolerance() {
        // size 差异 ≤ 10% 的情况
        let content = b"test-content".to_vec();
        let img1 = create_test_image_payload(content.clone(), 1920, 1080, 1000);
        let img2 = create_test_image_payload(content, 1920, 1080, 1095); // 9.5% 差异

        assert!(img1.is_duplicate(&img2));
    }

    #[test]
    fn test_image_size_exactly_10_percent_threshold() {
        // 测试恰好 10% 边界
        let content = b"test-content".to_vec();
        let img1 = create_test_image_payload(content.clone(), 1920, 1080, 1000);
        let img2 = create_test_image_payload(content, 1920, 1080, 1100); // 恰好 10%

        assert!(img1.is_duplicate(&img2));
    }

    // ========== 采样哈希特性测试 ==========

    #[test]
    fn test_image_small_content_hash() {
        // 小于等于 64KB 的图片，完整内容哈希
        let small_content = vec![0xAA; 32 * 1024]; // 32KB
        let img1 = create_test_image_payload(small_content.clone(), 1920, 1080, 32768);
        let img2 = create_test_image_payload(small_content, 1920, 1080, 32768);

        assert!(img1.is_duplicate(&img2));
    }

    #[test]
    fn test_image_large_content_sample_hash() {
        // 大于 64KB 的图片，只哈希前 64KB
        let large_content_1 = create_large_test_bytes(100 * 1024); // 100KB
        let large_content_2 = create_large_test_bytes(100 * 1024); // 另一个 100KB，前 64KB 相同

        let img1 = create_test_image_payload(large_content_1, 1920, 1080, 102400);
        let img2 = create_test_image_payload(large_content_2, 1920, 1080, 102400);

        // 前 64KB 相同，应该判定为重复
        assert!(img1.is_duplicate(&img2));
    }

    #[test]
    fn test_image_same_prefix_different_suffix() {
        // 头部 64KB 相同但尾部不同 - 这是采样哈希的特性
        const KB: usize = 1024;
        let prefix = vec![0xAB; 64 * KB]; // 前 64KB
        let mut content1 = prefix.clone();
        content1.extend_from_slice(&vec![0xCD; 36 * KB]); // 后 36KB

        let mut content2 = prefix.clone();
        content2.extend_from_slice(&vec![0xEF; 36 * KB]); // 后 36KB 不同

        let img1 = create_test_image_payload(content1, 1920, 1080, 102400);
        let img2 = create_test_image_payload(content2, 1920, 1080, 102400);

        // 前 64KB 相同，应判定为重复（这是采样哈希的已知行为）
        assert!(img1.is_duplicate(&img2));
    }

    #[test]
    fn test_image_different_prefix() {
        // 头部不同，即使尺寸相同也不应判定为重复
        let content1 = vec![0xAA; 100 * 1024];
        let content2 = vec![0xBB; 100 * 1024];

        let img1 = create_test_image_payload(content1, 1920, 1080, 102400);
        let img2 = create_test_image_payload(content2, 1920, 1080, 102400);

        assert!(!img1.is_duplicate(&img2));
    }

    // ========== 尺寸相关测试 ==========

    #[test]
    fn test_image_same_content_different_dimensions() {
        // 相同内容但宽/高不同
        let content = b"test-content".to_vec();
        let img1 = create_test_image_payload(content.clone(), 1920, 1080, 1024);
        let img2 = create_test_image_payload(content, 1280, 720, 1024);

        assert!(!img1.is_duplicate(&img2));
    }

    #[test]
    fn test_image_different_width_same_height() {
        let content = b"test-content".to_vec();
        let img1 = create_test_image_payload(content.clone(), 1920, 1080, 1024);
        let img2 = create_test_image_payload(content, 1680, 1080, 1024);

        assert!(!img1.is_duplicate(&img2));
    }

    #[test]
    fn test_image_different_height_same_width() {
        let content = b"test-content".to_vec();
        let img1 = create_test_image_payload(content.clone(), 1920, 1080, 1024);
        let img2 = create_test_image_payload(content, 1920, 1200, 1024);

        assert!(!img1.is_duplicate(&img2));
    }

    #[test]
    fn test_image_all_dimensions_match() {
        // 内容、尺寸、大小都匹配应判定为重复
        let content = b"test-content".to_vec();
        let img1 = create_test_image_payload(content, 1920, 1080, 1024);
        let img2 = create_test_image_payload(b"test-content".to_vec(), 1920, 1080, 1024);

        assert!(img1.is_duplicate(&img2));
    }

    // ========== 边界情况测试 ==========

    #[test]
    fn test_image_empty_content() {
        // 空内容图片
        let img1 = create_test_image_payload(vec![], 100, 100, 0);
        let img2 = create_test_image_payload(vec![], 100, 100, 0);

        assert!(img1.is_duplicate(&img2));
    }

    #[test]
    fn test_image_empty_vs_non_empty() {
        // 空内容 vs 非空内容
        let img1 = create_test_image_payload(vec![], 100, 100, 0);
        let img2 = create_test_image_payload(b"content".to_vec(), 100, 100, 1024);

        assert!(!img1.is_duplicate(&img2));
    }

    #[test]
    fn test_image_mixed_type_comparison() {
        // Image vs Text/File 不应判定为重复
        let img = create_test_image_payload(b"test".to_vec(), 100, 100, 1024);
        let text = create_test_text_payload("test");
        let file = create_test_file_payload(vec![]);

        assert!(!img.is_duplicate(&text));
        assert!(!text.is_duplicate(&img));
        assert!(!img.is_duplicate(&file));
        assert!(!file.is_duplicate(&img));
    }

    // ========== 文本类型去重测试 ==========

    #[test]
    fn test_text_identical() {
        let text1 = create_test_text_payload("hello world");
        let text2 = create_test_text_payload("hello world");

        assert!(text1.is_duplicate(&text2));
    }

    #[test]
    fn test_text_different() {
        let text1 = create_test_text_payload("hello world");
        let text2 = create_test_text_payload("goodbye world");

        assert!(!text1.is_duplicate(&text2));
    }

    #[test]
    fn test_text_empty() {
        let text1 = create_test_text_payload("");
        let text2 = create_test_text_payload("");

        assert!(text1.is_duplicate(&text2));
    }

    #[test]
    fn test_text_vs_file() {
        let text = create_test_text_payload("test");
        let file = create_test_file_payload(vec![]);

        assert!(!text.is_duplicate(&file));
        assert!(!file.is_duplicate(&text));
    }
}
