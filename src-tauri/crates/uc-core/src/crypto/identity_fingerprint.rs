//! Identity fingerprint generation and verification
//!
//! 这个模块提供了设备身份指纹的生成、验证和展示功能。
//!
//! # Security Model / 安全模型
//!
//! - **基于身份公钥**: 指纹基于设备的长期身份公钥,而非运行时 nonce 或网络地址
//! - **稳定且唯一**: 同一设备的指纹始终相同,不同设备的指纹几乎必然不同
//! - **可人工比对**: 指纹编码为易于人类阅读和比对的格式(Base32,分组显示)
//!
//! # Design / 设计
//!
//! ```text
//! Identity Keypair (Ed25519)
//!   ├── Private Key: 受保护存储(系统钥匙串/密钥管理器)
//!   └── Public Key: 用于生成指纹和签名验证
//!
//! Fingerprint Generation:
//!   public_key -> SHA-256(domain_sep || pub_key) -> Base32 -> 分组显示
//!                                          |
//!                                          v
//!                        "ABCD-EFGH-IJKL-MNOP" (16字符)
//!
//! Short Code (用户确认码):
//!   SHA-256(transcript) -> 前5字节 -> Base32 -> 6-8字符
//! ```

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// 身份指纹错误
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum FingerprintError {
    /// 无效的公钥长度
    #[error("Invalid public key length: expected {expected}, got {actual}")]
    InvalidKeyLength { expected: usize, actual: usize },

    /// 无效的指纹格式
    #[error("Invalid fingerprint format: {0}")]
    InvalidFormat(String),

    /// 指纹不匹配
    #[error("Fingerprint mismatch")]
    Mismatch,

    /// 编码错误
    #[error("Encoding error: {0}")]
    EncodingError(String),
}

/// 身份指纹 (16字符 Base32,分组显示)
///
/// Format: `ABCD-EFGH-IJKL-MNOP`
///
/// 每个分组4个字符,共4组,便于人类比对。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IdentityFingerprint(String);

impl IdentityFingerprint {
    /// 指纹的分组大小(字符数)
    const GROUP_SIZE: usize = 4;

    /// 指纹的总组数
    const GROUP_COUNT: usize = 4;

    /// 从原始字节数组创建指纹
    ///
    /// # Arguments
    ///
    /// * `bytes` - SHA-256 哈希输出(取前10字节用于指纹)
    ///
    /// # Process / 处理流程
    ///
    /// 1. 取输入的前10字节(80bit)
    /// 2. Base32 编码为16字符
    /// 3. 分组显示: `ABCD-EFGH-IJKL-MNOP`
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, FingerprintError> {
        if bytes.len() < 10 {
            return Err(FingerprintError::InvalidKeyLength {
                expected: 10,
                actual: bytes.len(),
            });
        }

        // 取前10字节并 Base32 编码
        let truncated = &bytes[0..10];
        let encoded = base32::encode(base32::Alphabet::Rfc4648 { padding: false }, truncated);

        // 分组显示
        let fingerprint = Self::format_with_groups(&encoded);

        Ok(Self(fingerprint))
    }

    /// 从公钥生成身份指纹
    ///
    /// # Algorithm / 算法
    ///
    /// ```text
    /// fingerprint_raw = SHA-256("uc-identity-fp-v1" || public_key_bytes)
    /// fingerprint_display = Base32(fingerprint_raw[0..10]) -> 分组
    /// ```
    ///
    /// # Arguments
    ///
    /// * `public_key` - 设备的身份公钥(Ed25519, 32字节)
    pub fn from_public_key(public_key: &[u8]) -> Result<Self, FingerprintError> {
        // Domain separator 防止不同用途的公钥混淆
        let mut hasher = Sha256::new();
        hasher.update(b"uc-identity-fp-v1");
        hasher.update(public_key);
        let hash = hasher.finalize();

        Self::from_bytes(&hash)
    }

    /// 从字符串解析指纹
    pub fn from_str(s: &str) -> Result<Self, FingerprintError> {
        let cleaned = s.replace('-', "");

        if cleaned.len() != 16 {
            return Err(FingerprintError::InvalidFormat(format!(
                "Expected 16 characters, got {}",
                cleaned.len()
            )));
        }

        // 验证 Base32 字符集
        if !cleaned.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(FingerprintError::InvalidFormat(
                "Non-alphanumeric characters found".to_string(),
            ));
        }

        Ok(IdentityFingerprint(Self::format_with_groups(&cleaned)))
    }

    /// 格式化为分组显示
    fn format_with_groups(encoded: &str) -> String {
        let groups: Vec<&str> = encoded
            .as_bytes()
            .chunks(Self::GROUP_SIZE)
            .map(|chunk| std::str::from_utf8(chunk).unwrap_or(""))
            .collect();

        groups.join("-")
    }

    /// 获取原始字符串(去除分组符号)
    pub fn as_raw(&self) -> String {
        self.0.replace('-', "")
    }

    /// 获取显示字符串(带分组符号)
    pub fn as_display(&self) -> &str {
        &self.0
    }

    /// 验证两个指纹是否匹配
    pub fn verify(&self, other: &IdentityFingerprint) -> Result<(), FingerprintError> {
        if self.as_raw() == other.as_raw() {
            Ok(())
        } else {
            Err(FingerprintError::Mismatch)
        }
    }
}

impl std::fmt::Display for IdentityFingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for IdentityFingerprint {
    type Err = FingerprintError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str(s)
    }
}

/// 短码生成器 (用于用户确认)
///
/// 短码基于配对会话的 transcript 哈希,而非固定指纹,
/// 防止攻击者提前伪造UI。
#[derive(Debug, Clone)]
pub struct ShortCodeGenerator {
    _private: (),
}

impl ShortCodeGenerator {
    /// 生成配对短码
    ///
    /// # Algorithm / 算法
    ///
    /// ```text
    /// transcript = "uc-pairing-transcript-v1" ||
    ///              session_id ||
    ///              nonce_initiator ||
    ///              nonce_responder ||
    ///              initiator_pubkey ||
    ///              responder_pubkey ||
    ///              protocol_version
    ///
    /// short_code = Base32(SHA-256(transcript)[0..5])
    /// ```
    ///
    /// # Arguments
    ///
    /// * `session_id` - 配对会话ID
    /// * `nonce_initiator` - 发起方 nonce
    /// * `nonce_responder` - 响应方 nonce
    /// * `initiator_pubkey` - 发起方身份公钥
    /// * `responder_pubkey` - 响应方身份公钥
    /// * `protocol_version` - 协议版本
    pub fn generate(
        session_id: &str,
        nonce_initiator: &[u8],
        nonce_responder: &[u8],
        initiator_pubkey: &[u8],
        responder_pubkey: &[u8],
        protocol_version: &str,
    ) -> Result<String, FingerprintError> {
        let mut hasher = Sha256::new();
        hasher.update(b"uc-pairing-transcript-v1");
        hasher.update(session_id.as_bytes());
        hasher.update(nonce_initiator);
        hasher.update(nonce_responder);
        hasher.update(initiator_pubkey);
        hasher.update(responder_pubkey);
        hasher.update(protocol_version.as_bytes());

        let hash = hasher.finalize();

        // 取前5字节(40bit) -> Base32 -> 8字符
        let truncated = &hash[0..5];
        let encoded = base32::encode(base32::Alphabet::Rfc4648 { padding: false }, truncated);

        // 返回前6-8字符(可根据需要调整)
        Ok(encoded.chars().take(8).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_from_public_key() {
        // 模拟 Ed25519 公钥(32字节)
        let pubkey = vec![0u8; 32];

        let fp = IdentityFingerprint::from_public_key(&pubkey).unwrap();

        // 应该是16字符,分组显示
        let display = fp.as_display();
        assert_eq!(display.chars().filter(|&c| c == '-').count(), 3);
        assert_eq!(display.replace('-', "").len(), 16);
    }

    #[test]
    fn test_fingerprint_formatting() {
        let fp = IdentityFingerprint::from_str("ABCDEFGH-IJKLMNOP").unwrap();
        assert_eq!(fp.as_display(), "ABCD-EFGH-IJKL-MNOP");
    }

    #[test]
    fn test_fingerprint_verification() {
        let fp1 = IdentityFingerprint::from_str("ABCD-EFGH-IJKL-MNOP").unwrap();
        let fp2 = IdentityFingerprint::from_str("ABCD-EFGH-IJKL-MNOP").unwrap();
        let fp3 = IdentityFingerprint::from_str("ZZZZ-ZZZZ-ZZZZ-ZZZZ").unwrap();

        assert!(fp1.verify(&fp2).is_ok());
        assert!(fp1.verify(&fp3).is_err());
    }

    #[test]
    fn test_short_code_generation() {
        let session_id = "test-session-123";
        let nonce_a = vec![1u8; 16];
        let nonce_b = vec![2u8; 16];
        let pub_a = vec![3u8; 32];
        let pub_b = vec![4u8; 32];
        let version = "1.0.0";

        let code =
            ShortCodeGenerator::generate(session_id, &nonce_a, &nonce_b, &pub_a, &pub_b, version)
                .unwrap();

        // 应该是6-8个字符
        assert!(code.len() >= 6 && code.len() <= 8);
        // 应该是纯字母数字
        assert!(code.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_fingerprint_invalid_length() {
        let short_key = vec![0u8; 5];
        let result = IdentityFingerprint::from_bytes(&short_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_fingerprint_invalid_format() {
        let result = IdentityFingerprint::from_str("TOO-SHORT");
        assert!(result.is_err());
    }
}
