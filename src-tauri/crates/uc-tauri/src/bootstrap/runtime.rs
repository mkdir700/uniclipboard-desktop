use uc_app::AppBuilder;

/// Seed for creating the application runtime.
///
/// This is an assembly context that holds the AppBuilder
/// before Tauri setup phase completes. It does NOT contain
/// a fully constructed runtime - that happens in the setup phase.
pub struct AppRuntimeSeed {
    pub app_builder: AppBuilder,
}

/// Create the runtime seed without touching Tauri.
///
/// This function must not depend on Tauri or any UI framework.
pub fn create_runtime() -> anyhow::Result<AppRuntimeSeed> {
    Ok(AppRuntimeSeed {
        app_builder: AppBuilder::new(),
    })
}
