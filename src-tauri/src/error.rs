//! Unified error type system for UniClipboard desktop application.
//!
//! This module provides a centralized error handling approach, replacing scattered
//! String-based error returns with a typed `AppError` enum.
//!
//! # Design Philosophy
//!
//! - **Typed errors**: Each error variant represents a specific failure scenario
//! - **Context preservation**: Errors carry relevant context for debugging
//! - **Easy conversion**: Automatic conversions from common error types (anyhow, diesel)
//! - **User-friendly**: String representations are suitable for displaying to users

use std::fmt;

/// Unified application error type.
///
/// This enum represents all possible error scenarios across the application,
/// organized by domain (Clipboard, Storage, P2P, Encryption, Config, etc.).
///
/// # Example
///
/// ```rust,no_run
/// use crate::error::AppError;
///
/// fn read_device(id: &str) -> Result<Device, AppError> {
///     if id.is_empty() {
///         return Err(AppError::Storage("Device ID cannot be empty".to_string()));
///     }
///     // ...
///     # Ok(Device::default())
/// }
/// ```
#[derive(Debug, Clone)]
pub enum AppError {
    /// Clipboard-related errors (reading, writing, format conversion)
    Clipboard(String),

    /// Storage/database errors (SQLite, Diesel, file system)
    Storage(String),

    /// P2P networking errors (libp2p, device discovery, pairing)
    P2P(String),

    /// Encryption/decryption errors (AES-GCM, key management)
    Encryption(String),

    /// Configuration errors (loading, parsing, validation)
    Config(String),

    /// I/O errors (file read/write, permissions)
    Io(String),

    /// Validation errors (invalid input, constraint violations)
    Validation(String),

    /// Generic/internal errors that don't fit other categories
    Internal(String),
}

impl AppError {
    /// Create a clipboard error with a message.
    pub fn clipboard(msg: impl Into<String>) -> Self {
        Self::Clipboard(msg.into())
    }

    /// Create a storage error with a message.
    pub fn storage(msg: impl Into<String>) -> Self {
        Self::Storage(msg.into())
    }

    /// Create a P2P error with a message.
    pub fn p2p(msg: impl Into<String>) -> Self {
        Self::P2P(msg.into())
    }

    /// Create an encryption error with a message.
    pub fn encryption(msg: impl Into<String>) -> Self {
        Self::Encryption(msg.into())
    }

    /// Create a config error with a message.
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create an I/O error with a message.
    pub fn io(msg: impl Into<String>) -> Self {
        Self::Io(msg.into())
    }

    /// Create a validation error with a message.
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Create an internal error with a message.
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    /// Get the error message as a string slice.
    pub fn message(&self) -> &str {
        match self {
            AppError::Clipboard(msg) => msg,
            AppError::Storage(msg) => msg,
            AppError::P2P(msg) => msg,
            AppError::Encryption(msg) => msg,
            AppError::Config(msg) => msg,
            AppError::Io(msg) => msg,
            AppError::Validation(msg) => msg,
            AppError::Internal(msg) => msg,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Clipboard(msg) => write!(f, "Clipboard error: {}", msg),
            AppError::Storage(msg) => write!(f, "Storage error: {}", msg),
            AppError::P2P(msg) => write!(f, "P2P error: {}", msg),
            AppError::Encryption(msg) => write!(f, "Encryption error: {}", msg),
            AppError::Config(msg) => write!(f, "Config error: {}", msg),
            AppError::Io(msg) => write!(f, "I/O error: {}", msg),
            AppError::Validation(msg) => write!(f, "Validation error: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

/// Convert from `anyhow::Error` to `AppError`.
///
/// This implementation preserves the error message and categorizes
/// anyhow errors as internal errors.
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

/// Convert from `diesel::result::Error` to `AppError`.
///
/// This implementation maps Diesel database errors to storage errors
/// with appropriate context.
impl From<diesel::result::Error> for AppError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::NotFound => {
                AppError::storage("Record not found in database")
            }
            diesel::result::Error::DatabaseError(kind, info) => {
                AppError::storage(format!("Database error: {:?}: {}", kind, info.message()))
            }
            diesel::result::Error::SerializationError(deser) => {
                AppError::storage(format!("Serialization error: {}", deser))
            }
            _ => AppError::storage(format!("Database error: {}", err)),
        }
    }
}

/// Convert from `std::io::Error` to `AppError`.
///
/// This implementation maps I/O errors to the Io variant.
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::io(err.to_string())
    }
}

/// Convert from `diesel::r2d2::PoolError` to `AppError`.
///
/// This implementation maps r2d2 pool errors to storage errors.
impl From<diesel::r2d2::PoolError> for AppError {
    fn from(err: diesel::r2d2::PoolError) -> Self {
        AppError::storage(format!("Connection pool error: {}", err))
    }
}

/// Convert from `serde_json::Error` to `AppError`.
///
/// This implementation maps JSON serialization errors to internal errors.
impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::internal(format!("JSON error: {}", err))
    }
}

/// Convert from `AppError` to `String`.
///
/// This implementation is used for Tauri command return values,
/// which require errors to be String type.
impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        err.to_string()
    }
}

/// Type alias for Result with AppError.
///
/// This simplifies function signatures throughout the application.
///
/// # Example
///
/// ```rust,no_run
/// use crate::error::{AppError, Result};
///
/// fn load_config() -> Result<Config> {
///     // Returns Result<Config, AppError>
///     # Ok(Config::default())
/// }
/// ```
pub type Result<T> = std::result::Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = AppError::clipboard("Failed to read clipboard");
        assert!(matches!(err, AppError::Clipboard(_)));
        assert_eq!(err.message(), "Failed to read clipboard");
    }

    #[test]
    fn test_error_display() {
        let err = AppError::storage("Database connection failed");
        let display = format!("{}", err);
        assert!(display.contains("Storage error"));
        assert!(display.contains("Database connection failed"));
    }

    #[test]
    fn test_from_anyhow() {
        let anyhow_err = anyhow::anyhow!("Something went wrong");
        let app_err: AppError = anyhow_err.into();
        assert!(matches!(app_err, AppError::Internal(_)));
    }

    #[test]
    fn test_from_diesel_not_found() {
        let diesel_err = diesel::result::Error::NotFound;
        let app_err: AppError = diesel_err.into();
        assert!(matches!(app_err, AppError::Storage(_)));
        assert!(app_err.message().contains("not found"));
    }

    #[test]
    fn test_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let app_err: AppError = io_err.into();
        assert!(matches!(app_err, AppError::Io(_)));
    }

    #[test]
    fn test_into_string() {
        let err = AppError::validation("Invalid input");
        let s: String = err.into();
        assert!(s.contains("Validation error"));
        assert!(s.contains("Invalid input"));
    }
}
