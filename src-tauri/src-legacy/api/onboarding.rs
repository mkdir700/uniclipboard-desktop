use crate::api::event::OnboardingCompletedEvent;
use crate::api::event::OnboardingPasswordSetEvent;
use crate::api::setting::get_encryption_password;
use crate::api::setting::set_encryption_password;
use crate::application::device_service::get_device_manager;
use crate::config::get_config_dir;
use crate::domain::device::Platform;
use crate::infrastructure::security::password::PasswordManager;
use anyhow::Result;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::SystemTime;
use tauri::{AppHandle, Emitter};

const ONBOARDING_FILE: &str = ".onboarding_completed";

/// Onboarding 状态信息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OnboardingStatus {
    pub has_completed: bool,
    pub vault_initialized: bool,
    pub device_registered: bool,
    pub encryption_password_set: bool,
}

/// 检查是否已完成 onboarding
///
/// 健壮性检查：
/// 1. 检查 onboarding 标记文件是否存在
/// 2. 检查 vault 密钥是否已初始化
/// 3. 检查当前设备是否已注册
/// 4. 检查加密密码是否已设置
#[tauri::command]
pub async fn check_onboarding_status() -> Result<OnboardingStatus, String> {
    let config_dir = get_config_dir().map_err(|e| format!("获取配置目录失败: {}", e))?;

    let onboarding_file = config_dir.join(ONBOARDING_FILE);
    let has_completed = onboarding_file.exists();

    // 检查 vault 密钥是否已初始化
    let vault_key_file = config_dir.join(".vault_key");
    let vault_initialized = vault_key_file.exists();

    // 检查设备是否已注册
    let device_manager = get_device_manager();
    let device_registered = match device_manager.get_current_device() {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            error!("检查设备注册状态失败: {}", e);
            false
        }
    };

    // 检查加密密码是否已设置（使用轻量级检查，避免触发单例初始化）
    let encryption_password_set = PasswordManager::has_encryption_password();

    // 如果 onboarding 标记存在但关键组件缺失，记录警告
    if has_completed && (!vault_initialized || !device_registered || !encryption_password_set) {
        warn!(
            "Onboarding 标记存在但状态不一致: completed={}, vault={}, device={}, password={}",
            has_completed, vault_initialized, device_registered, encryption_password_set
        );
    }

    Ok(OnboardingStatus {
        has_completed,
        vault_initialized,
        device_registered,
        encryption_password_set,
    })
}

/// 完成 onboarding 流程
///
/// 此操作会：
/// 1. 验证 vault 密钥是否已初始化（通过 PasswordManager 单例）
/// 2. 验证加密密码是否已设置
/// 3. 确保当前设备已注册
/// 4. 创建 onboarding 完成标记文件
/// 5. 发出完成事件通知前端
///
/// 注意：调用此命令前，用户必须已经设置了加密密码
#[tauri::command]
pub async fn complete_onboarding(app_handle: AppHandle) -> Result<(), String> {
    let config_dir = get_config_dir().map_err(|e| format!("获取配置目录失败: {}", e))?;

    // 确保配置目录存在
    std::fs::create_dir_all(&config_dir).map_err(|e| format!("创建配置目录失败: {}", e))?;

    // 验证 vault 密钥是否已初始化
    // PasswordManager 单例在首次访问时会自动初始化 vault 密码
    let vault_key_file = config_dir.join(".vault_key");
    if !vault_key_file.exists() {
        warn!("Vault 密钥文件尚未初始化，这通常不应该发生");
        return Err("Vault 密钥初始化失败".to_string());
    }

    // 验证加密密码是否已设置
    let has_password = match get_encryption_password().await {
        Ok(_) => true,
        Err(_) => false,
    };

    if !has_password {
        warn!("加密密码尚未设置");
        return Err("请先设置加密密码".to_string());
    }

    // 验证设备是否已注册
    let device_manager = get_device_manager();
    let device_exists = match device_manager.get_current_device() {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            error!("检查设备注册状态失败: {}", e);
            return Err(format!("检查设备注册状态失败: {}", e));
        }
    };

    if !device_exists {
        warn!("设备尚未注册");
        return Err("设备尚未注册".to_string());
    }

    // 创建 onboarding 完成标记文件
    let onboarding_file = config_dir.join(ONBOARDING_FILE);

    // 使用原子写入避免文件损坏
    let temp_file = onboarding_file.with_extension("tmp");
    fs::write(
        &temp_file,
        format!(
            "onboarding completed\n{}",
            chrono::Local::now().to_rfc3339()
        ),
    )
    .map_err(|e| format!("写入临时 onboarding 文件失败: {}", e))?;

    // 原子性重命名
    fs::rename(&temp_file, &onboarding_file)
        .map_err(|e| format!("创建 onboarding 标记文件失败: {}", e))?;

    // 设置文件权限（仅用户可读写）
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&onboarding_file)
            .map_err(|e| format!("获取文件权限失败: {}", e))?
            .permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&onboarding_file, perms)
            .map_err(|e| format!("设置文件权限失败: {}", e))?;
    }

    // 验证文件已创建
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    if !onboarding_file.exists() {
        return Err("Onboarding 标记文件创建验证失败".to_string());
    }

    // 发出完成事件
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| format!("获取时间戳失败: {}", e))?
        .as_millis() as u64;

    let event = OnboardingCompletedEvent { timestamp };
    app_handle
        .emit("onboarding-completed", event)
        .map_err(|e| format!("发送事件失败: {}", e))?;

    info!("Onboarding 流程完成");
    Ok(())
}

/// 设置加密密码（onboarding 专用）
///
/// 此命令用于 onboarding 流程中设置加密密码
/// 与 set_encryption_password 的区别在于：
/// - 验证密码强度
/// - 如果密码已存在则返回错误
/// - 设置完成后发出事件通知前端
#[tauri::command]
pub async fn setup_encryption_password(
    password: String,
    app_handle: AppHandle,
) -> Result<(), String> {
    // 验证密码强度
    if password.len() < 8 {
        return Err("密码长度至少为 8 位".to_string());
    }

    // 检查密码是否已设置
    match get_encryption_password().await {
        Ok(_) => {
            return Err("加密密码已设置".to_string());
        }
        Err(_) => {}
    }

    // 设置密码
    set_encryption_password(password).await?;

    // 等待 Stronghold 保存完成并验证
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let verify_result = get_encryption_password().await;
    if verify_result.is_err() {
        return Err("密码保存验证失败，请重试".to_string());
    }

    // 发出密码设置成功事件
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| format!("获取时间戳失败: {}", e))?
        .as_millis() as u64;

    let event = OnboardingPasswordSetEvent { timestamp };
    app_handle
        .emit("onboarding-password-set", event)
        .map_err(|e| format!("发送事件失败: {}", e))?;

    info!("Onboarding: 加密密码设置成功");
    Ok(())
}

/// 获取设备 ID
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

/// 保存设备信息
#[tauri::command]
pub fn save_device_info(alias: Option<String>, platform: Option<Platform>) -> Result<(), String> {
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
