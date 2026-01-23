// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use tauri::http::header::{
    HeaderValue, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE,
};
use tauri::http::{Request, Response, StatusCode};
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
use uc_tauri::bootstrap::{
    ensure_default_device_name, load_config, start_background_tasks, wire_dependencies, AppRuntime,
};
use uc_tauri::protocol::{parse_uc_request, UcRoute};

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

fn is_allowed_cors_origin(origin: &str) -> bool {
    origin == "tauri://localhost"
        || origin == "http://tauri.localhost"
        || origin == "https://tauri.localhost"
        || origin.starts_with("http://localhost:")
        || origin.starts_with("http://127.0.0.1:")
        || origin.starts_with("http://[::1]:")
}

fn set_cors_headers(response: &mut Response<Vec<u8>>, origin: Option<&str>) {
    let origin = match origin {
        Some(origin) if is_allowed_cors_origin(origin) => origin,
        _ => return,
    };

    match HeaderValue::from_str(origin) {
        Ok(value) => {
            response
                .headers_mut()
                .insert(ACCESS_CONTROL_ALLOW_ORIGIN, value);
        }
        Err(err) => {
            error!(error = %err, "Invalid origin for CORS response");
        }
    }

    if let Ok(value) = HeaderValue::from_str("GET") {
        response
            .headers_mut()
            .insert(ACCESS_CONTROL_ALLOW_METHODS, value);
    }
}

fn build_response(
    status: StatusCode,
    content_type: Option<&str>,
    body: Vec<u8>,
    origin: Option<&str>,
) -> Response<Vec<u8>> {
    let mut response = Response::new(body);
    *response.status_mut() = status;

    if let Some(content_type) = content_type {
        match HeaderValue::from_str(content_type) {
            Ok(value) => {
                response.headers_mut().insert(CONTENT_TYPE, value);
            }
            Err(err) => {
                error!(error = %err, "Invalid content type for response");
            }
        }
    }

    set_cors_headers(&mut response, origin);

    response
}

fn text_response(status: StatusCode, message: &str, origin: Option<&str>) -> Response<Vec<u8>> {
    build_response(
        status,
        Some("text/plain"),
        message.as_bytes().to_vec(),
        origin,
    )
}

async fn resolve_uc_request(
    app_handle: tauri::AppHandle,
    request: Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let uri = request.uri();
    let host = uri.host().unwrap_or_default();
    let path = uri.path();
    let origin = request
        .headers()
        .get("Origin")
        .and_then(|value| value.to_str().ok());

    let route = match parse_uc_request(&request) {
        Ok(route) => route,
        Err(err) => {
            error!(
                error = %err,
                host = %host,
                path = %path,
                "Failed to parse uc URI request"
            );
            return text_response(err.status_code(), err.response_message(), origin);
        }
    };

    match route {
        UcRoute::Blob { blob_id } => resolve_uc_blob_request(app_handle, blob_id, origin).await,
        UcRoute::Thumbnail { representation_id } => {
            resolve_uc_thumbnail_request(app_handle, representation_id, origin).await
        }
    }
}

async fn resolve_uc_blob_request(
    app_handle: tauri::AppHandle,
    blob_id: uc_core::BlobId,
    origin: Option<&str>,
) -> Response<Vec<u8>> {
    let runtime = match app_handle.try_state::<Arc<AppRuntime>>() {
        Some(state) => state,
        None => {
            error!("AppRuntime state not managed for uc URI handling");
            return text_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Runtime not ready",
                origin,
            );
        }
    };

    let use_case = runtime.usecases().resolve_blob_resource();
    match use_case.execute(&blob_id).await {
        Ok(result) => build_response(
            StatusCode::OK,
            Some(
                result
                    .mime_type
                    .as_deref()
                    .unwrap_or("application/octet-stream"),
            ),
            result.bytes,
            origin,
        ),
        Err(err) => {
            let err_msg = err.to_string();
            error!(error = %err, blob_id = %blob_id, "Failed to resolve blob resource");
            let status = if err_msg.contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            text_response(status, "Failed to resolve blob resource", origin)
        }
    }
}

async fn resolve_uc_thumbnail_request(
    app_handle: tauri::AppHandle,
    representation_id: uc_core::ids::RepresentationId,
    origin: Option<&str>,
) -> Response<Vec<u8>> {
    let runtime = match app_handle.try_state::<Arc<AppRuntime>>() {
        Some(state) => state,
        None => {
            error!("AppRuntime state not managed for uc URI handling");
            return text_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Runtime not ready",
                origin,
            );
        }
    };

    let use_case = runtime.usecases().resolve_thumbnail_resource();
    match use_case.execute(&representation_id).await {
        Ok(result) => build_response(
            StatusCode::OK,
            Some(
                result
                    .mime_type
                    .as_deref()
                    .unwrap_or("application/octet-stream"),
            ),
            result.bytes,
            origin,
        ),
        Err(err) => {
            let err_msg = err.to_string();
            error!(
                error = %err,
                representation_id = %representation_id,
                "Failed to resolve thumbnail resource"
            );
            let status = if err_msg.contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            text_response(status, "Failed to resolve thumbnail resource", origin)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_headers_are_set_for_dev_origin() {
        let origin = "http://localhost:1420";
        let response = build_response(StatusCode::OK, None, vec![], Some(origin));

        let headers = response.headers();
        assert_eq!(
            headers
                .get(ACCESS_CONTROL_ALLOW_ORIGIN)
                .and_then(|value| value.to_str().ok()),
            Some(origin)
        );
        assert_eq!(
            headers
                .get(ACCESS_CONTROL_ALLOW_METHODS)
                .and_then(|value| value.to_str().ok()),
            Some("GET")
        );
    }

    #[test]
    fn test_cors_headers_not_set_for_untrusted_origin() {
        let response = build_response(StatusCode::OK, None, vec![], Some("https://example.com"));

        let headers = response.headers();
        assert!(headers.get(ACCESS_CONTROL_ALLOW_ORIGIN).is_none());
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
            uc_tauri::commands::clipboard::get_clipboard_entry_resource,
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
            // Startup commands
            uc_tauri::commands::startup::frontend_ready,
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
    let wired = match wire_dependencies(&config, platform_cmd_tx.clone()) {
        Ok(wired) => wired,
        Err(e) => {
            error!("Failed to wire dependencies: {}", e);
            panic!("Dependency wiring failed: {}", e);
        }
    };

    let deps = wired.deps;
    let background = wired.background;

    // Create AppRuntime from dependencies
    let runtime = AppRuntime::new(deps);

    // Wrap runtime in Arc for clipboard handler (PlatformRuntime needs Arc<dyn ClipboardChangeHandler>)
    let runtime_for_handler = Arc::new(runtime);

    // Clone Arc for Tauri state management (will have app_handle injected in setup)
    let runtime_for_tauri = runtime_for_handler.clone();

    // Startup barrier used to coordinate splashscreen close timing.
    // NOTE: Must be managed before startup to be available via tauri::State<T>.
    let startup_barrier = Arc::new(uc_tauri::commands::startup::StartupBarrier::default());

    // Create clipboard handler from runtime (AppRuntime implements ClipboardChangeHandler)
    let clipboard_handler: Arc<dyn ClipboardChangeHandler> = runtime_for_handler.clone();

    info!("Creating platform runtime with clipboard callback");

    // Note: PlatformRuntime will be started in setup block
    // The actual startup will be completed in a follow-up task

    Builder::default()
        // Register AppRuntime for Tauri commands
        .manage(runtime_for_tauri)
        .manage(startup_barrier.clone())
        .register_asynchronous_uri_scheme_protocol("uc", move |ctx, request, responder| {
            let app_handle = ctx.app_handle().clone();
            tauri::async_runtime::spawn(async move {
                let response = resolve_uc_request(app_handle, request).await;
                responder.respond(response);
            });
        })
        // Manual verification (dev):
        // 1) In frontend devtools: fetch("uc://blob/<blob_id>")
        // 2) In frontend devtools: fetch("uc://thumbnail/<representation_id>")
        // 3) Network should show 200 with Access-Control-Allow-Origin matching http://localhost:1420
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

            // Start background spooler and blob worker tasks
            start_background_tasks(background, &runtime_for_handler.deps);

            // Clone handles for async blocks
            let app_handle_for_startup = app.handle().clone();
            let startup_barrier_for_backend = startup_barrier.clone();
            let startup_barrier_for_timeout = startup_barrier.clone();

            if app.get_webview_window("splashscreen").is_none() {
                // Load settings BEFORE creating the window to avoid race condition
                info!("=== [LAYER 1] SPLASHSCREEN: Starting settings load ===");
                let runtime_clone = runtime_for_handler.clone();
                let (theme_color, theme_mode) = tauri::async_runtime::block_on(async move {
                    info!("=== [LAYER 1] SPLASHSCREEN: Inside async block, calling settings.load() ===");
                    let settings_result = runtime_clone.deps.settings.load().await;
                    info!("=== [LAYER 1] SPLASHSCREEN: Settings load completed, is_ok={} ===", settings_result.is_ok());
                    match settings_result {
                        Ok(settings) => {
                            let color = settings.general.theme_color.as_deref().unwrap_or("zinc").to_string();
                            let mode = settings.general.theme.clone();
                            info!("=== [LAYER 1] SPLASHSCREEN: Raw data - theme_color={:?}, theme_mode={:?} ===",
                                settings.general.theme_color, mode);
                            info!("=== [LAYER 1] SPLASHSCREEN: Processed - color={}, mode={:?} ===", color, mode);
                            (color, mode)
                        }
                        Err(e) => {
                            error!("=== [LAYER 1] SPLASHSCREEN: Failed to load settings: {} ===", e);
                            warn!("Failed to load settings for splashscreen theme: {}", e);
                            ("zinc".to_string(), uc_core::settings::model::Theme::System)
                        }
                    }
                });
                info!("=== [LAYER 1] SPLASHSCREEN: block_on completed, theme_color={}, theme_mode={:?} ===", theme_color, theme_mode);

                match WebviewWindowBuilder::new(
                    app,
                    "splashscreen",
                    WebviewUrl::App("/splashscreen.html".into()),
                )
                // IMPORTANT: splashscreen.html 在页面脚本里会同步读取 window.__SPLASH_THEME__
                // 必须在 document 脚本执行前注入，避免退化为默认主题 + 跟随系统深浅色。
                .initialization_script({
                    #[derive(Serialize)]
                    struct SplashTheme<'a> {
                        theme_color: &'a str,
                        #[serde(skip_serializing_if = "Option::is_none")]
                        mode: Option<&'a str>,
                    }

                    let mode_str = match theme_mode {
                        uc_core::settings::model::Theme::Light => Some("light"),
                        uc_core::settings::model::Theme::Dark => Some("dark"),
                        uc_core::settings::model::Theme::System => None,
                    };

                    let payload = SplashTheme {
                        theme_color: theme_color.as_str(),
                        mode: mode_str,
                    };

                    match serde_json::to_string(&payload) {
                        Ok(json) => format!("window.__SPLASH_THEME__ = {};", json),
                        Err(e) => {
                            warn!(
                                "Failed to serialize splashscreen theme payload, falling back to defaults: {}",
                                e
                            );
                            "window.__SPLASH_THEME__ = { theme_color: 'zinc' };".to_string()
                        }
                    }
                })
                .title("UniClipboard")
                .inner_size(800.0, 600.0)
                .resizable(false)
                .decorations(false)
                .always_on_top(true)
                .skip_taskbar(true)
                .build()
                {
                    Ok(window) => {
                        info!("=== [LAYER 2] SPLASHSCREEN: Window created successfully ===");

                        // Ensure splashscreen is visible immediately (explicit is better than implicit).
                        if let Err(e) = window.show() {
                            warn!("Failed to show splashscreen window: {}", e);
                        }

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

            // Dev-only safety net: avoid getting stuck on splashscreen if the frontend handshake doesn't arrive.
            // In release builds, we prefer strict coordination to avoid showing a blank main window.
            #[cfg(debug_assertions)]
            {
                let app_handle_timeout = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(15)).await;
                    if app_handle_timeout.get_webview_window("splashscreen").is_some() {
                        warn!(
                            "frontend_ready handshake not received in time; forcing startup barrier completion (debug)"
                        );
                        startup_barrier_for_timeout.mark_frontend_ready();
                        startup_barrier_for_timeout.try_finish(&app_handle_timeout);
                    }
                });
            }

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

                        // Emit event to notify frontend that encryption session is ready
                        let app_handle_guard = runtime.app_handle();
                        if let Some(app_handle) = app_handle_guard.as_ref() {
                            if let Err(e) = uc_tauri::events::forward_encryption_event(
                                app_handle,
                                uc_tauri::events::EncryptionEvent::SessionReady,
                            ) {
                                warn!("Failed to emit encryption session ready event: {}", e);
                            } else {
                                info!("Emitted encryption session ready event to frontend");
                            }
                        }
                        drop(app_handle_guard);

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
                            if let Err(emit_err) = uc_tauri::events::forward_encryption_event(
                                app_handle,
                                uc_tauri::events::EncryptionEvent::Failed {
                                    reason: e.to_string(),
                                },
                            ) {
                                warn!("Failed to emit encryption error event: {}", emit_err);
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

                // Mark backend-side startup tasks completed. We intentionally do NOT close the splashscreen
                // or show the main window here; that is driven by the frontend via `frontend_ready` to avoid
                // showing a blank main window when `index.html` does not contain an inline splash.
                startup_barrier_for_backend.mark_backend_ready();
                startup_barrier_for_backend.try_finish(&app_handle_for_startup);

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
