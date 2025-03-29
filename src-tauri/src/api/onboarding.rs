use crate::application::device_service::get_device_manager;
use crate::domain::device::Platform;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

const ONBOARDING_FILE: &str = ".onboarding_completed";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
    pub full_name: String,
}

// 获取引导文件路径
fn get_onboarding_file_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("无法获取配置目录"))?
        .join("uniclipboard");

    // 确保配置目录存在
    fs::create_dir_all(&config_dir)?;

    Ok(config_dir.join(ONBOARDING_FILE))
}

// 检查引导是否已完成
#[tauri::command]
pub fn check_onboarding_status() -> Result<bool, String> {
    match get_onboarding_file_path() {
        Ok(path) => Ok(path.exists()),
        Err(e) => Err(format!("检查引导状态失败: {}", e)),
    }
}

// 标记引导完成
#[tauri::command]
pub fn complete_onboarding() -> Result<(), String> {
    match get_onboarding_file_path() {
        Ok(path) => {
            let mut file =
                File::create(path).map_err(|e| format!("创建引导完成标记文件失败: {}", e))?;
            file.write_all(b"onboarding completed")
                .map_err(|e| format!("写入引导完成标记文件失败: {}", e))?;
            Ok(())
        }
        Err(e) => Err(format!("获取引导文件路径失败: {}", e)),
    }
}

// 获取设备ID
#[tauri::command]
pub fn get_device_id() -> Result<String, String> {
    let device_manager = get_device_manager();
    let this_device = device_manager.get_current_device();
    match this_device {
        Ok(Some(device)) => Ok(device.id),
        Ok(None) => Err("无法获取当前设备".to_string()),
        Err(e) => Err(format!("获取当前设备失败: {}", e)),
    }
}

// 保存设备信息
#[tauri::command]
pub fn save_device_info(alias: Option<String>, platform: Option<Platform>) -> Result<(), String> {
    // 获取设备管理器
    let device_manager = get_device_manager();
    let device_id = get_device_id();
    match device_id {
        Ok(device_id) => {
            if let Some(alias) = alias {
                device_manager
                    .set_alias(&device_id, &alias)
                    .map_err(|e| e.to_string())?;
            }
            if let Some(platform) = platform {
                device_manager
                    .set_platform(&device_id, &platform)
                    .map_err(|e| e.to_string())?;
            }
            Ok(())
        }
        Err(e) => Err(format!("获取设备ID失败: {}", e)),
    }
}
