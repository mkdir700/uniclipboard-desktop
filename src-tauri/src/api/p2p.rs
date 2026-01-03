//! P2P device discovery and pairing commands
//!
//! 提供基于 libp2p 的设备发现、配对和剪贴板同步功能

use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::State;

use crate::domain::pairing::PairedPeer;
use crate::infrastructure::p2p::DiscoveredPeer;
use crate::infrastructure::runtime::LocalDeviceInfo;
use crate::services::p2p::{PairedPeerWithStatus, P2PService};

/// Type alias for the P2PService state (using Arc<P2PService> since P2PService is not Clone)
type P2PServiceState = Arc<Mutex<Option<Arc<P2PService>>>>;

/// Helper function to get P2PService from state
fn get_p2p_service(state: &P2PServiceState) -> Result<Arc<P2PService>, String> {
    let guard = state.lock().map_err(|e| format!("Failed to acquire lock: {}", e))?;
    guard
        .as_ref()
        .cloned()
        .ok_or_else(|| "P2PService not initialized".to_string())
}

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
pub async fn get_local_peer_id(
    service: State<'_, P2PServiceState>,
) -> Result<String, String> {
    let service = get_p2p_service(&service)?;
    Ok(service.local_peer_id().to_string())
}

/// 获取发现的 P2P 设备列表
#[tauri::command]
pub async fn get_p2p_peers(
    service: State<'_, P2PServiceState>,
) -> Result<Vec<P2PPeerInfo>, String> {
    let service = get_p2p_service(&service)?;
    let peers = service.discovered_peers().await;

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
    service: State<'_, P2PServiceState>,
) -> Result<P2PPairingResponse, String> {
    info!("Initiating P2P pairing with peer: {}", request.peer_id);

    let service = get_p2p_service(&service)?;
    let session_id = service
        .initiate_pairing(request.peer_id)
        .await
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
    service: State<'_, P2PServiceState>,
) -> Result<(), String> {
    info!(
        "Verifying PIN for session: {}, matches: {}",
        request.session_id, request.pin_matches
    );

    let service = get_p2p_service(&service)?;
    service
        .verify_pin(&request.session_id, request.pin_matches)
        .await
        .map_err(|e| e.to_string())
}

/// 拒绝 P2P 配对请求
#[tauri::command]
pub async fn reject_p2p_pairing(
    session_id: String,
    peer_id: String,
    service: State<'_, P2PServiceState>,
) -> Result<(), String> {
    info!(
        "Rejecting P2P pairing: session={}, peer={}",
        session_id, peer_id
    );

    let service = get_p2p_service(&service)?;
    service
        .reject_pairing(&session_id, peer_id)
        .await
        .map_err(|e| e.to_string())
}

/// 取消 P2P 配对连接
#[tauri::command]
pub async fn unpair_p2p_device(
    peer_id: String,
    service: State<'_, P2PServiceState>,
) -> Result<(), String> {
    info!("Unpairing P2P device: {}", peer_id);

    let service = get_p2p_service(&service)?;
    service
        .unpair_device(&peer_id)
        .await
        .map_err(|e| e.to_string())
}

/// 接受 P2P 配对请求（接收方）
#[tauri::command]
pub async fn accept_p2p_pairing(
    session_id: String,
    service: State<'_, P2PServiceState>,
) -> Result<(), String> {
    info!("Accepting P2P pairing: session={}", session_id);

    let service = get_p2p_service(&service)?;
    service
        .accept_pairing(&session_id)
        .await
        .map_err(|e| e.to_string())
}

/// 获取本地设备信息
#[tauri::command]
pub async fn get_local_device_info(
    service: State<'_, P2PServiceState>,
) -> Result<LocalDeviceInfo, String> {
    info!("Getting local device info");

    let service = get_p2p_service(&service)?;
    let device_info = service.local_device_info();
    Ok(LocalDeviceInfo {
        peer_id: device_info.peer_id,
        device_name: device_info.device_name,
    })
}

/// 获取已配对的设备列表
#[tauri::command]
pub async fn get_paired_peers(
    service: State<'_, P2PServiceState>,
) -> Result<Vec<PairedPeer>, String> {
    info!("Getting paired peers");

    let service = get_p2p_service(&service)?;
    service
        .paired_peers()
        .await
        .map_err(|e| e.to_string())
}

/// 获取已配对的设备列表（带连接状态）
#[tauri::command]
pub async fn get_paired_peers_with_status(
    service: State<'_, P2PServiceState>,
) -> Result<Vec<PairedPeerWithStatus>, String> {
    info!("Getting paired peers with connection status");

    let service = get_p2p_service(&service)?;
    service
        .paired_peers_with_status()
        .await
        .map_err(|e| e.to_string())
}
