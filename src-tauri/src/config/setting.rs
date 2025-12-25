use super::utils::get_setting_path;
use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

// 全局设置实例
pub static SETTING: Lazy<RwLock<Setting>> = Lazy::new(|| RwLock::new(Setting::default()));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowSyncContentTypes {
    pub text: bool,
    pub image: bool,
    pub link: bool,
    pub file: bool,
    pub code_snippet: bool,
    pub rich_text: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThemeMode {
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark,
    #[serde(rename = "system")]
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSetting {
    pub auto_start: bool,
    pub silent_start: bool,
    pub auto_check_update: bool,
    pub theme: ThemeMode,
    #[serde(default = "default_theme_color")]
    pub theme_color: String,
}

fn default_theme_color() -> String {
    "catppuccin".to_string()
}

// 同步设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSetting {
    // 是否启用自动同步
    pub auto_sync: bool,
    // 同步频率: "realtime", "30s", "1m", "5m", "15m"
    pub sync_frequency: String,
    // 同步内容类型
    pub content_types: AllowSyncContentTypes,
    // 最大同步文件大小 (MB)
    pub max_file_size: u32,
}

// 安全与隐私设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySetting {
    // 是否启用端到端加密
    pub end_to_end_encryption: bool,
}

// 网络设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSetting {
    // 同步方式: "lan_first", "cloud_only", "lan_only"
    pub sync_method: String,
    // 云服务器配置
    pub cloud_server: String,
    // Web服务器端口
    pub webserver_port: u16,
    // 是否手动填写对等设备地址
    pub custom_peer_device: bool,
    // 对等设备地址
    pub peer_device_addr: Option<String>,
    // 对等设备端口
    pub peer_device_port: Option<u16>,
    // 是否启用 WebDAV
    #[serde(default = "default_webdav_enabled")]
    pub webdav_enabled: bool,
    // WebDAV 服务地址
    #[serde(default = "default_webdav_url")]
    pub webdav_url: String,
    // WebDAV 用户名
    #[serde(default = "default_webdav_username")]
    pub webdav_username: String,
    // WebDAV 密码的存储 key（不直接存储密码）
    #[serde(default = "default_webdav_password_key")]
    pub webdav_password_key: Option<String>,
    // WebDAV 同步根目录
    #[serde(default = "default_webdav_base_path")]
    pub webdav_base_path: String,
    // WebDAV 轮询间隔（毫秒）
    #[serde(default = "default_webdav_polling_ms")]
    pub webdav_polling_ms: u64,
}

fn default_webdav_enabled() -> bool {
    false
}

fn default_webdav_url() -> String {
    String::new()
}

fn default_webdav_username() -> String {
    String::new()
}

fn default_webdav_password_key() -> Option<String> {
    None
}

fn default_webdav_base_path() -> String {
    "/uniclipboard".to_string()
}

fn default_webdav_polling_ms() -> u64 {
    30_000
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutoClearHistory {
    #[serde(rename = "never")]
    Never,
    #[serde(rename = "daily")]
    Daily,
    #[serde(rename = "weekly")]
    Weekly,
    #[serde(rename = "monthly")]
    Monthly,
    #[serde(rename = "on_exit")]
    OnExit,
}

// 存储管理
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSetting {
    pub auto_clear_history: AutoClearHistory,
    // 历史记录保留时间 (天)
    pub history_retention_days: u32,
    // 最大历史记录数: 100, 500, 1000, 5000, 0 (无限制)
    pub max_history_items: u32,
}

// 关于
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AboutSetting {
    // 应用版本
    pub version: String,
}

// 主设置结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    #[serde(skip)]
    device_id: String, // 设备ID，仅在程序启动时被设置，不参与序列化
    pub general: GeneralSetting,
    pub sync: SyncSetting,
    pub security: SecuritySetting,
    pub network: NetworkSetting,
    pub storage: StorageSetting,
    pub about: AboutSetting,
}

impl Setting {
    /// 获取当前设置的克隆
    pub fn get_instance() -> Self {
        SETTING.read().unwrap().clone()
    }

    /// 创建默认设置
    pub fn default() -> Self {
        Self {
            device_id: String::new(),
            general: GeneralSetting {
                auto_start: false,
                silent_start: false,
                auto_check_update: true,
                theme: ThemeMode::System,
                theme_color: "catppuccin".to_string(),
            },
            sync: SyncSetting {
                auto_sync: true,
                sync_frequency: "realtime".to_string(),
                content_types: AllowSyncContentTypes {
                    text: true,
                    image: true,
                    link: true,
                    file: true,
                    code_snippet: true,
                    rich_text: true,
                },
                max_file_size: 10, // MB
            },
            security: SecuritySetting {
                end_to_end_encryption: true,
            },
            network: NetworkSetting {
                sync_method: "lan_first".to_string(),
                cloud_server: "api.clipsync.com".to_string(),
                webserver_port: 29217,
                custom_peer_device: false,
                peer_device_addr: None,
                peer_device_port: None,
                webdav_enabled: false,
                webdav_url: String::new(),
                webdav_username: String::new(),
                webdav_password_key: None,
                webdav_base_path: "/uniclipboard".to_string(),
                webdav_polling_ms: 30_000,
            },
            storage: StorageSetting {
                auto_clear_history: AutoClearHistory::Never,
                history_retention_days: 30,
                max_history_items: 1000,
            },
            about: AboutSetting {
                version: "2.4.1".to_string(),
            },
        }
    }

    /// 加载设置
    ///
    /// 如果指定了设置文件路径，则从该路径加载设置
    /// 否则从默认配置目录加载设置
    pub fn load(setting_path: Option<PathBuf>) -> Result<Self> {
        let _setting_path = if let Some(path) = setting_path {
            path
        } else {
            get_setting_path()?
        };

        if let Some(setting_str) = fs::read_to_string(&_setting_path).ok() {
            let setting: Setting =
                serde_json::from_str(&setting_str).with_context(|| "无法解析设置文件")?;

            // 更新全局设置
            SETTING.write().unwrap().clone_from(&setting);

            Ok(setting)
        } else {
            // 如果设置文件不存在，则创建默认设置并保存
            let default_setting = Setting::default();
            default_setting.save(None)?;
            Ok(default_setting)
        }
    }

    /// 保存设置
    ///
    /// 如果指定了设置文件路径，则保存到该路径
    /// 否则保存到默认配置目录
    pub fn save(&self, setting_path: Option<PathBuf>) -> Result<()> {
        let _setting_path = if let Some(path) = setting_path {
            path
        } else {
            get_setting_path()?
        };

        // 确保目录存在
        if let Some(parent) = _setting_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // 将设置序列化为 JSON 格式
        let setting_str = serde_json::to_string_pretty(self)?;

        // 写入文件
        fs::write(&_setting_path, setting_str)
            .with_context(|| format!("无法写入设置文件: {:?}", _setting_path))?;
        // 更新全局设置
        SETTING.write().unwrap().clone_from(self);

        Ok(())
    }

    pub fn get_device_id(&self) -> String {
        self.device_id.clone()
    }

    pub fn set_device_id(&mut self, device_id: String) {
        self.device_id = device_id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_setting_default() {
        let setting = Setting::default();
        assert_eq!(setting.sync.auto_sync, true);
        assert_eq!(setting.sync.sync_frequency, "realtime");
        assert_eq!(setting.sync.content_types.text, true);
        assert_eq!(setting.security.end_to_end_encryption, true);
        assert_eq!(setting.network.sync_method, "lan_first");
        assert_eq!(setting.network.webdav_enabled, false);
        assert_eq!(setting.network.webdav_url, "");
        assert_eq!(setting.network.webdav_username, "");
        assert_eq!(setting.network.webdav_password_key, None);
        assert_eq!(setting.network.webdav_base_path, "/uniclipboard");
        assert_eq!(setting.network.webdav_polling_ms, 30_000);
        assert_eq!(setting.storage.history_retention_days, 30);
    }

    #[test]
    fn test_setting_save_load() -> Result<()> {
        // 创建临时目录
        let temp_dir = tempdir()?;
        let setting_path = temp_dir.path().join("test_setting.json");

        // 创建默认设置并保存
        let setting = Setting::default();
        setting.save(Some(setting_path.clone()))?;

        // 加载设置
        let loaded_setting = Setting::load(Some(setting_path))?;

        // 验证加载的设置与保存的设置一致
        assert_eq!(loaded_setting.sync.auto_sync, setting.sync.auto_sync);
        assert_eq!(
            loaded_setting.sync.sync_frequency,
            setting.sync.sync_frequency
        );
        assert_eq!(
            loaded_setting.network.cloud_server,
            setting.network.cloud_server
        );

        Ok(())
    }
}
