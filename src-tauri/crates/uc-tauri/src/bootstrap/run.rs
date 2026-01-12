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
/// This function uses the legacy AppBuilder pattern and will be removed
/// in Phase 3 (Task 6). Use the new dependency wiring approach instead:
///
/// ## English
/// - Phase 2: `wire_dependencies(seed, app_handle)` → creates `AppDeps`
/// - Phase 3: `create_app(deps)` → constructs `App`
///
/// ## 中文
/// - 阶段 2：`wire_dependencies(seed, app_handle)` → 创建 `AppDeps`
/// - 阶段 3：`create_app(deps)` → 构造 `App`
#[deprecated(note = "Use wire_dependencies() + create_app() instead")]
pub fn build_runtime(seed: AppRuntimeSeed, app_handle: &tauri::AppHandle) -> anyhow::Result<Runtime> {
    let autostart = Arc::new(TauriAutostart::new(app_handle.clone()));
    let ui_port = Arc::new(TauriUiPort::new(app_handle.clone(), "settings"));

    // Note: This uses the legacy AppBuilder pattern
    // This will be replaced in Phase 3 (Task 6)
    let app = Arc::new(
        uc_app::AppBuilder::new()
            .with_autostart(autostart)
            .with_ui_port(ui_port)
            .build()?,
    );

    Ok(Runtime::new(app))
}
