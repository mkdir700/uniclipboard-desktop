// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod api;
mod application;
mod config;
mod domain;
mod error;
mod infrastructure;
mod interface;
mod message;
mod models;
mod plugins;
mod utils;

use application::device_service::get_device_manager;
use config::setting::{Setting, SETTING};
use infrastructure::runtime::{AppRuntime, AppRuntimeHandle};
use infrastructure::security::password::PasswordManager;
use infrastructure::storage::db::pool::DB_POOL;
use log::error;
use std::sync::Arc;
use tauri::{WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_decorum::WebviewWindowExt;
use tokio::sync::mpsc;
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
        let hostname = gethostname::gethostname()
            .to_str()
            .unwrap_or("Unknown Device")
            .to_string();
        user_setting.general.device_name = hostname;
        // 保存更新后的配置
        if let Err(e) = user_setting.save(None) {
            error!("Fail to save default device name: {}", e);
        }
    }

    // 确保配置已保存到全局 CONFIG 变量中
    {
        let mut global_setting = SETTING
            .write()
            .expect("Failed to acquire write lock on SETTING");
        *global_setting = user_setting.clone();
    }

    // 检查 vault 状态一致性（在单例初始化之前）
    // 如果状态不一致，提供恢复提示
    let vault_key_path = PasswordManager::get_vault_key_path();
    let snapshot_path = PasswordManager::get_snapshot_path();
    let vault_exists = vault_key_path.exists();
    let snapshot_exists = snapshot_path.exists();

    if vault_exists != snapshot_exists {
        // 状态不一致 - 一个文件存在另一个不存在
        error!(
            "Vault state inconsistent: vault_key={}, snapshot={}. \
             Please delete both files to reset: {:?} and {:?}",
            vault_exists, snapshot_exists, vault_key_path, snapshot_path
        );
        panic!(
            "Vault state inconsistent. Please delete both files to reset:\n  {:?}\n  {:?}",
            vault_key_path, snapshot_path
        );
    }

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
                {
                    let mut setting = SETTING
                        .write()
                        .expect("Failed to acquire write lock on SETTING");
                    setting.set_device_id(device.id.clone());
                }
                log::info!("Self device already exists: {}", device.id);
                device.id
            }
            Ok(None) => {
                // TODO: 获取本地IP地址
                // 目前只支持一个 ip ，后续可能同时支持多个 ip
                let local_ip = utils::helpers::get_local_ip();
                let result =
                    manager.register_self_device(local_ip, user_setting.network.webserver_port);
                let device_id = match result {
                    Ok(id) => id,
                    Err(e) => {
                        error!("Failed to register self device: {}", e);
                        panic!("Failed to register self device: {}", e);
                    }
                };
                log::info!("Registered self device with ID: {}", device_id);
                {
                    let mut setting = SETTING
                        .write()
                        .expect("Failed to acquire write lock on SETTING");
                    setting.set_device_id(device_id.clone());
                }

                device_id
            }
            Err(e) => {
                panic!("Failed to get self device: {}", e);
            }
        }
    };

    // 运行应用
    run_app(user_setting, device_id);
}

// 运行应用程序
fn run_app(user_setting: Setting, device_id: String) {
    use tauri::Builder;
    use tauri_plugin_autostart::MacosLauncher;
    use tauri_plugin_decorum;
    use tauri_plugin_single_instance;
    use tauri_plugin_stronghold;

    // Create command channels BEFORE setup
    let (clipboard_cmd_tx, clipboard_cmd_rx) = mpsc::channel(100);
    let (p2p_cmd_tx, p2p_cmd_rx) = mpsc::channel(100);

    // Create AppRuntimeHandle with config
    let config = Arc::new(user_setting.clone());
    let runtime_handle =
        AppRuntimeHandle::new(clipboard_cmd_tx.clone(), p2p_cmd_tx.clone(), config);

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
        .manage(Arc::new(std::sync::Mutex::new(
            api::event::EventListenerState::default(),
        )))
        .manage(runtime_handle)
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

            let _window = win_builder.build().expect("Failed to build main window");

            // macOS specific window styling will be handled by the rounded corners plugin
            // The plugin will set up rounded corners, traffic lights positioning, and transparency

            // 创建 AppRuntime
            let app_handle = app.handle().clone();
            let device_name = user_setting.general.device_name.clone();

            // 启动 AppRuntime
            // Note: Encryption initialization happens first in the same task,
            // ensuring it completes before AppRuntime is created
            tauri::async_runtime::spawn(async move {
                // Step 1: Initialize unified encryption
                let encryption_init_result = match api::setting::get_encryption_password().await {
                    Ok(password) => {
                        log::info!("Encryption password found, initializing unified encryption");
                        match api::encryption::initialize_unified_encryption(password).await {
                            Ok(_) => {
                                log::info!("Unified encryption initialized successfully");
                                true
                            }
                            Err(e) => {
                                log::error!("Failed to initialize unified encryption: {}", e);
                                false
                            }
                        }
                    }
                    Err(e) => {
                        // Password not set is OK for first-time users
                        if e.contains("未设置") || e.contains("not set") {
                            log::info!(
                                "No encryption password set, will prompt user during onboarding"
                            );
                        } else {
                            log::error!("Failed to check encryption password: {}", e);
                        }
                        false
                    }
                };

                // Step 2: Create AppRuntime (encryption is now ready)
                let app_runtime = match AppRuntime::new_with_channels(
                    user_setting.clone(),
                    device_id,
                    device_name,
                    app_handle,
                    clipboard_cmd_rx,
                    p2p_cmd_rx,
                )
                .await
                {
                    Ok(runtime) => runtime,
                    Err(e) => {
                        error!("Failed to create AppRuntime: {}", e);
                        // If encryption was not initialized, provide a helpful error
                        if !encryption_init_result {
                            error!(
                                "AppRuntime creation failed - encryption was not initialized. \
                                 Please set an encryption password first."
                            );
                        }
                        return;
                    }
                };

                // Step 3: Start the runtime
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
            api::event::listen_p2p_pairing_request,
            api::event::stop_listen_p2p_pairing_request,
            api::event::listen_p2p_pin_ready,
            api::event::stop_listen_p2p_pin_ready,
            api::event::listen_p2p_pairing_complete,
            api::event::stop_listen_p2p_pairing_complete,
            api::event::listen_p2p_pairing_failed,
            api::event::stop_listen_p2p_pairing_failed,
            api::onboarding::check_onboarding_status,
            api::onboarding::complete_onboarding,
            api::onboarding::setup_encryption_password,
            api::onboarding::get_device_id,
            api::onboarding::save_device_info,
            api::vault::check_vault_status,
            api::vault::reset_vault,
            api::p2p::get_local_peer_id,
            api::p2p::get_p2p_peers,
            api::p2p::get_local_device_info,
            api::p2p::get_paired_peers,
            api::p2p::get_paired_peers_with_status,
            api::p2p::initiate_p2p_pairing,
            api::p2p::verify_p2p_pairing_pin,
            api::p2p::reject_p2p_pairing,
            api::p2p::unpair_p2p_device,
            api::p2p::accept_p2p_pairing,
            api::encryption::initialize_unified_encryption,
            api::encryption::verify_encryption_password,
            api::encryption::change_encryption_password,
            api::encryption::is_unified_encryption_initialized,
            plugins::mac_rounded_corners::enable_rounded_corners,
            plugins::mac_rounded_corners::enable_modern_window_style,
            plugins::mac_rounded_corners::reposition_traffic_lights,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
