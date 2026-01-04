//! Network port - abstracts P2P network capabilities
//!
//! This port defines the interface for network operations including
//! clipboard messaging, peer discovery, device pairing, and event subscriptions.

use async_trait::async_trait;
use anyhow::Result;
use crate::network::{ClipboardMessage, DiscoveredPeer, ConnectedPeer, NetworkEvent};

/// Network port - abstracts P2P network capabilities
///
/// This trait provides a clean abstraction over libp2p/network layer,
/// allowing use cases to interact with network functionality without
/// depending on concrete implementations.
#[async_trait]
pub trait NetworkPort: Send + Sync {
    // === Clipboard operations ===

    /// Send clipboard message to a specific peer
    async fn send_clipboard(&self, peer_id: &str, encrypted_data: Vec<u8>) -> Result<()>;

    /// Broadcast clipboard message to all connected peers
    async fn broadcast_clipboard(&self, encrypted_data: Vec<u8>) -> Result<()>;

    /// Subscribe to incoming clipboard messages
    ///
    /// Returns a receiver that will yield clipboard messages received from remote peers.
    async fn subscribe_clipboard(&self) -> Result<tokio::sync::mpsc::Receiver<ClipboardMessage>>;

    // === Peer operations ===

    /// Get all discovered peers (from mDNS)
    async fn get_discovered_peers(&self) -> Result<Vec<DiscoveredPeer>>;

    /// Get currently connected peers
    async fn get_connected_peers(&self) -> Result<Vec<ConnectedPeer>>;

    /// Get local peer ID
    fn local_peer_id(&self) -> String;

    // === Pairing operations ===

    /// Initiate pairing with a peer
    ///
    /// Returns the session ID for tracking this pairing attempt.
    async fn initiate_pairing(&self, peer_id: String, device_name: String) -> Result<String>;

    /// Send pairing PIN verification response
    async fn send_pin_response(&self, session_id: String, pin_match: bool) -> Result<()>;

    /// Send pairing rejection
    async fn send_pairing_rejection(&self, session_id: String, peer_id: String) -> Result<()>;

    /// Accept pairing request (responder side)
    async fn accept_pairing(&self, session_id: String) -> Result<()>;

    /// Unpair a device
    async fn unpair_device(&self, peer_id: String) -> Result<()>;

    // === Event operations ===

    /// Subscribe to network events
    ///
    /// Returns a receiver that will yield network events including:
    /// - Peer discovery/loss
    /// - Connection/disconnection
    /// - Pairing state changes
    /// - Clipboard send/receive confirmations
    async fn subscribe_events(&self) -> Result<tokio::sync::mpsc::Receiver<NetworkEvent>>;
}
