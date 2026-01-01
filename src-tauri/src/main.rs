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
use tauri_plugin_decorum::WebviewWindowExt;
use config::setting::{Setting, SETTING};
use infrastructure::runtime::AppRuntime;
use infrastructure::security::password::PasswordManager;
use infrastructure::storage::db::pool::DB_POOL;
use log::error;
use std::sync::Arc;
use tauri::{WebviewUrl, WebviewWindowBuilder};
use utils::logging;

fn main() {
    // 注意: 日志系统将在 Builder 插件注册时初始化

    // 加载用户设置
    let mut user_setting = match Setting::load(None) {
        Ok(config) => config,
        Err(e) => {
            error!("加载配置失败: {}", e);
            // 如果加载失败，使用默认配置
            let default_config = Setting::default();
            // 尝试保存默认配置
            if let Err(e) = default_config.save(None) {
                error!("保存默认配置失败: {}", e);
            }
            default_config
        }
    };

    // 如果设备名称为空，使用主机名
    if user_setting.general.device_name.is_empty() {
        let hostname = hostname::get()
            .map(|h: std::ffi::OsString| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "Unknown Device".to_string());
        user_setting.general.device_name = hostname;
        // 保存更新后的配置
        if let Err(e) = user_setting.save(None) {
            error!("Fail to save default device name: {}", e);
        }
    }

    // 确保配置已保存到全局 CONFIG 变量中
    {
        let mut global_setting = SETTING.write().unwrap();
        *global_setting = user_setting.clone();
    }

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

    // 注册当前设备
    let device_id = {
        let manager = get_device_manager();
        match manager.get_current_device() {
            Ok(Some(device)) => {
                SETTING.write().unwrap().set_device_id(device.id.clone());
                log::info!("Self device already exists: {}", device.id);
                device.id
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
                SETTING.write().unwrap().set_device_id(device_id.clone());

                device_id
            }
            Err(e) => {
                panic!("Failed to get self device: {}", e);
            }
        }
    };

    // 创建 AppRuntime
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let app_runtime = runtime.block_on(async {
        match AppRuntime::new(
            user_setting.clone(),
            device_id,
            user_setting.general.device_name.clone(),
        )
        .await
        {
            Ok(runtime) => runtime,
            Err(e) => {
                error!("Failed to create AppRuntime: {}", e);
                panic!("Failed to create AppRuntime: {}", e);
            }
        }
    });

    // 获取 runtime handle
    let app_handle = app_runtime.handle();

    // 运行应用
    run_app(app_runtime, app_handle, user_setting);
}

// 运行应用程序
fn run_app(
    app_runtime: AppRuntime,
    app_handle: infrastructure::runtime::AppRuntimeHandle,
    user_setting: Setting,
) {
    use tauri::Builder;
    use tauri_plugin_autostart::MacosLauncher;
    use tauri_plugin_single_instance;
    use tauri_plugin_stronghold;
    use tauri_plugin_decorum;

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
        .manage(app_handle)
        .manage(Arc::new(std::sync::Mutex::new(
            api::event::EventListenerState::default(),
        )))
        .setup(move |app| {
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

            let _window = win_builder.build().unwrap();

            // macOS specific window styling will be handled by the rounded corners plugin
            // The plugin will set up rounded corners, traffic lights positioning, and transparency

            // 启动 AppRuntime
            tauri::async_runtime::spawn(async move {
                // 启动 AppRuntime
                match app_runtime.start().await {
                    Ok(_) => {
                        log::info!("AppRuntime started successfully");
                    }
                    Err(e) => log::error!("Failed to start AppRuntime: {}", e),
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            api::setting::open_settings_window,
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
            api::p2p::get_local_peer_id,
            api::p2p::get_p2p_peers,
            api::p2p::initiate_p2p_pairing,
            api::p2p::verify_p2p_pairing_pin,
            api::p2p::reject_p2p_pairing,
            api::p2p::unpair_p2p_device,
            plugins::mac_rounded_corners::enable_rounded_corners,
            plugins::mac_rounded_corners::enable_modern_window_style,
            plugins::mac_rounded_corners::reposition_traffic_lights,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}