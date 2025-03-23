// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod api;
mod application;
mod config;
mod core;
mod domain;
mod infrastructure;
mod interface;
mod message;
mod utils;

use application::device_service::get_device_manager;
use config::setting::{Setting, SETTING};
use infrastructure::storage::db::pool::DB_POOL;
use core::{context::AppContextBuilder, uniclipboard::UniClipboard, UniClipboardBuilder};
use log::error;
use std::sync::Arc;
use utils::logging;

// 初始化UniClipboard
fn init_uniclipboard(user_setting: Setting) -> Arc<UniClipboard> {
    // 注册当前设备
    {
        let manager = get_device_manager();
        match manager.get_self_device() {
            Ok(Some(device)) => {
                SETTING.write().unwrap().set_device_id(device.id.clone());
                log::info!("Self device already exists: {}", device.id);
            }
            Ok(None) => {
                // TODO: 获取本地IP地址
                // 目前只支持一个 ip ，后续可能同时支持多个 ip
                let local_ip = utils::helpers::get_local_ip();
                let result =
                    manager.register_self_device(local_ip, user_setting.network.webserver_port);
                if let Err(e) = result {
                    panic!("Failed to register self device: {}", e);
                }
                let device_id = result.unwrap();
                log::info!("Registered self device with ID: {}", device_id);
                SETTING.write().unwrap().set_device_id(device_id);
            }
            Err(e) => {
                panic!("Failed to get self device: {}", e);
            }
        }
    }

    // 创建一个阻塞的运行时来执行异步初始化
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // 在运行时中执行异步初始化
    let app = runtime.block_on(async {
        // 创建AppContext，传递配置
        let app_context = match AppContextBuilder::new(user_setting.clone()).build().await {
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
            .set_record_manager(app_context.record_manager)
            .set_file_storage(app_context.file_storage)
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
    logging::init();

    // 加载用户设置
    let user_setting = match Setting::load(None) {
        Ok(config) => config,
        Err(e) => {
            error!("加载配置失败: {}", e);
            // 如果加载失败，使用默认配置
            let default_config = Setting::default();
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
        let mut global_setting = SETTING.write().unwrap();
        *global_setting = user_setting.clone();
    }

    // 创建一个配置的克隆，用于初始化
    let user_setting_for_init = user_setting.clone();

    // 初始化数据库
    match DB_POOL.init() {
        Ok(_) => log::info!("Database initialized successfully"),
        Err(e) => {
            error!("Failed to initialize database: {}", e);
            panic!("Failed to initialize database: {}", e);
        }
    };

    // 初始化 UniClipboard
    let uniclipboard_app = init_uniclipboard(user_setting_for_init);

    // 运行应用
    run_app(uniclipboard_app);
}

// 运行应用程序
fn run_app(uniclipboard_app: Arc<UniClipboard>) {
    use std::sync::Mutex;
    use tauri::Builder;
    use tauri_plugin_autostart::MacosLauncher;

    Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .manage(Arc::new(Mutex::new(Some(uniclipboard_app.clone()))))
        .manage(Arc::new(Mutex::new(api::event::EventListenerState::default())))
        .setup(move |_app| {
            // 获取应用句柄并克隆以便在异步任务中使用
            let _ = _app.handle().clone();

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
        .invoke_handler(tauri::generate_handler![
            api::setting::save_setting,
            api::setting::get_setting,
            api::autostart::enable_autostart,
            api::autostart::disable_autostart,
            api::autostart::is_autostart_enabled,
            api::clipboard_items::get_clipboard_items,
            api::clipboard_items::delete_clipboard_item,
            api::clipboard_items::clear_clipboard_items,
            api::clipboard_items::get_clipboard_item,
            api::clipboard_items::copy_clipboard_item,
            api::event::listen_clipboard_new_content,
            api::event::stop_listen_clipboard_new_content,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
