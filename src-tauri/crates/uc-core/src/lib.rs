//! # uc-core
//!
//! Core domain models and business logic for UniClipboard.
//!
//! This crate contains pure business logic without any infrastructure dependencies.

// Public module exports
pub mod clipboard;
pub mod device;
pub mod ids;
pub mod network;
pub mod ports;
pub mod security;
pub mod settings;

// Re-export commonly used types at the crate root
pub use clipboard::*;
pub use device::{Device, DeviceId, DeviceName, DeviceStatus, Platform};
pub use ids::BlobId;
pub use ids::{PeerId, SessionId};
pub use network::{NetworkEvent, NetworkStatus, ProtocolMessage};

// 不知道如何分类，临时定义在这里
pub struct MaterializeResult {
    pub blob_id: BlobId,
    pub locator: BlobStorageLocator,
    pub size_bytes: i64,
    pub content_hash: ContentHash,
    pub created_at_ms: i64,
}

impl MaterializeResult {
    pub fn new(
        blob_id: BlobId,
        locator: BlobStorageLocator,
        size_bytes: i64,
        content_hash: ContentHash,
        created_at_ms: i64,
    ) -> Self {
        Self {
            blob_id,
            locator,
            size_bytes,
            content_hash,
            created_at_ms,
        }
    }
}

pub struct EncryptionMeta {
    pub algo: String,   // "xchacha20poly1305"
    pub key_id: String, // keyslot id / key version
    pub nonce_b64: String,
    pub aad_b64: Option<String>,
}

#[derive(Debug, Clone)]
pub enum MaterializedPayload {
    /// 直接可交付
    Inline {
        mime: Option<String>,
        bytes: Vec<u8>,
    },

    /// 已经落 blob，可交付 blob 引用
    Blob {
        mime: Option<String>,
        blob_id: BlobId,
    },
}

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionAlgo {
    XChaCha20Poly1305,
}

/// 描述：
/// Blob 在「当前设备」上的存储定位方式
///
/// 重要约束：
/// - 不能跨设备使用
/// - 不能作为网络地址
/// - 不能推导 blob identity
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlobStorageLocator {
    LocalFs {
        /// 绝对路径
        path: PathBuf,
    },
    /// 本地文件系统 + 加密包裹
    ///
    /// 注意：
    /// - encryption 只描述“存储形态”
    /// - 不等价于传输加密
    EncryptedFs { path: PathBuf, algo: EncryptionAlgo },
}

impl BlobStorageLocator {
    pub fn new_local_fs(path: PathBuf) -> Self {
        BlobStorageLocator::LocalFs { path }
    }

    pub fn new_encrypted_fs(path: PathBuf, algo: EncryptionAlgo) -> Self {
        BlobStorageLocator::EncryptedFs { path, algo }
    }
}

#[derive(Debug, Clone)]
pub struct BlobRecord {
    pub blob_id: BlobId,
    pub locator: BlobStorageLocator,
    pub size_bytes: i64,
    pub content_hash: ContentHash,
    pub created_at_ms: i64,
}
