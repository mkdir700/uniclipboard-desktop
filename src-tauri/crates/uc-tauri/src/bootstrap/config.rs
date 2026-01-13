//! # Configuration Loader / 配置加载器
//!
//! ## Responsibilities / 职责
//!
//! - ✅ Read TOML configuration files / 读取 TOML 配置文件
//! - ✅ Parse TOML into AppConfig DTO / 将 TOML 解析为 AppConfig DTO
//! - ✅ Report I/O and parsing errors with context / 报告带上下文的 I/O 和解析错误
//!
//! ## Prohibited / 禁止事项
//!
//! ❌ **No validation logic / 禁止验证逻辑**
//! ❌ **No default value logic / 禁止默认值逻辑**
//! ❌ **No business rules / 禁止业务规则**
//!
//! ## Iron Rule / 铁律
//!
//! > **Pure data loading only. Accept whatever is in the file.**
//! > **仅纯数据加载。接受文件中的任何内容。**

use anyhow::Context;
use std::path::PathBuf;
use uc_core::config::AppConfig;

/// Load configuration from a TOML file
/// 从 TOML 文件加载配置
///
/// This function performs pure data loading:
/// - Reads file content
/// - Parses TOML format
/// - Maps to AppConfig DTO
/// 此函数执行纯数据加载：
/// - 读取文件内容
/// - 解析 TOML 格式
/// - 映射为 AppConfig DTO
///
/// **NO validation is performed**:
/// - Empty strings are valid (they are facts)
/// - Invalid ports are accepted (they are facts)
/// - Missing sections result in empty values (facts)
/// **不执行任何验证**：
/// - 空字符串是合法的（它们是事实）
/// - 无效端口被接受（它们是事实）
/// - 缺失的部分导致空值（事实）
///
/// # Errors / 错误
///
/// Returns error if:
/// - File cannot be read (I/O error)
/// - Content is not valid TOML (parse error)
/// - TOML structure is malformed (mapping error)
/// 在以下情况下返回错误：
/// - 无法读取文件（I/O 错误）
/// - 内容不是有效的 TOML（解析错误）
/// - TOML 结构错误（映射错误）
pub fn load_config(config_path: PathBuf) -> anyhow::Result<AppConfig> {
    let content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;
    let toml_value: toml::Value = toml::from_str(&content)
        .context("Failed to parse config as TOML")?;
    AppConfig::from_toml(&toml_value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    /// Test that valid TOML is parsed correctly
    /// 测试有效 TOML 被正确解析
    #[test]
    fn test_load_config_reads_valid_toml() {
        let toml_content = r#"
            [general]
            device_name = "TestDevice"
            silent_start = true

            [security]
            vault_key_path = "/path/to/key"
            vault_snapshot_path = "/path/to/snapshot"

            [network]
            webserver_port = 8080

            [storage]
            database_path = "/path/to/database"
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        let config_path = temp_file.path().to_path_buf();

        let config = load_config(config_path).unwrap();

        assert_eq!(config.device_name, "TestDevice");
        assert_eq!(config.webserver_port, 8080);
        assert_eq!(config.silent_start, true);
        assert_eq!(config.vault_key_path, PathBuf::from("/path/to/key"));
        assert_eq!(config.vault_snapshot_path, PathBuf::from("/path/to/snapshot"));
        assert_eq!(config.database_path, PathBuf::from("/path/to/database"));
    }

    /// Test that missing values result in empty/default values
    /// 测试缺失的值导致空/默认值
    #[test]
    fn test_load_config_returns_empty_values_when_missing() {
        let toml_content = r#"
            [general]
            # device_name is missing

            [network]
            # webserver_port is missing

            [security]
            # vault paths are missing

            [storage]
            # database_path is missing
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        let config_path = temp_file.path().to_path_buf();

        let config = load_config(config_path).unwrap();

        // Empty values are valid "facts"
        assert_eq!(config.device_name, "");
        assert_eq!(config.webserver_port, 0);
        assert_eq!(config.vault_key_path, PathBuf::new());
        assert_eq!(config.vault_snapshot_path, PathBuf::new());
        assert_eq!(config.database_path, PathBuf::new());
        assert_eq!(config.silent_start, false);
    }

    /// Test that port validation is NOT performed
    /// 测试不执行端口验证
    #[test]
    fn test_load_config_does_not_validate_port_range() {
        // Port 99999 is out of valid range
        // We should accept it as a "fact" (it will be truncated to u16)
        let toml_content = r#"
            [network]
            webserver_port = 99999
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        let config_path = temp_file.path().to_path_buf();

        let config = load_config(config_path).unwrap();

        // We don't validate - the value is truncated (99999 as u16 = 34463)
        // This is the raw "fact" from the TOML data
        assert_eq!(config.webserver_port, 34463);
    }

    /// Test that non-existent files return IO error
    /// 测试不存在的文件返回 IO 错误
    #[test]
    fn test_load_config_returns_io_error_on_file_not_found() {
        let non_existent_path = PathBuf::from("/this/path/does/not/exist/config.toml");

        let result = load_config(non_existent_path);

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string().to_lowercase();

        // Should mention file not found or similar IO error
        assert!(
            err_msg.contains("no such file")
            || err_msg.contains("not found")
            || err_msg.contains("failed to read"),
            "Expected IO error message, got: {}", err
        );
    }
}
