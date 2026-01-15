//! In-memory encryption session port implementation
//! 内存加密会话端口实现

use tracing::{debug_span, debug};
use uc_core::ports::EncryptionSessionPort;
use uc_core::security::model::{EncryptionError, MasterKey};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

#[async_trait]
impl EncryptionSessionPort for InMemoryEncryptionSessionPort {
    async fn is_ready(&self) -> bool {
        let state = self.state.lock().expect("lock state");
        state.master_key.is_some()
    }

    async fn get_master_key(&self) -> Result<MasterKey, EncryptionError> {
        let state = self.state.lock().expect("lock state");
        state
            .master_key
            .as_ref()
            .cloned()
            .ok_or(EncryptionError::NotInitialized)
    }

    async fn set_master_key(&self, master_key: MasterKey) -> Result<(), EncryptionError> {
        let span = debug_span!("platform.encryption.set_master_key");
        let _enter = span.enter();

        let mut state = self.state.lock().expect("lock state");
        // Replace old key - MasterKey will be dropped and zeroized automatically
        // 替换旧密钥 - MasterKey 将被丢弃并自动零化
        state.master_key = Some(master_key);
        debug!("Master key set successfully");
        Ok(())
    }

    async fn clear(&self) -> Result<(), EncryptionError> {
        let span = debug_span!("platform.encryption.clear");
        let _enter = span.enter();

        let mut state = self.state.lock().expect("lock state");
        // Drop old key - MasterKey will be zeroized automatically
        // 丢弃旧密钥 - MasterKey 将自动零化
        state.master_key = None;
        debug!("Master key cleared");
        Ok(())
    }
}

/// In-memory encryption session port implementation
/// 内存加密会话端口实现
///
/// This implementation maintains an in-memory master key for basic functionality.
/// 此实现维护内存中的主密钥以实现基本功能。
///
/// # Current Limitations / 当前限制
///
/// Phase 2 (Development):
/// - Keys are stored in-memory only / 密钥仅存储在内存中
/// - Keys are lost on app restart / 应用重启后密钥丢失
/// - No persistence to secure storage / 未持久化到安全存储
///
/// Future Enhancement (Phase 3+):
/// - Persist master key to system keyring / 将主密钥持久化到系统密钥环
/// - Implement key rotation / 实现密钥轮换
/// - Add session timeout / 添加会话超时
///
/// # Security / 安全性
///
/// The current implementation provides:
/// 当前实现提供：
/// - Thread-safe access via Arc<Mutex<>> / 通过 Arc<Mutex<>> 实现线程安全访问
/// - Automatic key zeroization on drop / 丢弃时自动密钥零化（通过 MasterKey Drop impl）
/// - No disk writes / 无磁盘写入
///
#[derive(Clone)]
pub struct InMemoryEncryptionSessionPort {
    state: Arc<Mutex<EncryptionSessionState>>,
}

#[derive(Debug)]
struct EncryptionSessionState {
    master_key: Option<MasterKey>,
}

impl InMemoryEncryptionSessionPort {
    /// Create a new in-memory encryption session
    /// 创建新的内存加密会话
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(EncryptionSessionState {
                master_key: None,
            })),
        }
    }
}

impl Default for InMemoryEncryptionSessionPort {
    fn default() -> Self {
        Self::new()
    }
}

/// Backward compatibility alias
/// 向后兼容的类型别名
pub type PlaceholderEncryptionSessionPort = InMemoryEncryptionSessionPort;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_encryption_session_lifecycle() {
        let session = InMemoryEncryptionSessionPort::new();

        // Initially not ready
        assert!(!session.is_ready().await);

        // Set master key
        let key = MasterKey::generate().unwrap();
        session.set_master_key(key.clone()).await.unwrap();

        // Now ready
        assert!(session.is_ready().await);

        // Get master key
        let retrieved = session.get_master_key().await.unwrap();
        assert_eq!(retrieved.as_bytes(), key.as_bytes());

        // Clear
        session.clear().await.unwrap();

        // No longer ready
        assert!(!session.is_ready().await);

        // Get fails
        assert!(session.get_master_key().await.is_err());
    }

    #[tokio::test]
    async fn test_encryption_session_replace_key() {
        let session = InMemoryEncryptionSessionPort::new();

        let key1 = MasterKey::generate().unwrap();
        let key2 = MasterKey::generate().unwrap();

        session.set_master_key(key1).await.unwrap();
        session.set_master_key(key2.clone()).await.unwrap();

        let retrieved = session.get_master_key().await.unwrap();
        assert_eq!(retrieved.as_bytes(), key2.as_bytes());
    }

    #[tokio::test]
    async fn test_encryption_session_default() {
        let session = InMemoryEncryptionSessionPort::default();
        assert!(!session.is_ready().await);
    }

    #[tokio::test]
    async fn test_encryption_session_clone() {
        let session1 = InMemoryEncryptionSessionPort::new();
        let session2 = session1.clone();

        let key = MasterKey::generate().unwrap();
        session1.set_master_key(key.clone()).await.unwrap();

        // Both clones should share the same state
        assert!(session2.is_ready().await);
        let retrieved = session2.get_master_key().await.unwrap();
        assert_eq!(retrieved.as_bytes(), key.as_bytes());
    }
}
