//! # Use Cases Accessor
//!
//! This module provides the `UseCases` accessor which is attached to `AppRuntime`
//! to provide convenient access to all use cases with their dependencies pre-wired.
//!
//! ## Architecture
//!
//! - **uc-app/usecases**: Pure use cases with `new()` constructors taking ports
//! - **uc-tauri/bootstrap**: This module wires `Arc<dyn Port>` from AppDeps into use cases
//! - **Commands**: Call `runtime.usecases().xxx()` to get use case instances
//!
//! ## Usage
//!
//! ```rust,no_run
//! use uc_tauri::bootstrap::AppRuntime;
//! use tauri::State;
//!
//! #[tauri::command]
//! async fn my_command(runtime: State<'_, AppRuntime>) -> Result<(), String> {
//!     let uc = runtime.usecases().list_clipboard_entries();
//!     uc.execute(50, 0).await.map_err(|e| e.to_string())?;
//!     Ok(())
//! }
//! ```
//!
//! ## Adding New Use Cases
//!
//! 1. Ensure use case has a `new()` constructor taking its required ports
//! 2. Add a method to `UseCases` that calls `new()` with deps
//! 3. Commands can now call `runtime.usecases().your_use_case()`

use uc_app::{App, AppDeps};
use uc_core::config::AppConfig;
use uc_core::ports::ClipboardChangeHandler;
use uc_core::SystemClipboardSnapshot;

/// Application runtime with dependencies.
///
/// This struct holds all application dependencies and provides
/// access to use cases through the `usecases()` method.
///
/// ## Architecture / 架构
///
/// The `AppRuntime` serves as the central point for accessing all application
/// dependencies and use cases. It wraps `AppDeps` and provides a `usecases()`
/// method that returns a `UseCases` accessor.
///
/// `AppRuntime` 是访问所有应用依赖和用例的中心点。它包装 `AppDeps` 并提供
/// 返回 `UseCases` 访问器的 `usecases()` 方法。
///
/// ## Usage Example / 使用示例
///
/// ```rust,no_run
/// use uc_tauri::bootstrap::AppRuntime;
/// use tauri::State;
///
/// #[tauri::command]
/// async fn get_entries(runtime: State<'_, AppRuntime>) -> Result<(), String> {
///     let uc = runtime.usecases().list_clipboard_entries();
///     let entries = uc.execute(50, 0).await.map_err(|e| e.to_string())?;
///     Ok(())
/// }
/// ```
///
/// 包含所有应用依赖的运行时。
///
/// 此结构体保存所有应用依赖，并通过 `usecases()` 方法提供用例访问。
pub struct AppRuntime {
    /// Application dependencies
    pub deps: AppDeps,
}

impl AppRuntime {
    /// Create a new AppRuntime from dependencies.
    /// 从依赖创建新的 AppRuntime。
    pub fn new(deps: AppDeps) -> Self {
        Self { deps }
    }

    /// Get use cases accessor.
    /// 获取用例访问器。
    pub fn usecases(&self) -> UseCases<'_> {
        UseCases::new(self)
    }
}

/// Use cases accessor for AppRuntime.
///
/// This struct provides methods to access all use cases with their dependencies
/// pre-wired from the AppRuntime's deps.
///
/// ## Architecture / 架构
///
/// The `UseCases` accessor serves as a factory for creating use case instances.
/// Each method returns a use case with its dependencies already wired from `AppDeps`.
///
/// `UseCases` 访问器作为用例实例的工厂。每个方法返回一个用例，其依赖已从
/// `AppDeps` 连接。
///
/// ## Design Pattern / 设计模式
///
/// This implements the Factory pattern for use cases:
/// - Commands don't need to know which ports a use case needs
/// - All port-to-use-case wiring is centralized in one place
/// - Use cases remain pure (no dependency on AppDeps)
///
/// 这为用例实现了工厂模式：
/// - 命令不需要知道用例需要哪些端口
/// - 所有端口到用例的连接集中在一个地方
/// - 用例保持纯净（不依赖 AppDeps）
///
/// ## Limitations / 限制
///
/// Currently, not all use cases are accessible through this accessor due to
/// architectural constraints with trait objects. Use cases that require
/// generic type parameters cannot be instantiated with `Arc<dyn Trait>`.
///
/// 目前，由于 trait 对象的架构限制，并非所有用例都可以通过此访问器访问。
/// 需要泛型类型参数的用例无法使用 `Arc<dyn Trait>` 实例化。
///
/// AppRuntime 的用例访问器。
pub struct UseCases<'a> {
    runtime: &'a AppRuntime,
}

impl<'a> UseCases<'a> {
    /// Create a new UseCases accessor from AppRuntime.
    /// 从 AppRuntime 创建新的 UseCases 访问器。
    pub fn new(runtime: &'a AppRuntime) -> Self {
        Self { runtime }
    }

    /// Accesses the use case for querying clipboard history.
    ///
    /// # Examples
    ///
    /// ```
    /// # use uc_tauri::bootstrap::AppRuntime;
    /// # use tauri::State;
    /// # async fn example(runtime: State<'_, AppRuntime>) -> Result<(), String> {
    /// let uc = runtime.usecases().list_clipboard_entries();
    /// let entries = uc.execute(50, 0).await.map_err(|e| e.to_string())?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_clipboard_entries(&self) -> uc_app::usecases::ListClipboardEntries {
        uc_app::usecases::ListClipboardEntries::from_arc(self.runtime.deps.clipboard_entry_repo.clone())
    }

    /// Create a `DeleteClipboardEntry` use case wired with this runtime's clipboard and selection repositories.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use uc_tauri::bootstrap::AppRuntime;
    /// # use tauri::State;
    /// # use uc_core::ids::EntryId;
    /// # async fn example(runtime: State<'_, AppRuntime>, entry_id: &EntryId) -> Result<(), String> {
    /// let uc = runtime.usecases().delete_clipboard_entry();
    /// uc.execute(entry_id).await.map_err(|e| e.to_string())?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn delete_clipboard_entry(&self) -> uc_app::usecases::DeleteClipboardEntry {
        uc_app::usecases::DeleteClipboardEntry::from_ports(
            self.runtime.deps.clipboard_entry_repo.clone(),
            self.runtime.deps.selection_repo.clone(),
            self.runtime.deps.clipboard_event_repo.clone(),
        )
    }

    /// Security use cases / 安全用例
    ///
    /// Get the InitializeEncryption use case for setting up encryption.
    ///
    /// 获取 InitializeEncryption 用例以设置加密。
    ///
    /// ## Example / 示例
    ///
    /// ```rust,no_run
    /// # use uc_tauri::bootstrap::AppRuntime;
    /// # use tauri::State;
    /// # async fn example(runtime: State<'_, AppRuntime>) -> Result<(), String> {
    /// let uc = runtime.usecases().initialize_encryption();
    /// uc.execute(uc_core::security::model::Passphrase("my_pass".to_string()))
    ///     .await
    ///     .map_err(|e| e.to_string())?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn initialize_encryption(&self) -> uc_app::usecases::InitializeEncryption {
        uc_app::usecases::InitializeEncryption::from_ports(
            self.runtime.deps.encryption.clone(),
            self.runtime.deps.key_material.clone(),
            self.runtime.deps.key_scope.clone(),
            self.runtime.deps.encryption_state.clone(),
        )
    }

    /// Check if encryption is initialized
    ///
    /// ## Example / 示例
    ///
    /// ```rust,no_run
    /// # use uc_tauri::bootstrap::AppRuntime;
    /// # use tauri::State;
    /// # async fn example(runtime: State<'_, AppRuntime>) -> Result<bool, String> {
    /// let uc = runtime.usecases().is_encryption_initialized();
    /// let is_init = uc.execute().await.map_err(|e| e.to_string())?;
    /// # Ok(is_init)
    /// # }
    /// ```
    pub fn is_encryption_initialized(&self) -> uc_app::usecases::IsEncryptionInitialized {
        uc_app::usecases::IsEncryptionInitialized::new(
            self.runtime.deps.encryption_state.clone(),
        )
    }

    /// Get application settings
    ///
    /// ## Example / 示例
    ///
    /// ```rust,no_run
    /// # use uc_tauri::bootstrap::AppRuntime;
    /// # use tauri::State;
    /// # async fn example(runtime: State<'_, AppRuntime>) -> Result<uc_core::settings::model::Settings, String> {
    /// let uc = runtime.usecases().get_settings();
    /// let settings = uc.execute().await.map_err(|e| e.to_string())?;
    /// # Ok(settings)
    /// # }
    /// ```
    pub fn get_settings(&self) -> uc_app::usecases::GetSettings {
        uc_app::usecases::GetSettings::new(self.runtime.deps.settings.clone())
    }

    /// Update application settings
    ///
    /// ## Example / 示例
    ///
    /// ```rust,no_run
    /// # use uc_tauri::bootstrap::AppRuntime;
    /// # use tauri::State;
    /// # use uc_core::settings::model::Settings;
    /// # async fn example(runtime: State<'_, AppRuntime>, settings: Settings) -> Result<(), String> {
    /// let uc = runtime.usecases().update_settings();
    /// uc.execute(settings).await.map_err(|e| e.to_string())?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn update_settings(&self) -> uc_app::usecases::UpdateSettings {
        uc_app::usecases::UpdateSettings::new(self.runtime.deps.settings.clone())
    }

    // NOTE: Other use case methods will be added as the use case design evolves
    // to support trait object instantiation. Currently, use cases with generic
    // type parameters cannot be instantiated through this accessor.
    //
    // 注意：随着用例设计的演进，将添加其他用例方法以支持 trait 对象实例化。
    // 目前，具有泛型类型参数的用例无法通过此访问器实例化。
}

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

/// Implement ClipboardChangeHandler for AppRuntime.
///
/// This allows AppRuntime to be used as a callback for clipboard change events
/// from the platform layer.
#[async_trait::async_trait]
impl ClipboardChangeHandler for AppRuntime {
    async fn on_clipboard_changed(&self, snapshot: SystemClipboardSnapshot) -> anyhow::Result<()> {
        // Create CaptureClipboardUseCase with dependencies
        let usecase = uc_app::usecases::internal::capture_clipboard::CaptureClipboardUseCase::new(
            self.deps.clipboard.clone(),
            self.deps.clipboard_entry_repo.clone(),
            self.deps.clipboard_event_repo.clone(),
            self.deps.representation_policy.clone(),
            self.deps.representation_materializer.clone(),
            self.deps.device_identity.clone(),
        );

        // Execute capture with the provided snapshot
        match usecase.execute_with_snapshot(snapshot).await {
            Ok(event_id) => {
                log::debug!("Successfully captured clipboard, event_id: {}", event_id);
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to capture clipboard: {:?}", e);
                Err(e)
            }
        }
    }
}