use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ClipboardStorageConfig {
    /// 单条 representation inline 的最大字节数
    pub inline_threshold_bytes: i64,
}

impl ClipboardStorageConfig {
    /// v1 默认值（**非常重要：永远保留**）
    pub fn defaults() -> Self {
        Self {
            inline_threshold_bytes: 16 * 1024, // 16 KB
        }
    }
}
