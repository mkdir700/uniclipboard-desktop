use tauri_plugin_autostart::ManagerExt as _;


// 启用开机自启动
#[tauri::command]
pub async fn enable_autostart(app_handle: tauri::AppHandle) -> Result<(), String> {
    let autostart_manager = app_handle.autolaunch();
    let _ = autostart_manager.enable();
    Ok(())
}

// 禁用开机自启动
#[tauri::command]
pub async fn disable_autostart(app_handle: tauri::AppHandle) -> Result<(), String> {
    let autostart_manager = app_handle.autolaunch();
    let _ = autostart_manager.disable();
    Ok(())
}

// 检查是否已启用开机自启动
#[tauri::command]
pub async fn is_autostart_enabled(app_handle: tauri::AppHandle) -> Result<bool, String> {
    let autostart_manager = app_handle.autolaunch();
    autostart_manager.is_enabled().map_err(|e| e.to_string())
}