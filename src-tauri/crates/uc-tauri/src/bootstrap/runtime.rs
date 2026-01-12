use uc_app::{App, AppDeps};
use uc_core::config::AppConfig;

/// Seed for creating the application runtime.
///
/// This is an assembly context that holds the AppConfig
/// before Tauri setup phase completes. It does NOT contain
/// a fully constructed runtime - that happens in the setup phase.
///
/// ## English
///
/// This struct serves as a bridge between:
/// - Phase 1: Configuration loading (pre-Tauri)
/// - Phase 2: Dependency wiring (Tauri setup)
/// - Phase 3: App construction (post-setup)
///
/// ## 中文
///
/// 此结构作为以下阶段之间的桥梁：
/// - 阶段 1：配置加载（Tauri 之前）
/// - 阶段 2：依赖连接（Tauri 设置）
/// - 阶段 3：应用构造（设置之后）
pub struct AppRuntimeSeed {
    /// Application configuration loaded from TOML
    /// 从 TOML 加载的应用配置
    pub config: AppConfig,
}

/// Create the runtime seed without touching Tauri.
///
/// This function must not depend on Tauri or any UI framework.
/// 不依赖 Tauri 或任何 UI 框架创建运行时种子。
///
/// ## Phase Integration / 阶段集成
///
/// - **Phase 1**: Call this after `load_config()` to create the seed
/// - **Phase 2**: Pass seed to `wire_dependencies()` in Tauri setup
/// - **Phase 3**: Call `create_app()` with wired dependencies
///
/// ## English
///
/// This is the entry point for the bootstrap sequence:
/// 1. `load_config()` → reads TOML into `AppConfig`
/// 2. `create_runtime()` → wraps config in `AppRuntimeSeed`
/// 3. `wire_dependencies()` → creates ports from config
/// 4. `create_app()` → constructs `App` from dependencies
pub fn create_runtime(config: AppConfig) -> anyhow::Result<AppRuntimeSeed> {
    Ok(AppRuntimeSeed { config })
}

/// Create the App instance from wired dependencies.
/// 从已连接的依赖创建 App 实例。
///
/// ## English
///
/// This function is called in Phase 3 (after Tauri setup completes)
/// to construct the final `App` instance from the dependencies
/// that were wired in Phase 2.
///
/// This is a direct construction function - NOT a builder pattern.
/// All dependencies must be provided; no defaults, no optionals.
///
/// ## 中文
///
/// 此函数在阶段 3（Tauri 设置完成后）调用，
/// 用于从阶段 2 中连接的依赖构造最终的 `App` 实例。
///
/// 这是一个直接构造函数 - 不是 Builder 模式。
/// 必须提供所有依赖；无默认值，无可选项。
///
/// # Parameters / 参数
///
/// - `deps`: Application dependencies wired from configuration
///           从配置连接的应用依赖
///
/// # Returns / 返回
///
/// - `App`: Fully constructed application runtime
///          完全构造的应用运行时
///
/// # Phase 3 Integration / 阶段 3 集成
///
/// This function completes the bootstrap sequence:
/// ```text
/// load_config() → create_runtime() → wire_dependencies() → create_app()
///     ↓                 ↓                    ↓                    ↓
///   AppConfig      AppRuntimeSeed        AppDeps               App
/// ```
pub fn create_app(deps: AppDeps) -> App {
    App::new(deps)
}
