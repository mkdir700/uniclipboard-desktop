//! Placeholder network port implementation
//! 占位符网络端口实现

use anyhow::Result;
use async_trait::async_trait;
use uc_core::network::{ClipboardMessage, ConnectedPeer, DiscoveredPeer, NetworkEvent};
use uc_core::ports::NetworkPort;

#[async_trait]
impl NetworkPort for PlaceholderNetworkPort {
    // === Clipboard operations ===

    async fn send_clipboard(&self, _peer_id: &str, _encrypted_data: Vec<u8>) -> Result<()> {
        // TODO: Implement actual P2P clipboard sending
        // 实现实际的 P2P 剪贴板发送
        Err(anyhow::anyhow!(
            "NetworkPort::send_clipboard not implemented yet"
        ))
    }

    async fn broadcast_clipboard(&self, _encrypted_data: Vec<u8>) -> Result<()> {
        // TODO: Implement actual P2P clipboard broadcasting
        // 实现实际的 P2P 剪贴板广播
        Err(anyhow::anyhow!(
            "NetworkPort::broadcast_clipboard not implemented yet"
        ))
    }

    async fn subscribe_clipboard(&self) -> Result<tokio::sync::mpsc::Receiver<ClipboardMessage>> {
        // TODO: Implement actual clipboard event subscription
        // 实现实际的剪贴板事件订阅
        let (_tx, rx) = tokio::sync::mpsc::channel(1);
        Ok(rx)
    }

    // === Peer operations ===

    async fn get_discovered_peers(&self) -> Result<Vec<DiscoveredPeer>> {
        // TODO: Implement actual mDNS peer discovery
        // 实现实际的 mDNS 对等发现
        Ok(Vec::new())
    }

    async fn get_connected_peers(&self) -> Result<Vec<ConnectedPeer>> {
        // TODO: Implement actual connected peer list
        // 实现实际的已连接对等列表
        Ok(Vec::new())
    }

    fn local_peer_id(&self) -> String {
        // TODO: Generate actual peer ID from libp2p
        // 从 libp2p 生成实际的对等 ID
        "placeholder-peer-id".to_string()
    }

    // === Pairing operations ===

    async fn initiate_pairing(&self, _peer_id: String, _device_name: String) -> Result<String> {
        // TODO: Implement actual pairing initiation
        // 实现实际的配对启动
        Err(anyhow::anyhow!(
            "NetworkPort::initiate_pairing not implemented yet"
        ))
    }

    async fn send_pin_response(&self, _session_id: String, _pin_match: bool) -> Result<()> {
        // TODO: Implement actual PIN response
        // 实现实际的 PIN 响应
        Err(anyhow::anyhow!(
            "NetworkPort::send_pin_response not implemented yet"
        ))
    }

    async fn send_pairing_rejection(&self, _session_id: String, _peer_id: String) -> Result<()> {
        // TODO: Implement actual pairing rejection
        // 实现实际的配对拒绝
        Err(anyhow::anyhow!(
            "NetworkPort::send_pairing_rejection not implemented yet"
        ))
    }

    async fn accept_pairing(&self, _session_id: String) -> Result<()> {
        // TODO: Implement actual pairing acceptance
        // 实现实际的配对接受
        Err(anyhow::anyhow!(
            "NetworkPort::accept_pairing not implemented yet"
        ))
    }

    async fn unpair_device(&self, _peer_id: String) -> Result<()> {
        // TODO: Implement actual device unpairing
        // 实现实际的设备取消配对
        Err(anyhow::anyhow!(
            "NetworkPort::unpair_device not implemented yet"
        ))
    }

    // === Event operations ===

    async fn subscribe_events(&self) -> Result<tokio::sync::mpsc::Receiver<NetworkEvent>> {
        // TODO: Implement actual network event subscription
        // 实现实际的网络事件订阅
        let (_tx, rx) = tokio::sync::mpsc::channel(1);
        Ok(rx)
    }
}

/// Placeholder network port implementation
/// 占位符网络端口实现
#[derive(Debug, Clone)]
pub struct PlaceholderNetworkPort;
