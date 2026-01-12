use super::runtime::AppRuntimeSeed;
use crate::adapters::{TauriAutostart, TauriUiPort};
use std::sync::Arc;
use uc_app::App;

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

    // Use legacy AppBuilder pattern for backward compatibility
    // This will be replaced in Phase 3
    let app = Arc::new(
        uc_app::AppBuilder::new()
            .with_autostart(autostart)
            .with_ui_port(ui_port)
            .build()?,
    );

    Ok(Runtime::new(app))
}
