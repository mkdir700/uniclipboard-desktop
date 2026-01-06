use std::time::Duration;

use serde::{Deserialize, Serialize};

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub auto_start: bool,
    pub silent_start: bool,
    pub auto_check_update: bool,
    pub theme: Theme,
    pub theme_color: Option<String>,
    pub language: Option<String>,
    pub device_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContentTypes {
    pub text: bool,
    pub image: bool,
    pub link: bool,
    pub file: bool,
    pub code_snippet: bool,
    pub rich_text: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSettings {
    pub auto_sync: bool,
    pub sync_frequency: SyncFrequency,

    #[serde(default)]
    pub content_types: ContentTypes,

    pub max_file_size_mb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncFrequency {
    Realtime,
    Interval,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetentionRule {
    /// 按时间清理
    ByAge { max_age: Duration },

    /// 按总数量上限
    ByCount { max_items: usize },

    /// 按内容类型的最大存活时间
    ByContentType {
        content_type: ContentTypes,
        max_age: Duration,
    },

    /// 按磁盘占用大小
    ByTotalSize { max_bytes: u64 },

    /// 敏感内容快速过期
    Sensitive { max_age: Duration },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleEvaluation {
    AnyMatch, // OR（推荐，默认）
    AllMatch, // AND（极少用）
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RetentionPolicy {
    pub enabled: bool,
    pub rules: Vec<RetentionRule>,
    pub skip_pinned: bool,
    pub evaluation: RuleEvaluation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SecuritySettings {
    /// 是否启用本地数据加密
    pub encryption_enabled: bool,

    /// 是否已经在 keyring 中设置过口令
    ///
    /// 仅用于 UI 与流程判断
    /// 不代表当前口令是否“可用”
    pub passphrase_configured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "current_schema_version")]
    pub schema_version: u32,

    #[serde(default)]
    pub general: GeneralSettings,

    #[serde(default)]
    pub sync: SyncSettings,

    #[serde(default)]
    pub retention_policy: RetentionPolicy,

    #[serde(default)]
    pub security: SecuritySettings,
    // #[serde(default)]
    // pub network: NetworkSettings,
}

fn current_schema_version() -> u32 {
    CURRENT_SCHEMA_VERSION
}
