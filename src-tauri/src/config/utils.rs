use anyhow::Result;
use std::env;
use std::path::PathBuf;

/// 判断是否为开发环境
fn is_development() -> bool {
    // 通过环境变量或编译时特性来判断
    // 1. 优先检查环境变量 UNICLIPBOARD_ENV
    if let Ok(env_val) = env::var("UNICLIPBOARD_ENV") {
        return env_val == "development";
    }
    // 2. 检查 debug_assertions 是否启用（编译时特性）
    #[cfg(debug_assertions)]
    {
        return true;
    }
    #[cfg(not(debug_assertions))]
    {
        return false;
    }
}

/// 获取环境标识符
fn get_env_suffix() -> &'static str {
    if is_development() {
        "-dev"
    } else {
        ""
    }
}

/// 获取配置目录
///
/// 开发环境和生产环境使用不同的配置目录，避免数据混淆
///
/// Returns:
///
/// - 如果获取到配置目录，则返回该目录
/// - 如果获取不到配置目录，则返回错误
pub fn get_config_dir() -> Result<PathBuf> {
    let base_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;

    // 开发环境和生产环境使用不同的配置目录，避免数据混淆
    let config_dir = if is_development() {
        base_dir.join("uniclipboard.mkdir700.dev-dev")
    } else {
        base_dir.join("uniclipboard.mkdir700.dev")
    };

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