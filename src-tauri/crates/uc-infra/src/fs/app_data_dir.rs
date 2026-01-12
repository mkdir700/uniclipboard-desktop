use anyhow::{Context, Result};
use std::path::PathBuf;

/// Get the UniClipboard application data root directory.
///
/// 获取 UniClipboard 应用数据根目录。
///
/// # Platform-specific Paths / 平台特定路径
/// - macOS: ~/Library/Application Support/UniClipboard
/// - Windows: %APPDATA%\UniClipboard  
/// - Linux: $XDG_DATA_HOME/uniclipboard or ~/.local/share/uniclipboard
///
/// # Behavior / 行为
/// - This function does not automatically create directories.
/// - The caller decides when to create the directory.
///
/// - 此函数不自动创建目录。
/// - 由调用者决定何时创建。
pub fn app_data_dir() -> Result<PathBuf> {
    let base_dir =
        get_platform_data_dir().context("Failed to get platform-specific data directory")?;

    Ok(base_dir.join("UniClipboard"))
}

/// 获取设备ID存储目录
pub fn device_dir() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("device"))
}

/// 获取数据库存储目录
pub fn db_dir() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("db"))
}

/// 获取Blob存储目录
pub fn blob_dir() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("blob"))
}

/// 根据平台获取基础数据目录
fn get_platform_data_dir() -> Result<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        dirs::data_dir().ok_or_else(|| anyhow::anyhow!("Unable to get macOS data directory"))
    }

    #[cfg(target_os = "windows")]
    {
        dirs::data_dir().ok_or_else(|| anyhow::anyhow!("Unable to get Windows APPDATA directory"))
    }

    #[cfg(target_os = "linux")]
    {
        // 优先使用 XDG_DATA_HOME，如果不存在则使用 ~/.local/share
        if let Some(xdg_data_home) = std::env::var_os("XDG_DATA_HOME") {
            Ok(PathBuf::from(xdg_data_home))
        } else {
            dirs::data_dir().ok_or_else(|| anyhow::anyhow!("Unable to get Linux data directory"))
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        compile_error!("Unsupported platform for app_data_dir")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_data_dir_returns_path() {
        let path = app_data_dir().expect("Should be able to get app data dir");
        assert!(path.ends_with("UniClipboard"));
    }

    #[test]
    fn test_derived_dirs() {
        let device_path = device_dir().expect("Should be able to get device dir");
        assert!(device_path.ends_with("device"));
        assert!(device_path
            .components()
            .any(|c| c.as_os_str() == "UniClipboard"));

        let db_path = db_dir().expect("Should be able to get db dir");
        assert!(db_path.ends_with("db"));
        assert!(db_path
            .components()
            .any(|c| c.as_os_str() == "UniClipboard"));

        let blob_path = blob_dir().expect("Should be able to get blob dir");
        assert!(blob_path.ends_with("blob"));
        assert!(blob_path
            .components()
            .any(|c| c.as_os_str() == "UniClipboard"));
    }
}
