//! P2P device discovery and pairing commands
//!
//! 提供基于 libp2p 的设备发现、配对和剪贴板同步功能

use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::domain::pairing::PairedPeer;
use crate::infrastructure::p2p::DiscoveredPeer;
use crate::infrastructure::runtime::{AppRuntimeHandle, LocalDeviceInfo, PairedPeerWithStatus, P2PCommand};

/// P2P 设备信息
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct P2PPeerInfo {
    /// Peer ID (libp2p identifier)
    pub peer_id: String,
    /// Device name
    pub device_name: Option<String>,
    /// Addresses
    pub addresses: Vec<String>,
    /// Whether this peer is paired
    pub is_paired: bool,
    /// Connection status
    pub connected: bool,
}

/// P2P 配对请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct P2PPairingRequest {
    /// Target peer ID
    pub peer_id: String,
}

/// P2P 配对响应
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct P2PPairingResponse {
    /// Session ID for this pairing attempt
    pub session_id: String,
    /// Whether pairing was initiated successfully
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// P2P PIN 验证请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct P2PPinVerifyRequest {
    /// Session ID
    pub session_id: String,
    /// Whether PIN matches
    pub pin_matches: bool,
}

/// 获取本地 Peer ID
#[tauri::command]
pub async fn get_local_peer_id(app_handle: State<'_, AppRuntimeHandle>) -> Result<String, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();

    app_handle
        .p2p_tx
        .send(P2PCommand::GetLocalPeerId { respond_to: tx })
        .await
        .map_err(|e| format!("Failed to send command: {}", e))?;

    rx.await
        .map_err(|e| format!("Failed to receive response: {}", e))?
        .map_err(|e| e.to_string())
}

/// 获取发现的 P2P 设备列表
#[tauri::command]
pub async fn get_p2p_peers(
    app_handle: State<'_, AppRuntimeHandle>,
) -> Result<Vec<P2PPeerInfo>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();

    app_handle
        .p2p_tx
        .send(P2PCommand::GetPeers { respond_to: tx })
        .await
        .map_err(|e| format!("Failed to send command: {}", e))?;

    let peers = rx
        .await
        .map_err(|e| format!("Failed to receive response: {}", e))?
        .map_err(|e| e.to_string())?;

    // Convert DiscoveredPeer to P2PPeerInfo
    Ok(peers
        .into_iter()
        .map(|p| P2PPeerInfo {
            peer_id: p.peer_id,
            device_name: p.device_name,
            addresses: p.addresses,
            is_paired: p.is_paired,
            connected: false, // TODO: implement connection state tracking
        })
        .collect())
}

/// 发起 P2P 配对请求
#[tauri::command]
pub async fn initiate_p2p_pairing(
    request: P2PPairingRequest,
    app_handle: State<'_, AppRuntimeHandle>,
) -> Result<P2PPairingResponse, String> {
    info!("Initiating P2P pairing with peer: {}", request.peer_id);

    let (tx, rx) = tokio::sync::oneshot::channel();

    // Get device name from hostname
    let device_name = gethostname::gethostname()
        .to_str()
        .unwrap_or("Unknown Device")
        .to_string();

    app_handle
        .p2p_tx
        .send(P2PCommand::InitiatePairing {
            peer_id: request.peer_id,
            device_name,
            respond_to: tx,
        })
        .await
        .map_err(|e| format!("Failed to send command: {}", e))?;

    let session_id = rx
        .await
        .map_err(|e| format!("Failed to receive response: {}", e))?
        .map_err(|e| e.to_string())?;

    Ok(P2PPairingResponse {
        session_id,
        success: true,
        error: None,
    })
}

/// 验证 PIN 并完成配对
#[tauri::command]
pub async fn verify_p2p_pairing_pin(
    request: P2PPinVerifyRequest,
    app_handle: State<'_, AppRuntimeHandle>,
) -> Result<(), String> {
    info!(
        "Verifying PIN for session: {}, matches: {}",
        request.session_id, request.pin_matches
    );

    let (tx, rx) = tokio::sync::oneshot::channel();

    app_handle
        .p2p_tx
        .send(P2PCommand::VerifyPin {
            session_id: request.session_id,
            pin_matches: request.pin_matches,
            respond_to: tx,
        })
        .await
        .map_err(|e| format!("Failed to send command: {}", e))?;

    rx.await
        .map_err(|e| format!("Failed to receive response: {}", e))?
        .map_err(|e| e.to_string())
}

/// 拒绝 P2P 配对请求
#[tauri::command]
pub async fn reject_p2p_pairing(
    session_id: String,
    peer_id: String,
    app_handle: State<'_, AppRuntimeHandle>,
) -> Result<(), String> {
    info!(
        "Rejecting P2P pairing: session={}, peer={}",
        session_id, peer_id
    );

    let (tx, rx) = tokio::sync::oneshot::channel();

    app_handle
        .p2p_tx
        .send(P2PCommand::RejectPairing {
            session_id,
            peer_id,
            respond_to: tx,
        })
        .await
        .map_err(|e| format!("Failed to send command: {}", e))?;

    rx.await
        .map_err(|e| format!("Failed to receive response: {}", e))?
        .map_err(|e| e.to_string())
}

/// 取消 P2P 配对连接
#[tauri::command]
pub async fn unpair_p2p_device(
    peer_id: String,
    app_handle: State<'_, AppRuntimeHandle>,
) -> Result<(), String> {
    info!("Unpairing P2P device: {}", peer_id);

    let (tx, rx) = tokio::sync::oneshot::channel();

    app_handle
        .p2p_tx
        .send(P2PCommand::UnpairDevice {
            peer_id,
            respond_to: tx,
        })
        .await
        .map_err(|e| format!("Failed to send command: {}", e))?;

    rx.await
        .map_err(|e| format!("Failed to receive response: {}", e))?
}

/// 接受 P2P 配对请求（接收方）
#[tauri::command]
pub async fn accept_p2p_pairing(
    session_id: String,
    app_handle: State<'_, AppRuntimeHandle>,
) -> Result<(), String> {
    info!("Accepting P2P pairing: session={}", session_id);

    let (tx, rx) = tokio::sync::oneshot::channel();

    app_handle
        .p2p_tx
        .send(P2PCommand::AcceptPairing {
            session_id,
            respond_to: tx,
        })
        .await
        .map_err(|e| format!("Failed to send command: {}", e))?;

    rx.await
        .map_err(|e| format!("Failed to receive response: {}", e))?
}

/// 获取本地设备信息
#[tauri::command]
pub async fn get_local_device_info(
    app_handle: State<'_, AppRuntimeHandle>,
) -> Result<LocalDeviceInfo, String> {
    info!("Getting local device info");

    let (tx, rx) = tokio::sync::oneshot::channel();

    app_handle
        .p2p_tx
        .send(P2PCommand::GetLocalDeviceInfo { respond_to: tx })
        .await
        .map_err(|e| format!("Failed to send command: {}", e))?;

    rx.await
        .map_err(|e| format!("Failed to receive response: {}", e))?
        .map_err(|e| e.to_string())
}

/// 获取已配对的设备列表
#[tauri::command]
pub async fn get_paired_peers(
    app_handle: State<'_, AppRuntimeHandle>,
) -> Result<Vec<PairedPeer>, String> {
    info!("Getting paired peers");

    let (tx, rx) = tokio::sync::oneshot::channel();

    app_handle
        .p2p_tx
        .send(P2PCommand::GetPairedPeers { respond_to: tx })
        .await
        .map_err(|e| format!("Failed to send command: {}", e))?;

    rx.await
        .map_err(|e| format!("Failed to receive response: {}", e))?
        .map_err(|e| e.to_string())
}

/// 获取已配对的设备列表（带连接状态）
#[tauri::command]
pub async fn get_paired_peers_with_status(
    app_handle: State<'_, AppRuntimeHandle>,
) -> Result<Vec<PairedPeerWithStatus>, String> {
    info!("Getting paired peers with connection status");

    let (tx, rx) = tokio::sync::oneshot::channel();

    app_handle
        .p2p_tx
        .send(P2PCommand::GetPairedPeersWithStatus { respond_to: tx })
        .await
        .map_err(|e| format!("Failed to send command: {}", e))?;

    rx.await
        .map_err(|e| format!("Failed to receive response: {}", e))?
        .map_err(|e| e.to_string())
}
