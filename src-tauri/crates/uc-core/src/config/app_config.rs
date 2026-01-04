//! Application configuration domain model

use serde::{Deserialize, Serialize};

/// Application configuration
///
/// This is a simplified version of the full Setting struct,
/// containing only the configuration needed by the application layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Device name for this instance
    pub device_name: String,

    /// Synchronization settings
    pub sync: SyncConfig,

    /// Storage settings
    pub storage: StorageConfig,
}

/// Synchronization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Whether to automatically write received clipboard content to local clipboard
    pub auto_write_to_clipboard: bool,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Maximum number of clipboard history items to keep
    pub max_history_items: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            device_name: "My Device".to_string(),
            sync: SyncConfig {
                auto_write_to_clipboard: true,
            },
            storage: StorageConfig {
                max_history_items: 1000,
            },
        }
    }
}
