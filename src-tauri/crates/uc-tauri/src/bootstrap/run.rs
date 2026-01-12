use super::runtime::AppRuntimeSeed;
use crate::adapters::{TauriAutostart, TauriUiPort};
use std::sync::Arc;
use uc_app::{App, AppDeps};

/// The completed application runtime.
///
/// This struct holds the fully assembled App instance
/// and is managed by Tauri's state system.
pub struct Runtime {
    pub app: Arc<App>,
}

impl Runtime {
    pub fn new(app: Arc<App>) -> Self {
        Self { app }
    }
}

/// Run the Tauri application with the given runtime seed.
///
/// This function handles the Tauri setup phase where
/// AppHandle-dependent adapters are created and injected.
///
/// Note: This is a simplified version. The actual integration with
/// the existing main.rs will happen in later tasks.
pub fn run_app(_seed: AppRuntimeSeed) -> anyhow::Result<()> {
    // TODO: This will be properly integrated in Task 6
    // For now, this is just a placeholder to satisfy the compiler
    Ok(())
}

/// Build the completed runtime from the seed.
///
/// This should be called from the Tauri setup closure.
///
/// # DEPRECATED / 已弃用
///
/// This function is deprecated and will be removed in Phase 3 (Task 6).
/// The new approach uses `wire_dependencies()` + `create_app()`:
///
/// ## Migration / 迁移
///
/// **Old way (this function)**:
/// ```ignore
/// let seed = create_runtime(config);
/// // ... later in Tauri setup ...
/// let runtime = build_runtime(seed, app_handle);
/// ```
///
/// **New way (Phase 3)**:
/// ```ignore
/// let config = load_config(path);
/// let deps = wire_dependencies(&config);
/// // ... later in Tauri setup, create adapters that need AppHandle ...
/// let app = create_app(deps);
/// ```
#[deprecated(note = "Use wire_dependencies() + create_app() instead (Phase 3)")]
pub fn build_runtime(_seed: AppRuntimeSeed, app_handle: &tauri::AppHandle) -> anyhow::Result<Runtime> {
    let autostart = Arc::new(TauriAutostart::new(app_handle.clone()));
    let ui_port = Arc::new(TauriUiPort::new(app_handle.clone(), "settings"));

    // Use direct App construction with AppDeps
    // TODO: This will be replaced in Phase 3
    let deps = AppDeps {
        clipboard: todo!("Inject clipboard port"),
        clipboard_event_repo: todo!("Inject clipboard event repo"),
        representation_repo: todo!("Inject representation repo"),
        representation_materializer: todo!("Inject representation materializer"),
        encryption: todo!("Inject encryption port"),
        encryption_session: todo!("Inject encryption session"),
        keyring: todo!("Inject keyring port"),
        key_material: todo!("Inject key material port"),
        device_repo: todo!("Inject device repo"),
        device_identity: todo!("Inject device identity"),
        network: todo!("Inject network port"),
        blob_store: todo!("Inject blob store"),
        blob_repository: todo!("Inject blob repository"),
        blob_materializer: todo!("Inject blob materializer"),
        settings: todo!("Inject settings port"),
        ui_port,
        autostart,
        clock: todo!("Inject clock"),
        hash: todo!("Inject hash"),
    };

    let app = Arc::new(App::new(deps));

    Ok(Runtime::new(app))
}
