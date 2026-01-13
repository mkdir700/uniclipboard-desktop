// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use log::error;
use tauri::{WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_single_instance;
use tauri_plugin_stronghold;

use uc_core::config::AppConfig;
use uc_tauri::bootstrap::{load_config, wire_dependencies};

/// Main entry point
fn main() {
    // TODO: In a real application, we would:
    // 1. Load configuration from a proper path
    // 2. Handle configuration errors gracefully
    // 3. Initialize logging

    // For now, use a default config path
    let config_path = PathBuf::from("config.toml");

    // Load configuration using the new bootstrap flow
    let config = match load_config(config_path) {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load config: {}", e);
            // For now, use an empty config
            // In production, this should be handled by a use case
            AppConfig::empty()
        }
    };

    // Run the application with the loaded config
    run_app(config);
}

/// Run the Tauri application
fn run_app(config: AppConfig) {
    use tauri::Builder;

    // Wire all dependencies using the new bootstrap flow
    let deps = match wire_dependencies(&config) {
        Ok(deps) => deps,
        Err(e) => {
            error!("Failed to wire dependencies: {}", e);
            panic!("Dependency wiring failed: {}", e);
        }
    };

    Builder::default()
        // Manage AppDeps as Tauri state for command handlers
        .manage(deps)
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(
            tauri_plugin_stronghold::Builder::new(move |key| {
                // Use a simple password hash function
                // In production, this should use Argon2 or similar
                key.as_bytes().to_vec()
            })
            .build(),
        )
        .setup(move |app_handle| {
            // Create the main window
            let win_builder = WebviewWindowBuilder::new(app_handle, "main", WebviewUrl::default())
                .title("UniClipboard")
                .inner_size(800.0, 600.0)
                .min_inner_size(800.0, 600.0);

            // Use platform-specific title bar settings
            #[cfg(target_os = "macos")]
            let win_builder = win_builder.decorations(false);

            #[cfg(target_os = "windows")]
            let win_builder = win_builder.decorations(false).shadow(true);

            // Apply silent start setting
            let win_builder = if config.silent_start {
                win_builder.visible(false)
            } else {
                win_builder
            };

            let _window = win_builder.build().expect("Failed to build main window");

            // TODO: Start the app runtime
            // This will be implemented in later tasks
            // For now, we just create the window

            Ok(())
        })
        // Register Tauri command handlers
        // Commands are defined in uc-tauri crate and need to be referenced by full path
        .invoke_handler(tauri::generate_handler![
            // Clipboard commands
            uc_tauri::commands::clipboard::get_clipboard_entries,
            uc_tauri::commands::clipboard::delete_clipboard_entry,
            uc_tauri::commands::clipboard::capture_clipboard,
            // Encryption commands
            uc_tauri::commands::encryption::initialize_encryption,
            uc_tauri::commands::encryption::is_encryption_initialized,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
