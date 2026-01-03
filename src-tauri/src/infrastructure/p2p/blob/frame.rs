//! BlobStream 帧格式定义
//!
//! 定义了分片流协议的帧类型、帧头和完整帧结构。

use serde::{Deserialize, Serialize};

/// 分片流协议版本
pub const PROTOCOL_VERSION: u8 = 1;

/// 分片大小：32KB
pub const CHUNK_SIZE: usize = 32768;

/// 帧头固定大小（24 字节，对齐到 8 字节边界）
pub const FRAME_HEADER_SIZE: usize = 24;

// 导出 ChunkSize 作为 CHUNK_SIZE 的别名（用于 sender.rs）
pub use CHUNK_SIZE as ChunkSize;

/// 帧类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum FrameType {
    /// 元数据帧：包含传输的元信息（文件大小、哈希等）
    Metadata = 0,
    /// 数据帧：包含实际的数据分片
    Data = 1,
    /// 完成帧：表示传输完成
    Complete = 2,
    /// 错误帧：表示传输过程中发生错误
    Error = 3,
}

impl FrameType {
    /// 从 u8 转换为 FrameType
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(FrameType::Metadata),
            1 => Some(FrameType::Data),
            2 => Some(FrameType::Complete),
            3 => Some(FrameType::Error),
            _ => None,
        }
    }
}

/// 分片帧头（24 字节，对齐到 8 字节边界）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameHeader {
    /// 协议版本
    pub version: u8,
    /// 帧类型
    pub frame_type: u8,
    /// 保留字段（2 字节）
    pub reserved: [u8; 2],
    /// 会话 ID（用于区分不同的传输会话）
    pub session_id: u32,
    /// 序列号（当前帧的序号）
    pub sequence: u32,
    /// 总帧数（元数据帧中设置）
    pub total_frames: u32,
    /// 数据长度（当前帧的数据部分长度）
    pub data_length: u32,
    /// BLAKE3 哈希前 16 字节（用于完整性校验）
    pub hash_prefix: [u8; 16],
}

impl FrameHeader {
    /// 创建新的帧头
    pub fn new(
        frame_type: FrameType,
        session_id: u32,
        sequence: u32,
        total_frames: u32,
        data_length: u32,
        hash_prefix: [u8; 16],
    ) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            frame_type: frame_type as u8,
            reserved: [0; 2],
            session_id,
            sequence,
            total_frames,
            data_length,
            hash_prefix,
        }
    }

    /// 创建元数据帧头
    pub fn metadata(session_id: u32, total_frames: u32, hash_prefix: [u8; 16]) -> Self {
        Self::new(
            FrameType::Metadata,
            session_id,
            0,
            total_frames,
            0,
            hash_prefix,
        )
    }

    /// 创建数据帧头
    pub fn data(
        session_id: u32,
        sequence: u32,
        total_frames: u32,
        data_length: u32,
        hash_prefix: [u8; 16],
    ) -> Self {
        Self::new(
            FrameType::Data,
            session_id,
            sequence,
            total_frames,
            data_length,
            hash_prefix,
        )
    }

    /// 创建完成帧头
    pub fn complete(session_id: u32) -> Self {
        Self::new(FrameType::Complete, session_id, 0, 0, 0, [0; 16])
    }

    /// 创建错误帧头
    pub fn error(session_id: u32) -> Self {
        Self::new(FrameType::Error, session_id, 0, 0, 0, [0; 16])
    }

    /// 获取帧类型
    pub fn get_frame_type(&self) -> Option<FrameType> {
        FrameType::from_u8(self.frame_type)
    }
}

/// 完整帧（头部 + 数据）
#[derive(Debug, Clone)]
pub struct Frame {
    /// 帧头
    pub header: FrameHeader,
    /// 帧数据
    pub data: Vec<u8>,
}

impl Frame {
    /// 创建新帧
    pub fn new(header: FrameHeader, data: Vec<u8>) -> Self {
        Self { header, data }
    }

    /// 序列化为字节（使用 bincode）
    pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // 序列化头部
        let header_bytes = bincode::serialize(&self.header)
            .map_err(|e| format!("Failed to serialize frame header: {}", e))?;

        // 计算总长度
        let total_length = header_bytes.len() + self.data.len();

        // 分配缓冲区
        let mut buffer = Vec::with_capacity(total_length);
        buffer.extend_from_slice(&header_bytes);
        buffer.extend_from_slice(&self.data);

        Ok(buffer)
    }

    /// 从字节反序列化
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        // 检查最小长度（至少要能放下头部）
        if bytes.len() < FRAME_HEADER_SIZE {
            return Err(format!(
                "Frame too short: expected at least {} bytes, got {}",
                FRAME_HEADER_SIZE,
                bytes.len()
            )
            .into());
        }

        // 反序列化头部
        let header: FrameHeader = bincode::deserialize(&bytes[..FRAME_HEADER_SIZE])
            .map_err(|e| format!("Failed to deserialize frame header: {}", e))?;

        // 检查版本
        if header.version != PROTOCOL_VERSION {
            return Err(format!(
                "Invalid protocol version: expected {}, got {}",
                PROTOCOL_VERSION, header.version
            )
            .into());
        }

        // 检查数据长度
        let expected_data_len = header.data_length as usize;
        let actual_data_len = bytes.len() - FRAME_HEADER_SIZE;

        if expected_data_len != actual_data_len {
            return Err(format!(
                "Data length mismatch: expected {} bytes, got {}",
                expected_data_len, actual_data_len
            )
            .into());
        }

        // 提取数据
        let data = bytes[FRAME_HEADER_SIZE..].to_vec();

        Ok(Self { header, data })
    }

    /// 获取帧类型
    pub fn frame_type(&self) -> Option<FrameType> {
        self.header.get_frame_type()
    }

    /// 获取会话 ID
    pub fn session_id(&self) -> u32 {
        self.header.session_id
    }

    /// 获取序列号
    pub fn sequence(&self) -> u32 {
        self.header.sequence
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_type_conversion() {
        assert_eq!(FrameType::from_u8(0), Some(FrameType::Metadata));
        assert_eq!(FrameType::from_u8(1), Some(FrameType::Data));
        assert_eq!(FrameType::from_u8(2), Some(FrameType::Complete));
        assert_eq!(FrameType::from_u8(3), Some(FrameType::Error));
        assert_eq!(FrameType::from_u8(99), None);
    }

    #[test]
    fn test_frame_serialization() {
        let header = FrameHeader::metadata(12345, 100, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
        let data = vec![1u8, 2, 3, 4, 5];
        let frame = Frame::new(header, data.clone());

        let bytes = frame.to_bytes().unwrap();
        let decoded = Frame::from_bytes(&bytes).unwrap();

        assert_eq!(decoded.header.version, PROTOCOL_VERSION);
        assert_eq!(decoded.header.frame_type, FrameType::Metadata as u8);
        assert_eq!(decoded.header.session_id, 12345);
        assert_eq!(decoded.header.total_frames, 100);
        assert_eq!(decoded.data, data);
    }

    #[test]
    fn test_data_frame() {
        let header = FrameHeader::data(100, 5, 10, 1024, [0; 16]);
        let data = vec![42u8; 1024];
        let frame = Frame::new(header, data);

        assert_eq!(frame.session_id(), 100);
        assert_eq!(frame.sequence(), 5);
    }

    #[test]
    fn test_complete_frame() {
        let header = FrameHeader::complete(999);
        let frame = Frame::new(header, vec![]);

        assert_eq!(frame.frame_type(), Some(FrameType::Complete));
        assert_eq!(frame.session_id(), 999);
    }
}
