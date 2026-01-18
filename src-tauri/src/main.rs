// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_single_instance;
use tauri_plugin_stronghold;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use uc_core::config::AppConfig;
use uc_core::ports::AppDirsPort;
use uc_core::ports::ClipboardChangeHandler;
use uc_platform::ipc::PlatformCommand;
use uc_platform::ports::PlatformCommandExecutorPort;
use uc_platform::runtime::event_bus::{
    PlatformCommandReceiver, PlatformEventReceiver, PlatformEventSender,
};
use uc_platform::runtime::runtime::PlatformRuntime;
use uc_tauri::bootstrap::tracing as bootstrap_tracing;
use uc_tauri::bootstrap::{ensure_default_device_name, load_config, wire_dependencies, AppRuntime};

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
                info!("StartClipboardWatcher command received");
            }
            PlatformCommand::StopClipboardWatcher => {
                info!("StopClipboardWatcher command received");
            }
            PlatformCommand::ReadClipboard => {
                info!("ReadClipboard command received (not implemented)");
            }
            PlatformCommand::WriteClipboard { .. } => {
                info!("WriteClipboard command received (not implemented)");
            }
            PlatformCommand::Shutdown => {
                info!("Shutdown command received (not implemented)");
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
            info!("Loaded config from config.toml (development mode)");
            config
        }
        Err(e) => {
            debug!("No config.toml found, using system defaults: {}", e);

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
            // Autostart commands
            uc_tauri::commands::autostart::enable_autostart,
            uc_tauri::commands::autostart::disable_autostart,
            uc_tauri::commands::autostart::is_autostart_enabled,
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

    info!("Creating platform runtime with clipboard callback");

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
            info!("AppHandle set on AppRuntime for event emission");

            if app.get_webview_window("splashscreen").is_none() {
                match WebviewWindowBuilder::new(
                    app,
                    "splashscreen",
                    WebviewUrl::App("/splashscreen.html".into()),
                )
                .title("UniClipboard")
                .inner_size(800.0, 600.0)
                .resizable(false)
                .decorations(false)
                .always_on_top(true)
                .skip_taskbar(true)
                .build()
                {
                    Ok(window) => {
                        info!("Splashscreen window created");

                        // Apply rounded corners to splashscreen window (macOS only)
                        #[cfg(target_os = "macos")]
                        if let Err(e) = plugins::mac_rounded_corners::apply_modern_window_style(
                            &window,
                            plugins::mac_rounded_corners::WindowStyleConfig {
                                corner_radius: 16.0,
                                has_shadow: true,
                                ..Default::default()
                            },
                        ) {
                            warn!("Failed to apply rounded corners to splashscreen: {}", e);
                        } else {
                            info!("Applied rounded corners to splashscreen window");
                        }
                    }
                    Err(e) => warn!("Failed to create splashscreen window: {}", e),
                }
            }

            // Ensure the main window becomes visible even if backend-ready is never emitted.
            let app_handle_for_fallback = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(Duration::from_secs(30)).await;

                if let Some(main_window) = app_handle_for_fallback.get_webview_window("main") {
                    if let Err(e) = main_window.show() {
                        warn!("Failed to show main window after splash timeout: {}", e);
                    }
                } else {
                    warn!("Main window not found after splash timeout");
                }

                if let Some(splash_window) =
                    app_handle_for_fallback.get_webview_window("splashscreen")
                {
                    if let Err(e) = splash_window.close() {
                        warn!("Failed to close splashscreen after timeout: {}", e);
                    }
                }
            });

            // Clone app handle for the spawn task
            let app_handle_for_spawn = app.handle().clone();

            // Spawn the initialization task immediately (don't wait for frontend)
            let runtime = runtime_for_handler.clone();
            let platform_event_tx_clone = platform_event_tx.clone();
            tauri::async_runtime::spawn(async move {
                info!("Starting backend initialization");

                // 0. Ensure device name is initialized (runs on every startup)
                if let Err(e) = ensure_default_device_name(runtime.deps.settings.clone()).await {
                    warn!("Failed to initialize default device name: {}", e);
                    // Non-fatal: continue startup even if device name initialization fails
                }

                // 1. Check if encryption initialized and auto-unlock
                let uc = runtime.usecases().auto_unlock_encryption_session();
                let should_start_watcher = match uc.execute().await {
                    Ok(true) => {
                        info!("Encryption session auto-unlocked successfully");
                        true
                    }
                    Ok(false) => {
                        info!("Encryption not initialized, clipboard watcher will not start");
                        info!("User must set encryption password via onboarding");
                        false
                    }
                    Err(e) => {
                        error!("Auto-unlock failed: {:?}", e);
                        // Emit error event to frontend for user notification
                        let app_handle_guard = runtime.app_handle();
                        if let Some(app_handle) = app_handle_guard.as_ref() {
                            if let Err(emit_err) =
                                app_handle.emit("encryption-auto-unlock-error", format!("{}", e))
                            {
                                warn!(
                                    "Failed to emit encryption-auto-unlock-error event: {}",
                                    emit_err
                                );
                            }
                        }
                        drop(app_handle_guard);
                        false
                    }
                };

                // 2. Create PlatformRuntime
                info!("Creating PlatformRuntime...");
                let executor = Arc::new(SimplePlatformCommandExecutor);
                let platform_runtime = match PlatformRuntime::new(
                    platform_event_tx_clone,
                    platform_event_rx,
                    platform_cmd_rx,
                    executor,
                    Some(clipboard_handler),
                ) {
                    Ok(rt) => {
                        info!("PlatformRuntime created successfully");
                        rt
                    }
                    Err(e) => {
                        error!("Failed to create platform runtime: {}", e);
                        return;
                    }
                };

                // 3. Start watcher if encryption is ready
                if should_start_watcher {
                    match runtime.usecases().start_clipboard_watcher().execute().await {
                        Ok(_) => info!("Clipboard watcher started successfully"),
                        Err(e) => {
                            error!("Failed to start clipboard watcher: {}", e);
                            // Emit error event to frontend for user notification
                            let app_handle_guard = runtime.app_handle();
                            if let Some(app_handle) = app_handle_guard.as_ref() {
                                if let Err(emit_err) = app_handle
                                    .emit("clipboard-watcher-start-failed", format!("{}", e))
                                {
                                    warn!(
                                        "Failed to emit clipboard-watcher-start-failed event: {}",
                                        emit_err
                                    );
                                }
                            }
                            drop(app_handle_guard);
                        }
                    }
                }

                // 4. Emit backend-ready event to notify frontend
                info!("Emitting backend-ready event...");
                if let Err(e) = app_handle_for_spawn.emit("backend-ready", ()) {
                    error!("Failed to emit backend-ready event: {}", e);
                } else {
                    info!("backend-ready event emitted successfully");
                }

                if let Some(main_window) = app_handle_for_spawn.get_webview_window("main") {
                    if let Err(e) = main_window.show() {
                        warn!("Failed to show main window after backend-ready: {}", e);
                    }
                } else {
                    warn!("Main window not found when backend-ready emitted");
                }

                if let Some(splash_window) = app_handle_for_spawn.get_webview_window("splashscreen")
                {
                    if let Err(e) = splash_window.close() {
                        warn!("Failed to close splashscreen after backend-ready: {}", e);
                    }
                }

                // 5. Start platform runtime (this is an infinite loop that runs until app exits)
                platform_runtime.start().await;

                info!("Platform runtime task ended");
            });

            info!("App runtime initialized, backend initialization started");
            Ok(())
        })
        .invoke_handler(generate_invoke_handler!())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
