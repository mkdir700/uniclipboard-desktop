//! Application runtime handle
//!
//! Provides a thread-safe, cloneable handle to the application runtime.
//! This handle is managed by Tauri and satisfies Clone + Send + Sync + 'static.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::config::Setting;

/// Commands that can be sent to the clipboard subsystem
#[derive(Debug)]
pub enum ClipboardCommand {
    /// Get clipboard items
    GetItems {
        order_by: Option<crate::infrastructure::storage::db::models::clipboard_record::OrderBy>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<crate::infrastructure::storage::db::models::clipboard_record::Filter>,
        respond_to: tokio::sync::oneshot::Sender<
            Result<Vec<crate::application::clipboard_service::ClipboardItemResponse>, String>,
        >,
    },
    // More commands as needed...
    /// Get clipboard stats
    GetStats {
        respond_to: tokio::sync::oneshot::Sender<
            Result<crate::infrastructure::storage::record_manager::ClipboardStats, String>,
        >,
    },
    /// Get single clipboard item
    GetItem {
        id: String,
        full_content: bool,
        respond_to: tokio::sync::oneshot::Sender<
            Result<Option<crate::application::clipboard_service::ClipboardItemResponse>, String>,
        >,
    },
    /// Delete clipboard item
    DeleteItem {
        id: String,
        respond_to: tokio::sync::oneshot::Sender<Result<bool, String>>,
    },
    /// Clear all clipboard items
    ClearItems {
        respond_to: tokio::sync::oneshot::Sender<Result<usize, String>>,
    },
    /// Copy clipboard item to system clipboard
    CopyItem {
        id: String,
        respond_to: tokio::sync::oneshot::Sender<Result<bool, String>>,
    },
    /// Toggle favorite status
    ToggleFavorite {
        id: String,
        is_favorited: bool,
        respond_to: tokio::sync::oneshot::Sender<Result<bool, String>>,
    },
}

/// Local device info
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalDeviceInfo {
    pub peer_id: String,
    pub device_name: String,
}

/// Paired peer with connection status
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairedPeerWithStatus {
    pub peer_id: String,
    pub device_name: String,
    pub paired_at: String,
    pub last_seen: Option<String>,
    pub last_known_addresses: Vec<String>,
    pub connected: bool,
}

/// Thread-safe handle to the application runtime
///
/// This handle satisfies Tauri's requirements for managed state:
/// - Clone (allows multiple references)
/// - Send (can be sent between threads)
/// - Sync (can be shared between threads)
/// - 'static (contains no borrowed data)
#[derive(Clone)]
pub struct AppRuntimeHandle {
    /// Sender for clipboard commands
    pub clipboard_tx: mpsc::Sender<ClipboardCommand>,
    /// Application configuration (immutable, shared via Arc)
    pub config: Arc<Setting>,
}

impl AppRuntimeHandle {
    /// Create a new runtime handle
    pub fn new(
        clipboard_tx: mpsc::Sender<ClipboardCommand>,
        config: Arc<Setting>,
    ) -> Self {
        Self {
            clipboard_tx,
            config,
        }
    }
}
