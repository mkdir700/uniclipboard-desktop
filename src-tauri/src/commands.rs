use crate::setting::Setting;

// test func
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// 从前端获取配置的Tauri命令
#[tauri::command]
pub fn save_setting(setting_json: &str) -> Result<(), String> {
    match serde_json::from_str::<Setting>(setting_json) {
        Ok(setting) => {
            if let Err(e) = setting.save(None) {
                return Err(format!("保存设置失败: {}", e));
            }
            Ok(())
        },
        Err(e) => Err(format!("解析TOML设置失败: {}", e)),
    }
}

// 获取当前配置的Tauri命令
#[tauri::command]
pub fn get_setting() -> Result<String, String> {
    match Setting::load(None) {
        Ok(setting) => {
            match serde_json::to_string_pretty(&setting) {
                Ok(json_str) => Ok(json_str),
                Err(e) => Err(format!("序列化设置为JSON失败: {}", e)),
            }
        },
        Err(e) => Err(format!("加载设置失败: {}", e)),
    }
}