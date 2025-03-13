use crate::utils::generate_device_id;
use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

pub static CONFIG: Lazy<RwLock<Config>> = Lazy::new(|| RwLock::new(Config::default()));

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub device_id: String,
    pub device_name: Option<String>,
    pub enable_webdav: Option<bool>,
    pub webdav_url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub max_history_size: Option<u64>,
    pub push_interval: Option<u64>, // ms
    pub pull_interval: Option<u64>, // ms
    pub sync_interval: Option<u64>, // ms
    pub enable_push: Option<bool>,
    pub enable_pull: Option<bool>,
    pub enable_key_mouse_monitor: Option<bool>,
    pub key_mouse_monitor_sleep_timeout: Option<u64>, // ms
    pub enable_websocket: Option<bool>,
    pub webserver_addr: Option<String>,
    pub webserver_port: Option<u16>,
    pub peer_device_addr: Option<String>,
    pub peer_device_port: Option<u16>,
}

impl Config {
    /// 获取当前配置的克隆
    /// 
    /// 这个方法会从全局 CONFIG 变量中获取当前配置的克隆，
    /// 避免在异步上下文中直接使用 CONFIG 变量，从而解决 Send trait 的问题。
    pub fn get_instance() -> Self {
        CONFIG.read().unwrap().clone()
    }

    pub fn default() -> Self {
        Self {
            device_id: generate_device_id(),
            device_name: Some("未命名设备".to_string()),
            webdav_url: None,
            username: None,
            password: None,
            enable_webdav: Some(false),
            max_history_size: Some(10),
            push_interval: Some(500),
            pull_interval: Some(500),
            sync_interval: Some(500),
            enable_push: Some(true),
            enable_pull: Some(true),
            enable_key_mouse_monitor: Some(true),
            key_mouse_monitor_sleep_timeout: Some(5000),
            enable_websocket: Some(true),
            webserver_addr: Some("0.0.0.0".to_string()),
            webserver_port: Some(8113),
            peer_device_addr: None,
            peer_device_port: None,
        }
    }

    pub fn merge_with_default(&mut self) {
        let default_config = Config::default();
        if self.enable_key_mouse_monitor.is_none() {
            self.enable_key_mouse_monitor = default_config.enable_key_mouse_monitor;
        }
        if self.key_mouse_monitor_sleep_timeout.is_none() {
            self.key_mouse_monitor_sleep_timeout = default_config.key_mouse_monitor_sleep_timeout;
        }
        if self.webserver_addr.is_none() {
            self.webserver_addr = default_config.webserver_addr;
        }
        if self.webserver_port.is_none() {
            self.webserver_port = default_config.webserver_port;
        }
        if self.device_name.is_none() {
            self.device_name = Some("未命名设备".to_string());
        }
        if self.enable_webdav.is_none() {
            self.enable_webdav = default_config.enable_webdav;
        }
        if self.enable_websocket.is_none() {
            self.enable_websocket = default_config.enable_websocket;
        }
        if self.max_history_size.is_none() {
            self.max_history_size = default_config.max_history_size;
        }
    }

    pub fn load(config_path: Option<PathBuf>) -> Result<Self> {
        let _config_path = if let Some(path) = config_path {
            path
        } else {
            get_config_path()?
        };

        if let Some(config_str) = fs::read_to_string(&_config_path).ok() {
            let mut config: Config =
                toml::from_str(&config_str).with_context(|| "Could not parse config file")?;
            config.merge_with_default();
            CONFIG.write().unwrap().clone_from(&config);
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub fn save(&self, config_path: Option<PathBuf>) -> Result<()> {
        if self.max_history_size.is_none() || self.max_history_size.unwrap() <= 1 {
            anyhow::bail!("max_history must be greater than 1");
        }
        if let Some(config_path) = config_path {
            let config_str = toml::to_string(self)?;
            fs::create_dir_all(config_path.parent().unwrap())?;
            fs::write(&config_path, config_str)
                .with_context(|| format!("Could not write config file: {:?}", config_path))?;
        } else {
            let config_path = get_config_path()?;
            let config_str = toml::to_string(self)?;
            fs::create_dir_all(config_path.parent().unwrap())?;
            fs::write(&config_path, config_str)
                .with_context(|| format!("Could not write config file: {:?}", config_path))?;
        }
        CONFIG.write().unwrap().clone_from(self);
        Ok(())
    }

    pub fn get_device_id(&self) -> String {
        self.device_id.clone()
    }
}

/// 获取配置文件路径
///
/// 优先从环境变量中获取，如果没有设置环境变量，则从系统配置目录中获取
///
/// Returns:
///
/// - 如果获取到配置文件路径，则返回该路径
/// - 如果获取不到配置文件路径，则返回错误
pub fn get_config_path() -> Result<PathBuf> {
    if let Ok(path) = env::var("UNICLIPBOARD_CONFIG_PATH") {
        return Ok(PathBuf::from(path));
    }

    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
        .join("uniclipboard");
    Ok(config_dir.join("config.toml"))
}

/// 获取配置目录
///
/// Returns:
///
/// - 如果获取到配置目录，则返回该目录
/// - 如果获取不到配置目录，则返回错误
pub fn get_config_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
        .join("uniclipboard");
    Ok(config_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_get_config_path_with_env() {
        env::set_var("UNICLIPBOARD_CONFIG_PATH", "/tmp/uniclipboard");
        let path = get_config_path().unwrap();
        assert_eq!(path, PathBuf::from("/tmp/uniclipboard"));
        env::remove_var("UNICLIPBOARD_CONFIG_PATH");
    }

    #[test]
    #[serial]
    fn test_get_config_path_without_env() {
        let path = get_config_path().unwrap();
        assert_eq!(
            path,
            PathBuf::from(
                dirs::config_dir()
                    .unwrap()
                    .join("uniclipboard")
                    .join("config.toml")
            )
        );
    }
}
