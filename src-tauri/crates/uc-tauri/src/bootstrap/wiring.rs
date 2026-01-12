//! # Dependency Injection / 依赖注入模块
//!
//! ## Responsibilities / 职责
//!
//! - ✅ Create infra implementations (db, fs, keyring) / 创建 infra 层具体实现
//! - ✅ Create platform implementations (clipboard, network) / 创建 platform 层具体实现
//! - ✅ Inject all dependencies into App / 将所有依赖注入到 App
//!
//! ## Prohibited / 禁止事项
//!
//! ❌ **No business logic / 禁止包含任何业务逻辑**
//! - Do not decide "what to do if encryption uninitialized"
//! - 不判断"如果加密未初始化就怎样"
//! - Do not handle "what to do if device not registered"
//! - 不处理"如果设备未注册就怎样"
//!
//! ❌ **No configuration validation / 禁止做配置验证**
//! - Config already loaded in config.rs
//! - 配置已在 config.rs 加载
//! - Validation should be in use case or upper layer
//! - 验证应在 use case 或上层
//!
//! ❌ **No direct concrete implementation usage / 禁止直接使用具体实现**
//! - Must inject through Port traits
//! - 必须通过 Port trait 注入
//! - Do not call implementation methods directly after App construction
//! - 不在 App 构造后直接调用实现方法
//!
//! ## Architecture Principle / 架构原则
//!
//! > **This is the only place allowed to depend on uc-infra + uc-platform + uc-app simultaneously.**
//! > **这是唯一允许同时依赖 uc-infra、uc-platform 和 uc-app 的地方。**
//! > But this privilege is only for "assembly", not for "decision making".
//! > 但这种特权仅用于"组装"，不用于"决策"。

use uc_app::AppDeps;
use uc_core::config::AppConfig;

/// Result type for wiring operations
pub type WiringResult<T> = Result<T, WiringError>;

/// Errors during dependency injection
/// 依赖注入错误（基础设施初始化失败）
#[derive(Debug, thiserror::Error)]
pub enum WiringError {
    #[error("Database initialization failed: {0}")]
    DatabaseInit(String),

    #[error("Keyring initialization failed: {0}")]
    KeyringInit(String),

    #[error("Clipboard initialization failed: {0}")]
    ClipboardInit(String),

    #[error("Network initialization failed: {0}")]
    NetworkInit(String),

    #[error("Blob storage initialization failed: {0}")]
    BlobStorageInit(String),

    #[error("Settings repository initialization failed: {0}")]
    SettingsInit(String),
}

/// Wire all dependencies together.
/// 将所有依赖连接在一起。
///
/// This function constructs the complete dependency graph by creating instances
/// from infrastructure and platform layers, then packaging them into `AppDeps`.
///
/// 此函数通过从基础设施层和平台层创建实例，然后将它们打包到 `AppDeps` 中来构建完整的依赖图。
///
/// # Arguments / 参数
///
/// * `config` - Application configuration / 应用配置
///
/// # Returns / 返回
///
/// * `WiringResult<AppDeps>` - The wired dependencies on success / 成功时返回已连接的依赖
///
/// # Errors / 错误
///
/// Returns `WiringError` if dependency construction fails.
/// 如果依赖构造失败，返回 `WiringError`。
///
/// # Phase 3 Implementation Plan / 第3阶段实现计划
///
/// When implementing Phase 3, this function will:
/// 实现第3阶段时，此函数将：
///
/// 1. Create infrastructure implementations (database repos, encryption, etc.)
///    创建基础设施实现（数据库仓库、加密等）
/// 2. Create platform adapters (clipboard, network, UI, etc.)
///    创建平台适配器（剪贴板、网络、UI等）
/// 3. Wrap all in `Arc<dyn Trait>` for shared ownership
///    将所有内容包装在 `Arc<dyn Trait>` 中以实现共享所有权
/// 4. Construct `AppDeps` with all dependencies
///    使用所有依赖构造 `AppDeps`
pub fn wire_dependencies(_config: &AppConfig) -> WiringResult<AppDeps> {
    // Phase 3: TODO - Implement real dependency wiring
    // 第3阶段：待办 - 实现真实的依赖注入

    // This will be implemented in subsequent tasks:
    // 这将在后续任务中实现：
    // 1. Create database repositories / 创建数据库仓库
    // 2. Create platform adapters / 创建平台适配器
    // 3. Wrap in Arc<dyn Trait> / 包装在 Arc<dyn Trait> 中
    // 4. Construct AppDeps / 构造 AppDeps

    Err(WiringError::DatabaseInit(
        "Phase 3 implementation in progress".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wiring_error_display() {
        let err = WiringError::DatabaseInit("connection failed".to_string());
        assert!(err.to_string().contains("Database initialization"));
        assert!(err.to_string().contains("connection failed"));
    }

    #[test]
    fn test_wiring_error_keyring() {
        let err = WiringError::KeyringInit("keyring unavailable".to_string());
        assert!(err.to_string().contains("Keyring initialization"));
    }

    #[test]
    fn test_wiring_error_clipboard() {
        let err = WiringError::ClipboardInit("platform error".to_string());
        assert!(err.to_string().contains("Clipboard initialization"));
    }

    #[test]
    fn test_wiring_error_network() {
        let err = WiringError::NetworkInit("bind failed".to_string());
        assert!(err.to_string().contains("Network initialization"));
    }

    #[test]
    fn test_wiring_error_blob_storage() {
        let err = WiringError::BlobStorageInit("path invalid".to_string());
        assert!(err.to_string().contains("Blob storage initialization"));
    }

    #[test]
    fn test_wiring_error_settings() {
        let err = WiringError::SettingsInit("load failed".to_string());
        assert!(err.to_string().contains("Settings repository initialization"));
    }

    #[test]
    fn test_wiring_result_success() {
        let result: WiringResult<()> = Ok(());
        assert!(result.is_ok());
    }

    #[test]
    fn test_wiring_result_error() {
        let result: WiringResult<()> = Err(WiringError::DatabaseInit("test".to_string()));
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(WiringError::DatabaseInit(_))
        ));
    }

    #[test]
    fn test_wire_dependencies_returns_not_implemented() {
        let config = AppConfig::empty();
        let result = wire_dependencies(&config);

        match result {
            Ok(_) => panic!("Expected error but got Ok - Phase 3 not yet implemented"),
            Err(WiringError::DatabaseInit(msg)) => {
                assert!(msg.contains("Phase 3"));
            }
            Err(e) => panic!("Expected DatabaseInit error, got: {}", e),
        }
    }
}
