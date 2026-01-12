//! # Bootstrap Module Integration Tests / Bootstrap 模块集成测试
//!
//! These tests verify that the bootstrap module correctly:
//! 这些测试验证 bootstrap 模块正确地：
//!
//! 1. Loads configuration from TOML files / 从 TOML 文件加载配置
//! 2. Creates the dependency injection structure / 创建依赖注入结构
//! 3. Maintains separation between config and business logic / 保持配置和业务逻辑的分离
//!
//! ## Test Philosophy / 测试理念
//!
//! **Pure data behavior only / 仅纯数据行为**:
//! - Config loader accepts whatever is in the file / 配置加载器接受文件中的任何内容
//! - No validation logic / 无验证逻辑
//! - No default value logic / 无默认值逻辑
//! - Paths are loaded as-is (no existence checks) / 路径按原样加载（不检查存在性）
//!
//! ## Phase 2 Status / 第2阶段状态
//!
//! In Phase 2, `wire_dependencies` is a skeleton that returns an error.
//! 在第2阶段，`wire_dependencies` 是一个返回错误的骨架。
//! Phase 3 will add real implementations.
//! 第3阶段将添加真实实现。

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;
use uc_tauri::bootstrap::{load_config, wire_dependencies};
use uc_core::config::AppConfig;

/// Test 1: Full integration test for config loading
/// 测试1：配置加载的完整集成测试
///
/// This test verifies that:
/// 此测试验证：
/// - A complete TOML file is parsed correctly / 完整的 TOML 文件被正确解析
/// - All fields are loaded into AppConfig / 所有字段被加载到 AppConfig
/// - The integration between file I/O and parsing works / 文件 I/O 和解析之间的集成工作
#[test]
fn test_bootstrap_load_config_integration() {
    // Create a temporary directory for test isolation
    // 为测试隔离创建临时目录
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.toml");

    // Write a complete TOML configuration
    // 写入完整的 TOML 配置
    let toml_content = r#"
        [general]
        device_name = "TestDevice"
        silent_start = true

        [security]
        vault_key_path = "/tmp/test/key"
        vault_snapshot_path = "/tmp/test/snapshot"

        [network]
        webserver_port = 8080

        [storage]
        database_path = "/tmp/test/database.db"
    "#;

    let mut file = fs::File::create(&config_path).unwrap();
    file.write_all(toml_content.as_bytes()).unwrap();

    // Load config and verify all fields
    // 加载配置并验证所有字段
    let config = load_config(config_path).unwrap();

    assert_eq!(config.device_name, "TestDevice");
    assert_eq!(config.webserver_port, 8080);
    assert_eq!(config.silent_start, true);
    assert_eq!(config.vault_key_path, PathBuf::from("/tmp/test/key"));
    assert_eq!(config.vault_snapshot_path, PathBuf::from("/tmp/test/snapshot"));
    assert_eq!(config.database_path, PathBuf::from("/tmp/test/database.db"));

    // TempDir is automatically cleaned up when dropped
    // TempDir 在 drop 时自动清理
}

/// Test 2: Empty values are valid facts
/// 测试2：空值是合法的事实
///
/// This test verifies the "no validation" principle:
/// 此测试验证"无验证"原则：
/// - Empty strings are accepted / 空字符串被接受
/// - Empty paths are accepted / 空路径被接受
/// - Missing sections result in empty values / 缺失的部分导致空值
#[test]
fn test_bootstrap_config_empty_values_are_valid_facts() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("empty_config.toml");

    // Write a TOML with all sections missing
    // 写入所有部分都缺失的 TOML
    let toml_content = r#"
        [general]
        # All fields missing

        [network]
        # Port missing

        [security]
        # Paths missing

        [storage]
        # Database path missing
    "#;

    let mut file = fs::File::create(&config_path).unwrap();
    file.write_all(toml_content.as_bytes()).unwrap();

    let config = load_config(config_path).unwrap();

    // All empty values are valid "facts" - no validation
    // 所有空值都是合法的"事实" - 无验证
    assert_eq!(config.device_name, "");
    assert_eq!(config.webserver_port, 0);
    assert_eq!(config.vault_key_path, PathBuf::new());
    assert_eq!(config.vault_snapshot_path, PathBuf::new());
    assert_eq!(config.database_path, PathBuf::new());
    assert_eq!(config.silent_start, false);
}

/// Test 3: Paths are loaded as-is (no state checks)
/// 测试3：路径按原样加载（无状态检查）
///
/// This test verifies that:
/// 此测试验证：
/// - Paths don't need to exist / 路径不需要存在
/// - Paths can be absolute or relative / 路径可以是绝对或相对的
/// - No filesystem checks are performed / 不执行文件系统检查
#[test]
fn test_bootstrap_config_path_info_only_no_state_check() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("path_config.toml");

    // Use paths that definitely don't exist
    // 使用肯定不存在的路径
    let toml_content = r#"
        [general]
        device_name = "NonExistentDevice"

        [security]
        vault_key_path = "/nonexistent/path/to/secret/key.dat"
        vault_snapshot_path = "/another/nonexistent/path/snapshot.bin"

        [storage]
        database_path = "/tmp/this/does/not/exist/database.db"

        [network]
        webserver_port = 9999
    "#;

    let mut file = fs::File::create(&config_path).unwrap();
    file.write_all(toml_content.as_bytes()).unwrap();

    let config = load_config(config_path).unwrap();

    // Paths are loaded as-is, no existence checks
    // 路径按原样加载，无存在性检查
    assert_eq!(config.vault_key_path, PathBuf::from("/nonexistent/path/to/secret/key.dat"));
    assert_eq!(config.vault_snapshot_path, PathBuf::from("/another/nonexistent/path/snapshot.bin"));
    assert_eq!(config.database_path, PathBuf::from("/tmp/this/does/not/exist/database.db"));

    // Verify the files DON'T actually exist (prove no state check happened)
    // 验证文件实际上不存在（证明没有执行状态检查）
    assert!(!config.vault_key_path.exists());
    assert!(!config.vault_snapshot_path.exists());
    assert!(!config.database_path.exists());
}

/// Test 4: Invalid values are accepted (no validation)
/// 测试4：无效值被接受（无验证）
///
/// This test verifies that:
/// 此测试验证：
/// - Ports outside valid range are accepted / 有效范围外的端口被接受
/// - No business rules are enforced / 不执行业务规则
/// - Values are taken as "facts" from the file / 值作为来自文件的"事实"
#[test]
fn test_bootstrap_config_invalid_port_is_accepted() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid_port.toml");

    // Port 99999 is way outside the valid u16 range (0-65535)
    // 端口 99999 远超有效 u16 范围（0-65535）
    // When parsed as u16, it will be truncated/overflow
    // 解析为 u16 时，它将被截断/溢出
    let toml_content = r#"
        [network]
        webserver_port = 99999
    "#;

    let mut file = fs::File::create(&config_path).unwrap();
    file.write_all(toml_content.as_bytes()).unwrap();

    let config = load_config(config_path).unwrap();

    // We don't validate - the value is what TOML gives us
    // 我们不验证 - 值就是 TOML 给我们的
    // 99999 as u16 = 34463 (due to overflow/truncation)
    // 99999 作为 u16 = 34463（由于溢出/截断）
    assert_eq!(config.webserver_port, 34463);

    // Also test port 0 (technically invalid but accepted as a fact)
    // 也测试端口 0（技术上无效但作为事实接受）
    let config_path2 = temp_dir.path().join("zero_port.toml");
    let toml_content2 = r#"
        [network]
        webserver_port = 0
    "#;

    let mut file2 = fs::File::create(&config_path2).unwrap();
    file2.write_all(toml_content2.as_bytes()).unwrap();

    let config2 = load_config(config_path2).unwrap();
    assert_eq!(config2.webserver_port, 0);
}

/// Test 5: wire_dependencies returns expected Phase 3 error
/// 测试5：wire_dependencies 返回预期的第3阶段错误
///
/// This test verifies that:
/// 此测试验证：
/// - The skeleton implementation returns the correct error / 骨架实现返回正确错误
/// - The error message indicates Phase 3 implementation is needed / 错误消息表明需要第3阶段实现
/// - The function signature is correct for future implementation / 函数签名对未来实现是正确的
#[test]
fn test_bootstrap_wire_dependencies_not_yet_implemented() {
    let config = AppConfig::empty();

    let result = wire_dependencies(&config);

    // Should return an error, not success
    // 应该返回错误，而不是成功
    match result {
        Ok(_) => panic!("wire_dependencies should return error in Phase 2, but got Ok"),
        Err(error) => {
            let error_msg = error.to_string().to_lowercase();

            // Error message should mention Phase 3
            // 错误消息应该提到第3阶段
            assert!(
                error_msg.contains("phase 3") || error_msg.contains("phase3"),
                "Error message should mention Phase 3, got: {}",
                error
            );

            // Error message should mention "not yet implemented" or similar
            // 错误消息应该提到"尚未实现"或类似内容
            assert!(
                error_msg.contains("not yet implemented")
                    || error_msg.contains("not implemented")
                    || error_msg.contains("pending"),
                "Error message should indicate implementation is pending, got: {}",
                error
            );
        }
    }
}

/// Test 6: Integration test - real file I/O error handling
/// 测试6：集成测试 - 真实文件 I/O 错误处理
///
/// This test verifies that:
/// 此测试验证：
/// - File not found errors are properly reported / 文件未找到错误被正确报告
/// - Error messages include context / 错误消息包含上下文
/// - I/O errors don't cause panics / I/O 错误不会导致 panic
#[test]
fn test_bootstrap_load_config_handles_io_errors() {
    // Use a path that definitely doesn't exist
    // 使用肯定不存在的路径
    let non_existent_path = "/tmp/uniclipboard_test_this_path_does_not_exist_12345.toml";

    let result = load_config(non_existent_path.into());

    assert!(result.is_err(), "Should return error for non-existent file");

    let error = result.unwrap_err();
    let error_msg = error.to_string().to_lowercase();

    // Error should mention the file or reading failure
    // 错误应该提到文件或读取失败
    assert!(
        error_msg.contains("failed to read")
            || error_msg.contains("no such file")
            || error_msg.contains("not found")
            || error_msg.contains("config"),
        "Error should mention file I/O failure, got: {}",
        error
    );
}

/// Test 7: Integration test - malformed TOML handling
/// 测试7：集成测试 - 格式错误的 TOML 处理
///
/// This test verifies that:
/// 此测试验证：
/// - Invalid TOML syntax is caught / 无效的 TOML 语法被捕获
/// - Parse errors are properly reported / 解析错误被正确报告
/// - Error messages are context-rich / 错误消息包含丰富上下文
#[test]
fn test_bootstrap_load_config_handles_malformed_toml() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("malformed.toml");

    // Write invalid TOML (missing closing bracket)
    // 写入无效的 TOML（缺少闭合括号）
    let malformed_toml = r#"
        [general
        device_name = "Test"
        # Missing closing bracket above
    "#;

    let mut file = fs::File::create(&config_path).unwrap();
    file.write_all(malformed_toml.as_bytes()).unwrap();

    let result = load_config(config_path);

    assert!(result.is_err(), "Should return error for malformed TOML");

    let error = result.unwrap_err();
    let error_msg = error.to_string().to_lowercase();

    // Error should mention TOML parsing
    // 错误应该提到 TOML 解析
    assert!(
        error_msg.contains("toml") || error_msg.contains("parse"),
        "Error should mention TOML parsing failure, got: {}",
        error
    );
}

/// Test 8: Edge case - completely empty file
/// 测试8：边界情况 - 完全空文件
///
/// This test verifies that:
/// 此测试验证：
/// - An empty TOML file is handled gracefully / 空的 TOML 文件被优雅处理
/// - Results in AppConfig with all empty values / 导致所有字段为空的 AppConfig
/// - No crashes or panics / 无崩溃或 panic
#[test]
fn test_bootstrap_load_config_handles_empty_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("empty.toml");

    // Write completely empty file
    // 写入完全空的文件
    let mut file = fs::File::create(&config_path).unwrap();
    file.write_all(b"").unwrap();

    let config = load_config(config_path).unwrap();

    // Should get empty config (all defaults/empty values)
    // 应该得到空配置（所有默认/空值）
    assert_eq!(config.device_name, "");
    assert_eq!(config.webserver_port, 0);
    assert_eq!(config.vault_key_path, PathBuf::new());
    assert_eq!(config.vault_snapshot_path, PathBuf::new());
    assert_eq!(config.database_path, PathBuf::new());
    assert_eq!(config.silent_start, false);
}
