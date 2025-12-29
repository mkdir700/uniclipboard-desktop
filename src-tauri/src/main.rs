// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod api;
mod application;
mod config;
mod domain;
mod infrastructure;
mod interface;
mod message;
mod plugins;
mod utils;

use application::device_service::get_device_manager;
use config::setting::{Setting, SETTING};
use infrastructure::context::AppContextBuilder;
use infrastructure::security::password::PasswordManager;
use infrastructure::storage::db::pool::DB_POOL;
use infrastructure::uniclipboard::{UniClipboard, UniClipboardBuilder};
use log::error;
use std::sync::Arc;
use tauri::{WebviewUrl, WebviewWindowBuilder};
#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;
use utils::logging;

// 初始化UniClipboard
fn init_uniclipboard(user_setting: Setting) -> Arc<UniClipboard> {
    // 注册当前设备
    {
        let manager = get_device_manager();
        match manager.get_current_device() {
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
    // 注意: 日志系统将在 Builder 插件注册时初始化

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

    // 初始化密码管理器
    PasswordManager::init_salt_file_if_not_exists().unwrap();

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
    run_app(uniclipboard_app, user_setting);
}

// 运行应用程序
fn run_app(uniclipboard_app: Arc<UniClipboard>, user_setting: Setting) {
    use std::sync::Mutex;
    use tauri::Builder;
    use tauri_plugin_autostart::MacosLauncher;
    use tauri_plugin_single_instance;
    use tauri_plugin_stronghold;

    Builder::default()
        .plugin(logging::get_builder().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(
            tauri_plugin_stronghold::Builder::with_argon2(&PasswordManager::get_salt_file_path())
                .build(),
        )
        .manage(Arc::new(Mutex::new(Some(uniclipboard_app.clone()))))
        .manage(Arc::new(Mutex::new(
            api::event::EventListenerState::default(),
        )))
        .setup(move |app| {
            // 获取应用句柄并克隆以便在异步任务中使用
            let app_handle = app.handle().clone();

            let win_builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
                .title("")
                .inner_size(800.0, 600.0)
                .min_inner_size(800.0, 600.0);

            // Use platform-specific title bar settings
            #[cfg(target_os = "macos")]
            let win_builder = win_builder.decorations(false);

            #[cfg(target_os = "windows")]
            let win_builder = win_builder.decorations(false).shadow(true);

            // 如果启用了静默启动，则初始不可见
            let win_builder = if user_setting.general.silent_start {
                win_builder.visible(false)
            } else {
                win_builder
            };

            let window = win_builder.build().unwrap();

            // macOS specific window styling will be handled by the rounded corners plugin
            // The plugin will set up rounded corners, traffic lights positioning, and transparency

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
            api::setting::get_encryption_password,
            api::setting::set_encryption_password,
            api::setting::delete_encryption_password,
            api::autostart::enable_autostart,
            api::autostart::disable_autostart,
            api::autostart::is_autostart_enabled,
            api::clipboard_items::get_clipboard_items,
            api::clipboard_items::delete_clipboard_item,
            api::clipboard_items::clear_clipboard_items,
            api::clipboard_items::get_clipboard_item,
            api::clipboard_items::copy_clipboard_item,
            api::clipboard_items::toggle_favorite_clipboard_item,
            api::clipboard_items::get_clipboard_stats,
            api::event::listen_clipboard_new_content,
            api::event::stop_listen_clipboard_new_content,
            api::event::listen_connection_request,
            api::event::stop_listen_connection_request,
            api::event::listen_connection_response,
            api::event::stop_listen_connection_response,
            api::onboarding::check_onboarding_status,
            api::onboarding::complete_onboarding,
            api::onboarding::get_device_id,
            api::onboarding::save_device_info,
            api::device_connection::get_local_network_interfaces,
            api::device_connection::connect_to_device_manual,
            api::device_connection::respond_to_connection_request,
            api::device_connection::cancel_connection_request,
            plugins::mac_rounded_corners::enable_rounded_corners,
            plugins::mac_rounded_corners::enable_modern_window_style,
            plugins::mac_rounded_corners::reposition_traffic_lights,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
