// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::Arc;

use log::error;
use tauri::Emitter;
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_single_instance;
use tauri_plugin_stronghold;
use tokio::sync::mpsc;

use uc_core::config::AppConfig;
use uc_core::ports::ClipboardChangeHandler;
use uc_platform::ipc::PlatformCommand;
use uc_platform::ports::PlatformCommandExecutorPort;
use uc_core::ports::AppDirsPort;
use uc_platform::runtime::event_bus::{
    PlatformCommandReceiver, PlatformEventReceiver, PlatformEventSender,
};
use uc_platform::runtime::runtime::PlatformRuntime;
use uc_tauri::bootstrap::tracing as bootstrap_tracing;
use uc_tauri::bootstrap::{load_config, wire_dependencies, AppRuntime};

// Platform-specific command modules
mod plugins;

/// Simple executor for platform commands
///
/// This is a placeholder implementation that logs commands.
/// In a full implementation, this would execute the actual platform commands.
struct SimplePlatformCommandExecutor;

#[async_trait::async_trait]
impl PlatformCommandExecutorPort for SimplePlatformCommandExecutor {
    async fn execute(&self, command: PlatformCommand) -> anyhow::Result<()> {
        // For now, just acknowledge the command
        // TODO: Implement actual command execution in future tasks
        match command {
            PlatformCommand::StartClipboardWatcher => {
                log::info!("StartClipboardWatcher command received");
            }
            PlatformCommand::StopClipboardWatcher => {
                log::info!("StopClipboardWatcher command received");
            }
            PlatformCommand::ReadClipboard => {
                log::info!("ReadClipboard command received (not implemented)");
            }
            PlatformCommand::WriteClipboard { .. } => {
                log::info!("WriteClipboard command received (not implemented)");
            }
            PlatformCommand::Shutdown => {
                log::info!("Shutdown command received (not implemented)");
            }
        }
        Ok(())
    }
}

/// Starts the application.
///
/// Initializes tracing, attempts to load `config.toml` (development mode), falls back to system
/// defaults using the platform app-data directory when no config file is present, and then runs
/// the Tauri application. On fatal initialization failures (tracing or app-data resolution) the
/// process exits with code 1.
///
/// # Examples
///
/// ```no_run
/// // Running the application (example; do not run in doctests)
/// crate::main();
/// ```
fn main() {
    // Initialize tracing subscriber FIRST (before any logging)
    // This sets up the tracing infrastructure and enables log-tracing bridge
    if let Err(e) = bootstrap_tracing::init_tracing_subscriber() {
        eprintln!("Failed to initialize tracing: {}", e);
        std::process::exit(1);
    }

    // NOTE: config.toml is optional and intended for development use only
    // Production environment uses system-default paths automatically

    let config_path = PathBuf::from("config.toml");

    // Load configuration using the new bootstrap flow
    let config = match load_config(config_path) {
        Ok(config) => {
            log::info!("Loaded config from config.toml (development mode)");
            config
        }
        Err(e) => {
            log::debug!("No config.toml found, using system defaults: {}", e);

            let app_dirs = match uc_platform::app_dirs::DirsAppDirsAdapter::new().get_app_dirs() {
                Ok(dirs) => dirs,
                Err(err) => {
                    error!("Failed to determine system data directory: {}", err);
                    error!("Please ensure your platform's data directory is accessible");
                    error!("macOS: ~/Library/Application Support/");
                    error!("Linux: ~/.local/share/");
                    error!("Windows: %LOCALAPPDATA%");
                    std::process::exit(1);
                }
            };

            AppConfig::with_system_defaults(app_dirs.app_data_root)
        }
    };

    // Run the application with the loaded config
    run_app(config);
}

/// Macro to generate invoke handler with platform-specific commands
macro_rules! generate_invoke_handler {
    () => {
        tauri::generate_handler![
            // Clipboard commands
            uc_tauri::commands::clipboard::get_clipboard_entries,
            uc_tauri::commands::clipboard::get_clipboard_entry_detail,
            uc_tauri::commands::clipboard::delete_clipboard_entry,
            // Encryption commands
            uc_tauri::commands::encryption::initialize_encryption,
            uc_tauri::commands::encryption::is_encryption_initialized,
            // Settings commands
            uc_tauri::commands::settings::get_settings,
            uc_tauri::commands::settings::update_settings,
            // Onboarding commands
            uc_tauri::commands::onboarding::get_onboarding_state,
            uc_tauri::commands::onboarding::complete_onboarding,
            uc_tauri::commands::onboarding::initialize_onboarding,
            // macOS-specific commands (conditionally compiled)
            #[cfg(target_os = "macos")]
            plugins::mac_rounded_corners::enable_rounded_corners,
            #[cfg(target_os = "macos")]
            plugins::mac_rounded_corners::enable_modern_window_style,
            #[cfg(target_os = "macos")]
            plugins::mac_rounded_corners::reposition_traffic_lights,
        ]
    };
}

/// Run the Tauri application
fn run_app(config: AppConfig) {
    use tauri::Builder;

    // Create event channels for PlatformRuntime
    let (platform_event_tx, platform_event_rx): (PlatformEventSender, PlatformEventReceiver) =
        mpsc::channel(100);
    let (platform_cmd_tx, platform_cmd_rx): (
        tokio::sync::mpsc::Sender<uc_platform::ipc::PlatformCommand>,
        PlatformCommandReceiver,
    ) = mpsc::channel(100);

    // Wire all dependencies using the new bootstrap flow
    let deps = match wire_dependencies(&config, platform_cmd_tx.clone()) {
        Ok(deps) => deps,
        Err(e) => {
            error!("Failed to wire dependencies: {}", e);
            panic!("Dependency wiring failed: {}", e);
        }
    };

    // Create AppRuntime from dependencies
    let runtime = AppRuntime::new(deps);

    // Wrap runtime in Arc for clipboard handler (PlatformRuntime needs Arc<dyn ClipboardChangeHandler>)
    let runtime_for_handler = Arc::new(runtime);

    // Clone Arc for Tauri state management (will have app_handle injected in setup)
    let runtime_for_tauri = runtime_for_handler.clone();

    // Create clipboard handler from runtime (AppRuntime implements ClipboardChangeHandler)
    let clipboard_handler: Arc<dyn ClipboardChangeHandler> = runtime_for_handler.clone();

    log::info!("Creating platform runtime with clipboard callback");

    // Note: PlatformRuntime will be started in setup block
    // The actual startup will be completed in a follow-up task

    Builder::default()
        // Register AppRuntime for Tauri commands
        .manage(runtime_for_tauri)
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
        .setup(move |app| {
            // Set AppHandle on runtime so it can emit events to frontend
            // In Tauri 2, use app.handle() to get the AppHandle
            runtime_for_handler.set_app_handle(app.handle().clone());
            log::info!("AppHandle set on AppRuntime for event emission");

            // Clone handle for use in async block
            let runtime_for_unlock = runtime_for_handler.clone();
            let platform_cmd_tx_for_spawn = platform_cmd_tx.clone();
            let platform_event_tx_clone = platform_event_tx.clone();

            tauri::async_runtime::spawn(async move {
                log::info!("Platform runtime task started");

                // 1. Check if encryption initialized and auto-unlock
                let uc = runtime_for_unlock.usecases().auto_unlock_encryption_session();
                let should_start_watcher = match uc.execute().await {
                    Ok(true) => {
                        log::info!("Encryption session auto-unlocked successfully");
                        true
                    }
                    Ok(false) => {
                        log::info!("Encryption not initialized, clipboard watcher will not start");
                        log::info!("User must set encryption password via onboarding");
                        false
                    }
                    Err(e) => {
                        log::error!("Auto-unlock failed: {:?}", e);
                        // Emit error event to frontend for user notification
                        let app_handle_guard = runtime_for_unlock.app_handle();
                        if let Some(app) = app_handle_guard.as_ref() {
                            if let Err(emit_err) = app.emit("encryption-auto-unlock-error", format!("{}", e)) {
                                log::warn!("Failed to emit encryption-auto-unlock-error event: {}", emit_err);
                            }
                        }
                        drop(app_handle_guard);
                        false
                    }
                };

                // 2. Create PlatformRuntime
                let executor = Arc::new(SimplePlatformCommandExecutor);
                let platform_runtime = match PlatformRuntime::new(
                    platform_event_tx_clone,
                    platform_event_rx,
                    platform_cmd_rx,
                    executor,
                    Some(clipboard_handler),
                ) {
                    Ok(rt) => rt,
                    Err(e) => {
                        log::error!("Failed to create platform runtime: {}", e);
                        return;
                    }
                };

                // 3. Start watcher if encryption is ready
                if should_start_watcher {
                    match platform_cmd_tx_for_spawn
                        .send(PlatformCommand::StartClipboardWatcher)
                        .await
                    {
                        Ok(_) => log::info!("StartClipboardWatcher command sent"),
                        Err(e) => {
                            log::error!("Failed to send StartClipboardWatcher command: {}", e)
                        }
                    }
                }

                platform_runtime.start().await;
                log::info!("Platform runtime task ended");
            });

            log::info!("App runtime initialized with clipboard capture integration");
            Ok(())
        })
        .invoke_handler(generate_invoke_handler!())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}