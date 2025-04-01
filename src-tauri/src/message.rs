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
use crate::core::transfer_message::ClipboardTransferMessage;
use crate::core::ClipboardMetadata;
use crate::domain::device::{Device, DeviceStatus};
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
    // 新增方法：计算图片内容的哈希值
    pub fn content_hash(&self) -> u64 {
        hash64(&self.content)
    }

    pub fn is_similar(&self, other: &ImagePayload) -> bool {
        // 尺寸一致且文件大小相差不超过 3%
        self.width == other.width
            && self.height == other.height
            && (self.size as f64 - other.size as f64).abs() / (self.size as f64) <= 0.1
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
    /// 图片消息只比较内容是否相似，不要求完全相同
    pub fn is_duplicate(&self, other: &Payload) -> bool {
        match (self, other) {
            (Payload::Text(t1), Payload::Text(t2)) => t1.content == t2.content,
            (Payload::Image(i1), Payload::Image(i2)) => i1.is_similar(i2),
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
