use anyhow::Result;
use std::env;
use std::path::PathBuf;

/// 获取配置目录
///
/// Returns:
///
/// - 如果获取到配置目录，则返回该目录
/// - 如果获取不到配置目录，则返回错误
pub fn get_config_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
        .join("com.uniclipboard.app");
    Ok(config_dir)
}

/// 获取设置文件路径
///
/// 优先从环境变量中获取，如果没有设置环境变量，则从系统配置目录中获取
///
/// Returns:
///
/// - 如果获取到设置文件路径，则返回该路径
/// - 如果获取不到设置文件路径，则返回错误
pub fn get_setting_path() -> Result<PathBuf> {
    if let Ok(path) = env::var("UNICLIPBOARD_SETTING_PATH") {
        return Ok(PathBuf::from(path));
    }

    let config_dir = get_config_dir()?;
    Ok(config_dir.join("setting.json"))
}