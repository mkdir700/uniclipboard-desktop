// TODO: 暂时不知道如何分类，先写这里。

use uc_core::{BlobId, ContentHash};

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

#[derive(Debug, Clone)]
pub struct BlobRecord {
    pub blob_id: BlobId,
    pub locator: BlobStorageLocator,
    pub size_bytes: i64,
    pub content_hash: ContentHash,
    pub created_at_ms: i64,
}
