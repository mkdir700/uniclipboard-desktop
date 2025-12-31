//! Application runtime handle
//!
//! Provides a thread-safe, cloneable handle to the application runtime.
//! This handle is managed by Tauri and satisfies Clone + Send + Sync + 'static.

use std::sync::Arc;
use tokio::sync::mpsc;

use crate::config::Setting;

/// Commands that can be sent to the clipboard subsystem
#[derive(Debug)]
pub enum ClipboardCommand {
    /// Get clipboard items
    GetItems {
        respond_to: tokio::sync::oneshot::Sender<
            Result<
                Vec<
                    crate::infrastructure::storage::db::models::clipboard_record::DbClipboardRecord,
                >,
                String,
            >,
        >,
    },
    // More commands as needed...
}

/// Commands that can be sent to the P2P subsystem
#[derive(Debug)]
pub enum P2PCommand {
    /// Get local peer ID
    GetLocalPeerId {
        respond_to: tokio::sync::oneshot::Sender<Result<String, String>>,
    },
    /// Get discovered peers
    GetPeers {
        respond_to: tokio::sync::oneshot::Sender<
            Result<Vec<crate::infrastructure::p2p::DiscoveredPeer>, String>,
        >,
    },
    /// Initiate pairing with a peer
    InitiatePairing {
        peer_id: String,
        device_name: String,
        respond_to: tokio::sync::oneshot::Sender<Result<String, String>>,
    },
    /// Verify PIN for pairing
    VerifyPin {
        session_id: String,
        pin_matches: bool,
        respond_to: tokio::sync::oneshot::Sender<Result<(), String>>,
    },
    /// Reject pairing request
    RejectPairing {
        session_id: String,
        peer_id: String,
        respond_to: tokio::sync::oneshot::Sender<Result<(), String>>,
    },
    /// Unpair a device
    UnpairDevice {
        peer_id: String,
        respond_to: tokio::sync::oneshot::Sender<Result<(), String>>,
    },
}

/// Commands that can be sent to the Connection Manager (Legacy/WebSocket)
#[derive(Debug)]
pub enum ConnectionCommand {
    /// Manually connect to a device
    ManualConnect {
        ip: String,
        port: u16,
        respond_to: tokio::sync::oneshot::Sender<Result<String, String>>,
    },
    /// Respond to an incoming connection request
    RespondConnection {
        requester_device_id: String,
        accept: bool,
        respond_to: tokio::sync::oneshot::Sender<Result<(), String>>,
    },
    /// Cancel all pending connection requests
    CancelConnectionRequest {
        respond_to: tokio::sync::oneshot::Sender<Result<(), String>>,
    },
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
    /// Sender for P2P commands
    pub p2p_tx: mpsc::Sender<P2PCommand>,
    /// Sender for connection commands
    pub connection_tx: mpsc::Sender<ConnectionCommand>,
    /// Application configuration (immutable, shared via Arc)
    pub config: Arc<Setting>,
}

impl AppRuntimeHandle {
    /// Create a new runtime handle
    pub fn new(
        clipboard_tx: mpsc::Sender<ClipboardCommand>,
        p2p_tx: mpsc::Sender<P2PCommand>,
        connection_tx: mpsc::Sender<ConnectionCommand>,
        config: Arc<Setting>,
    ) -> Self {
        Self {
            clipboard_tx,
            p2p_tx,
            connection_tx,
            config,
        }
    }
}
