// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::Arc;

use log::error;
use tauri::{WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_single_instance;
use tauri_plugin_stronghold;
use tokio::sync::mpsc;

use uc_core::config::AppConfig;
use uc_core::ports::ClipboardChangeHandler;
use uc_platform::ipc::PlatformCommand;
use uc_platform::ports::PlatformCommandExecutorPort;
use uc_platform::runtime::event_bus::{
    PlatformCommandReceiver, PlatformEventReceiver, PlatformEventSender,
};
use uc_platform::runtime::runtime::PlatformRuntime;
use uc_tauri::bootstrap::{load_config, wire_dependencies, AppRuntime};
use uc_tauri::bootstrap::tracing as bootstrap_tracing;

// Platform-specific command modules
mod plugins;

#[cfg(target_os = "macos")]
use plugins::{enable_modern_window_style, enable_rounded_corners, reposition_traffic_lights};

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

/// Main entry point
fn main() {
    // Initialize tracing subscriber FIRST (before any logging)
    // This sets up the tracing infrastructure and enables log-tracing bridge
    if let Err(e) = bootstrap_tracing::init_tracing_subscriber() {
        eprintln!("Failed to initialize tracing: {}", e);
        std::process::exit(1);
    }

    // NOTE: In a production application, we would:
    // 1. Load configuration from a proper path
    // 2. Handle configuration errors gracefully

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

/// Macro to generate invoke handler with platform-specific commands
macro_rules! generate_invoke_handler {
    () => {
        tauri::generate_handler![
            // Clipboard commands
            uc_tauri::commands::clipboard::get_clipboard_entries,
            uc_tauri::commands::clipboard::delete_clipboard_entry,
            uc_tauri::commands::clipboard::capture_clipboard,
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
            enable_rounded_corners,
            #[cfg(target_os = "macos")]
            enable_modern_window_style,
            #[cfg(target_os = "macos")]
            reposition_traffic_lights,
        ]
    };
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

    // Create AppRuntime from dependencies
    let runtime = AppRuntime::new(deps);

    // Wrap runtime in Arc for clipboard handler (PlatformRuntime needs Arc<dyn ClipboardChangeHandler>)
    let runtime_for_handler = Arc::new(runtime);

    // Get a reference to the runtime for Tauri state management
    // We need to use Arc::try_unwrap() or create a clone without moving
    // Since AppRuntime doesn't implement Clone, we need a different approach
    // The solution: manage Arc<AppRuntime> and update commands to use State<'_, Arc<AppRuntime>>
    // For now, let's use the Arc directly for Tauri state management
    let runtime_for_tauri = runtime_for_handler.clone();

    // Create event channels for PlatformRuntime
    let (platform_event_tx, platform_event_rx): (PlatformEventSender, PlatformEventReceiver) =
        mpsc::channel(100);
    let (platform_cmd_tx, platform_cmd_rx): (
        tokio::sync::mpsc::Sender<uc_platform::ipc::PlatformCommand>,
        PlatformCommandReceiver,
    ) = mpsc::channel(100);

    // Create clipboard handler from runtime (AppRuntime implements ClipboardChangeHandler)
    let clipboard_handler: Arc<dyn ClipboardChangeHandler> = runtime_for_handler.clone();

    log::info!("Creating platform runtime with clipboard callback");

    // Note: PlatformRuntime will be started in setup block
    // The actual startup will be completed in a follow-up task

    Builder::default()
        // Manage Arc<AppRuntime> for use case access
        // NOTE: Commands need to use State<'_, Arc<AppRuntime>> instead of State<'_, AppRuntime>
        .manage(runtime_for_tauri)
        // Initialize logging system
        .plugin(logging::get_builder().build())
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

            let _window = match win_builder.build() {
                Ok(window) => window,
                Err(e) => {
                    log::error!("Failed to build main window: {}", e);
                    return Err(Box::new(e));
                }
            };

            // Start the platform runtime in background
            let platform_cmd_tx_for_spawn = platform_cmd_tx.clone();
            let platform_event_tx_clone = platform_event_tx.clone();
            tauri::async_runtime::spawn(async move {
                log::info!("Platform runtime task started");

                // Send StartClipboardWatcher command to enable monitoring
                match platform_cmd_tx_for_spawn
                    .send(PlatformCommand::StartClipboardWatcher)
                    .await
                {
                    Ok(_) => log::info!("StartClipboardWatcher command sent"),
                    Err(e) => log::error!("Failed to send StartClipboardWatcher command: {}", e),
                }

                // Create PlatformRuntime with the callback
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

                // Start the platform runtime event loop
                platform_runtime.start().await;

                log::info!("Platform runtime task ended");
            });

            log::info!("App runtime initialized with clipboard capture integration");
            log::info!("Platform runtime started with clipboard callback");

            Ok(())
        })
        // Register Tauri command handlers
        // Commands are defined in uc-tauri crate and need to be referenced by full path
        .invoke_handler(generate_invoke_handler!())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
