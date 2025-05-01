use crate::config::Setting;
use crate::infrastructure::security::password::{PASSWORD_SENDER, PasswordRequest};
use tokio::sync::mpsc;

/// 保存设置的Tauri命令
#[tauri::command]
pub fn save_setting(setting_json: &str) -> Result<(), String> {
    match serde_json::from_str::<Setting>(setting_json) {
        Ok(setting) => {
            if let Err(e) = setting.save(None) {
                return Err(format!("保存设置失败: {}", e));
            }
            Ok(())
        }
        Err(e) => Err(format!("解析TOML设置失败: {}", e)),
    }
}

/// 获取当前配置的Tauri命令
#[tauri::command]
pub fn get_setting() -> Result<String, String> {
    match Setting::load(None) {
        Ok(setting) => match serde_json::to_string_pretty(&setting) {
            Ok(json_str) => Ok(json_str),
            Err(e) => Err(format!("序列化设置为JSON失败: {}", e)),
        },
        Err(e) => Err(format!("加载设置失败: {}", e)),
    }
}

/// 获取加密口令
#[tauri::command]
pub async fn get_encryption_password() -> Result<String, String> {
    // 克隆通道发送端，避免持有锁
    let sender = match PASSWORD_SENDER.lock().unwrap().clone() {
        Some(sender) => sender,
        None => return Err("密码通道未初始化".to_string()),
    };
    
    // 创建一次性通道用于接收结果
    let (tx, mut rx) = mpsc::channel(1);
    
    // 发送获取密码的请求
    sender.send(PasswordRequest::GetEncryptionPassword(tx))
        .await
        .map_err(|e| format!("发送密码请求失败: {}", e))?;
    
    // 等待结果
    let result = rx.recv()
        .await
        .ok_or_else(|| "工作线程已关闭".to_string())?;
        
    // 处理结果
    match result {
        Ok(password) => Ok(password.unwrap_or_default()),
        Err(e) => Err(format!("获取加密口令失败: {}", e)),
    }
}

/// 设置加密口令
#[tauri::command]
pub async fn set_encryption_password(password: String) -> Result<(), String> {
    // 克隆通道发送端，避免持有锁
    let sender = match PASSWORD_SENDER.lock().unwrap().clone() {
        Some(sender) => sender,
        None => return Err("密码通道未初始化".to_string()),
    };
    
    // 创建一次性通道用于接收结果
    let (tx, mut rx) = mpsc::channel(1);
    
    // 发送设置密码的请求
    sender.send(PasswordRequest::SetEncryptionPassword(password, tx))
        .await
        .map_err(|e| format!("发送密码请求失败: {}", e))?;
    
    // 等待结果
    rx.recv()
        .await
        .ok_or_else(|| "工作线程已关闭".to_string())?
        .map_err(|e| format!("设置加密口令失败: {}", e))
}

/// 删除加密口令
#[tauri::command]
pub async fn delete_encryption_password() -> Result<(), String> {
    // 克隆通道发送端，避免持有锁
    let sender = match PASSWORD_SENDER.lock().unwrap().clone() {
        Some(sender) => sender,
        None => return Err("密码通道未初始化".to_string()),
    };
    
    // 创建一次性通道用于接收结果
    let (tx, mut rx) = mpsc::channel(1);
    
    // 发送删除密码的请求
    sender.send(PasswordRequest::DeleteEncryptionPassword(tx))
        .await
        .map_err(|e| format!("发送密码请求失败: {}", e))?;
    
    // 等待结果
    rx.recv()
        .await
        .ok_or_else(|| "工作线程已关闭".to_string())?
        .map_err(|e| format!("删除加密口令失败: {}", e))
}
