//! App runtime and handle definitions
//!
//! This module contains the runtime management for the UniClipboard application.

use std::sync::Arc;
use tokio::sync::mpsc;

use uc_core::config::AppConfig;

/// Application runtime handle
///
/// This handle is used to send commands to the background runtime.
pub struct AppRuntimeHandle {
    pub clipboard_cmd_tx: mpsc::Sender<ClipboardCommand>,
    pub p2p_cmd_tx: mpsc::Sender<P2PCommand>,
    pub config: Arc<AppConfig>,
}

impl AppRuntimeHandle {
    pub fn new(
        clipboard_cmd_tx: mpsc::Sender<ClipboardCommand>,
        p2p_cmd_tx: mpsc::Sender<P2PCommand>,
        config: Arc<AppConfig>,
    ) -> Self {
        Self {
            clipboard_cmd_tx,
            p2p_cmd_tx,
            config,
        }
    }
}

/// Clipboard commands sent to the runtime
#[derive(Debug, Clone)]
pub enum ClipboardCommand {
    StartMonitoring,
    StopMonitoring,
    WriteContent(Vec<u8>),
}

/// P2P commands sent to the runtime
#[derive(Debug, Clone)]
pub enum P2PCommand {
    Start,
    Stop,
    DiscoverPeers,
    PairDevice(String),
    UnpairDevice(String),
}
