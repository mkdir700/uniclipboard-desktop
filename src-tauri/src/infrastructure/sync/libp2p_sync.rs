//! P2P clipboard synchronization using libp2p
//!
//! This module implements the `RemoteClipboardSync` trait using libp2p for
//! peer-to-peer clipboard sharing over local networks.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Utc;
use log::{debug, error, info, warn};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::domain::transfer_message::ClipboardTransferMessage;
use crate::infrastructure::p2p::{ClipboardMessage, NetworkCommand};
use crate::infrastructure::security::encryption::Encryptor;
use crate::infrastructure::storage::peer_storage::PeerStorage;
use crate::interface::RemoteClipboardSync;

/// P2P-based clipboard synchronization
#[derive(Clone)]
pub struct Libp2pSync {
    /// Sender for network commands
    network_command_tx: tokio::sync::mpsc::Sender<NetworkCommand>,
    /// Device name for this peer
    device_name: String,
    /// Device ID (PeerId)
    device_id: String,
    /// Peer storage for retrieving shared secrets
    peer_storage: Arc<PeerStorage>,
    /// Sender for clipboard pull requests (to application)
    clipboard_tx: tokio::sync::mpsc::Sender<ClipboardTransferMessage>,
    /// Receiver for clipboard pull (used internally)
    clipboard_rx: Arc<Mutex<tokio::sync::mpsc::Receiver<ClipboardTransferMessage>>>,
    /// Whether sync is running
    running: Arc<Mutex<bool>>,
}

impl Libp2pSync {
    pub fn new(
        network_command_tx: tokio::sync::mpsc::Sender<NetworkCommand>,
        device_name: String,
        device_id: String,
        peer_storage: Arc<PeerStorage>,
    ) -> Self {
        let (clipboard_tx, clipboard_rx) = tokio::sync::mpsc::channel(100);

        Self {
            network_command_tx,
            device_name,
            device_id,
            peer_storage,
            clipboard_tx,
            clipboard_rx: Arc::new(Mutex::new(clipboard_rx)),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Get a channel sender for receiving clipboard events
    /// The NetworkManager/Swarm loop will send received messages here
    pub fn get_clipboard_tx(&self) -> tokio::sync::mpsc::Sender<ClipboardTransferMessage> {
        self.clipboard_tx.clone()
    }

    /// Access peer storage
    pub fn peer_storage(&self) -> &PeerStorage {
        &self.peer_storage
    }

    /// Compute content hash for deduplication
    fn compute_content_hash(content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        format!("{:x}", hasher.finalize())
    }

    /// Encrypt content for a specific peer
    fn encrypt_for_peer(&self, content: &[u8], shared_secret: &[u8]) -> Result<Vec<u8>> {
        let secret_bytes: [u8; 32] = shared_secret
            .try_into()
            .map_err(|_| anyhow!("Invalid shared secret length"))?;

        let encryptor = Encryptor::from_key(&secret_bytes);
        encryptor.encrypt(content)
    }

    /// Attempt to decrypt content using the sender's shared secret
    fn decrypt_content(&self, content: &[u8], origin_peer_id: &str) -> Result<Vec<u8>> {
        debug!(
            "Attempting to decrypt message from peer {}, content length: {}",
            origin_peer_id,
            content.len()
        );

        let peer = self.peer_storage.get_peer(origin_peer_id)?;

        if peer.is_none() {
            warn!("Peer {} not found in storage", origin_peer_id);
            return Err(anyhow!(
                "Could not decrypt content from {}: peer not found",
                origin_peer_id
            ));
        }

        let peer = peer.unwrap();

        if peer.shared_secret.len() != 32 {
            error!(
                "Invalid shared secret length for peer {}: expected 32, got {}",
                origin_peer_id,
                peer.shared_secret.len()
            );
            return Err(anyhow!(
                "Could not decrypt content from {}: invalid secret length",
                origin_peer_id
            ));
        }

        let secret_bytes: [u8; 32] = peer
            .shared_secret
            .as_slice()
            .try_into()
            .map_err(|_| anyhow!("Invalid shared secret format"))?;

        debug!(
            "Using shared secret for peer {} (first 4 bytes: {})",
            origin_peer_id,
            hex::encode(&secret_bytes[..4])
        );

        let encryptor = Encryptor::from_key(&secret_bytes);

        match encryptor.decrypt(content) {
            Ok(data) => {
                debug!("Successfully decrypted {} bytes", data.len());
                Ok(data)
            }
            Err(e) => {
                error!(
                    "Failed to decrypt message from peer {}: {} (content_len: {}, secret_hash: {})",
                    origin_peer_id,
                    e,
                    content.len(),
                    hex::encode(&Sha256::digest(&secret_bytes)[..8])
                );
                Err(anyhow!("Could not decrypt content from {}: {}", origin_peer_id, e))
            }
        }
    }

    /// Handle an incoming encrypted clipboard message from the network
    /// This is called when NetworkManager receives a message
    pub async fn handle_incoming_message(&self, msg: ClipboardMessage) -> Result<()> {
        let decrypted = self.decrypt_content(&msg.encrypted_content, &msg.origin_device_id)?;

        let transfer_msg: ClipboardTransferMessage = serde_json::from_slice(&decrypted)
            .map_err(|e| anyhow!("Failed to deserialize clipboard message: {}", e))?;

        self.clipboard_tx
            .send(transfer_msg)
            .await
            .map_err(|e| anyhow!("Failed to send to clipboard channel: {}", e))?;

        Ok(())
    }
}

#[async_trait]
impl RemoteClipboardSync for Libp2pSync {
    /// Push clipboard content to all paired peers
    async fn push(&self, message: ClipboardTransferMessage) -> Result<()> {
        let content_bytes = serde_json::to_vec(&message)
            .map_err(|e| anyhow!("Failed to serialize clipboard message: {}", e))?;

        let peers = self.peer_storage.get_all_peers().unwrap_or_else(|e| {
            log::error!("Failed to get peers: {}", e);
            Vec::new()
        });

        if peers.is_empty() {
            debug!("No paired peers to sync with");
            return Ok(());
        }

        for peer in &peers {
            // Encrypt for this peer
            match self.encrypt_for_peer(&content_bytes, &peer.shared_secret) {
                Ok(encrypted_content) => {
                    let content_hash = Self::compute_content_hash(&encrypted_content);

                    let clipboard_msg = ClipboardMessage {
                        id: message.message_id.clone(),
                        content_hash,
                        encrypted_content,
                        timestamp: Utc::now(),
                        origin_device_id: self.device_id.clone(),
                        origin_device_name: self.device_name.clone(),
                    };

                    // Broadcast (GossipSub will send to everyone, but only this peer can decrypt)
                    if let Err(e) = self
                        .network_command_tx
                        .send(NetworkCommand::BroadcastClipboard {
                            message: clipboard_msg,
                        })
                        .await
                    {
                        warn!("Failed to send command to network manager: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to encrypt for peer {}: {}", peer.peer_id, e);
                }
            }
        }

        debug!("Clipboard message broadcasted to {} peers", peers.len());
        Ok(())
    }

    /// Pull clipboard content from P2P network
    async fn pull(&self, _timeout: Option<Duration>) -> Result<ClipboardTransferMessage> {
        // We just wait for messages on the channel
        let mut rx = self.clipboard_rx.lock().await;
        rx.recv()
            .await
            .ok_or_else(|| anyhow!("Clipboard channel closed"))
    }

    /// Sync is a no-op for P2P (continuous sync via gossipsub)
    async fn sync(&self) -> Result<()> {
        Ok(())
    }

    /// Start P2P synchronization
    async fn start(&self) -> Result<()> {
        let mut running = self.running.lock().await;
        if *running {
            return Ok(());
        }
        *running = true;
        info!("Starting P2P clipboard synchronization");
        Ok(())
    }

    /// Stop P2P synchronization
    async fn stop(&self) -> Result<()> {
        let mut running = self.running.lock().await;
        *running = false;
        info!("Stopped P2P clipboard synchronization");
        Ok(())
    }

    /// Pause P2P synchronization
    async fn pause(&self) -> Result<()> {
        self.stop().await
    }

    /// Resume P2P synchronization
    async fn resume(&self) -> Result<()> {
        self.start().await
    }
}
