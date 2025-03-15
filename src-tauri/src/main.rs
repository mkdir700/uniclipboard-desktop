// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;
mod clipboard;
mod config;
mod connection;
mod context;
mod db;
mod device;
mod encrypt;
mod errors;
mod file_metadata;
mod setting;
mod key_mouse_monitor;
mod logger;
mod message;
mod migrations;
mod models;
mod network;
mod remote_sync;
mod schema;
mod uni_clipboard;
mod utils;
mod web;
mod commands;
use crate::config::Config;
use crate::uni_clipboard::UniClipboard;
use crate::uni_clipboard::UniClipboardBuilder;
use config::CONFIG;
use context::AppContextBuilder;
use device::{get_device_manager, Device};
use log::error;
use std::sync::Arc;
use utils::get_local_ip;

// 初始化UniClipboard
fn init_uniclipboard(config: Config) -> Arc<UniClipboard> {
    // 获取本地IP地址
    let local_ip = get_local_ip();

    // 设置设备管理器
    {
        let manager = get_device_manager();
        let device = Device::new(
            config.device_id.clone(),
            Some(local_ip.clone()),
            None,
            Some(config.webserver_port.unwrap()),
        );
        if let Err(e) = manager.add(device.clone()) {
            error!("添加设备失败: {}", e);
        }
        if let Err(e) = manager.set_self_device(&device) {
            error!("设置自身设备失败: {}", e);
        }
        if let Err(e) = manager.set_online(&config.device_id) {
            error!("设置设备在线状态失败: {}", e);
        }
    }

    // 创建一个阻塞的运行时来执行异步初始化
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // 在运行时中执行异步初始化
    let app = runtime.block_on(async {
        // 创建AppContext，传递配置
        let app_context = match AppContextBuilder::new(config.clone()).build().await {
            Ok(context) => context,
            Err(e) => {
                error!("创建AppContext失败: {}", e);
                panic!("创建AppContext失败: {}", e);
            }
        };

        // 构建UniClipboard实例
        let app = match UniClipboardBuilder::new()
            .set_webserver(app_context.webserver)
            .set_local_clipboard(app_context.local_clipboard)
            .set_remote_sync(app_context.remote_sync_manager)
            .set_connection_manager(app_context.connection_manager)
            .build()
        {
            Ok(app) => app,
            Err(e) => {
                error!("构建UniClipboard实例失败: {}", e);
                panic!("构建UniClipboard实例失败: {}", e);
            }
        };

        Arc::new(app)
    });

    app
}

fn main() {
    // 初始化日志系统
    logger::init();

    // 加载或创建配置
    let config = match Config::load(None) {
        Ok(config) => config,
        Err(e) => {
            error!("加载配置失败: {}", e);
            // 如果加载失败，使用默认配置
            let default_config = Config::default();
            // 尝试保存默认配置
            if let Err(e) = default_config.save(None) {
                error!("保存默认配置失败: {}", e);
                // 即使保存失败，我们仍然可以使用默认配置继续运行
            }
            default_config
        }
    };

    // 确保配置已保存到全局 CONFIG 变量中
    {
        let mut global_config = CONFIG.write().unwrap();
        *global_config = config.clone();
    }

    // 创建一个配置的克隆，用于初始化
    let config_for_init = config.clone();

    // 初始化 UniClipboard
    let uniclipboard_app = init_uniclipboard(config_for_init);

    // 运行应用
    run_app(uniclipboard_app);
}

// 运行应用程序
fn run_app(uniclipboard_app: Arc<UniClipboard>) {
    use tauri::{Builder, Manager};
    use std::sync::Mutex;
    use crate::commands::{greet, save_setting, get_setting};

    Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Arc::new(Mutex::new(Some(uniclipboard_app.clone()))))
        .setup(move |app| {
            // 获取应用句柄并克隆以便在异步任务中使用
            let app_handle = app.handle().clone();
            
            // 启动异步任务
            tauri::async_runtime::spawn(async move {
                // 启动UniClipboard
                match uniclipboard_app.start().await {
                    Ok(_) => {
                        log::info!("UniClipboard started successfully");
                        // 等待UniClipboard停止
                        if let Err(e) = uniclipboard_app.wait_for_stop().await {
                            log::error!("Error while waiting for UniClipboard to stop: {}", e);
                        }
                    }
                    Err(e) => log::error!("Failed to start UniClipboard: {}", e),
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet, save_setting, get_setting])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
