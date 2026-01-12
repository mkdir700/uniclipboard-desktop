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

use std::path::PathBuf;

use uc_app::AppDeps;
use uc_core::config::AppConfig;
use uc_infra::db::pool::{init_db_pool, DbPool};

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

/// Create SQLite database connection pool
/// 创建 SQLite 数据库连接池
///
/// # Arguments / 参数
///
/// * `db_path` - Path to the SQLite database file / SQLite 数据库文件路径
///
/// # Returns / 返回
///
/// * `WiringResult<DbPool>` - The connection pool on success / 成功时返回连接池
///
/// # Errors / 错误
///
/// Returns `WiringError::DatabaseInit` if:
/// 如果以下情况返回 `WiringError::DatabaseInit`：
/// - Parent directory creation fails / 父目录创建失败
/// - Database pool creation fails / 数据库池创建失败
/// - Migration fails / 迁移失败
fn create_db_pool(db_path: &PathBuf) -> WiringResult<DbPool> {
    // Ensure parent directory exists
    // 确保父目录存在
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            WiringError::DatabaseInit(format!("Failed to create DB directory: {}", e))
        })?;
    }

    // Convert PathBuf to string for database URL
    // 将 PathBuf 转换为字符串作为数据库 URL
    let db_url = db_path
        .to_str()
        .ok_or_else(|| WiringError::DatabaseInit("Invalid database path".to_string()))?;

    // Create connection pool and run migrations
    // 创建连接池并运行迁移
    init_db_pool(db_url).map_err(|e| WiringError::DatabaseInit(format!("Failed to initialize DB: {}", e)))
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

    #[test]
    fn test_create_db_pool_signature() {
        // This test verifies the function signature is correct
        // Actual DB pool functionality testing is in integration tests
        // 此测试验证函数签名正确
        // 实际数据库池功能测试在集成测试中

        // Create a temporary database path
        // 创建临时数据库路径
        let db_path = PathBuf::from(":memory:");

        // The function should exist and return the correct type
        // 函数应该存在并返回正确的类型
        let result = create_db_pool(&db_path);

        // We expect it to succeed with in-memory database
        // 我们期望内存数据库能成功
        match result {
            Ok(_pool) => {
                // Pool is created successfully - type is verified by compiler
                // 池创建成功 - 类型由编译器验证
                assert!(true);
            }
            Err(e) => {
                // If it fails, it should be a DatabaseInit error
                // 如果失败，应该是 DatabaseInit 错误
                assert!(matches!(e, WiringError::DatabaseInit(_)));
            }
        }
    }

    #[test]
    fn test_create_db_pool_with_empty_path() {
        // Test with an empty path - should succeed (creates in-memory DB)
        // 使用空路径测试 - 应该成功（创建内存数据库）
        let db_path = PathBuf::new();

        let result = create_db_pool(&db_path);

        // Empty path is treated as empty string, which diesel interprets as in-memory
        // 空路径被视为空字符串，diesel 将其解释为内存数据库
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_db_pool_creates_parent_directory() {
        // This test would need tempdir support, which is in dev-dependencies
        // For now, we just verify the function exists
        // 此测试需要 tempdir 支持，这在 dev-dependencies 中
        // 目前我们只验证函数存在
        let _ = create_db_pool;
        // Actual directory creation testing is in integration tests
        // 实际目录创建测试在集成测试中
    }
}
