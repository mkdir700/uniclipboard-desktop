use crate::config::Config;

// test func
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// 从前端获取配置的Tauri命令
#[tauri::command]
pub fn save_config(config_json: &str) -> Result<(), String> {
    match serde_json::from_str::<Config>(config_json) {
        Ok(config) => {
            if let Err(e) = config.save(None) {
                return Err(format!("保存配置失败: {}", e));
            }
            Ok(())
        },
        Err(e) => Err(format!("解析配置失败: {}", e)),
    }
}

// 获取当前配置的Tauri命令
#[tauri::command]
pub fn get_config() -> Result<String, String> {
    match Config::load(None) {
        Ok(config) => {
            match serde_json::to_string(&config) {
                Ok(json) => Ok(json),
                Err(e) => Err(format!("序列化配置失败: {}", e)),
            }
        },
        Err(e) => Err(format!("加载配置失败: {}", e)),
    }
}