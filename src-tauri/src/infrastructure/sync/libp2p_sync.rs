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
use crate::infrastructure::security::unified_encryptor::UnifiedEncryptor;
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
    /// Unified encryptor for encrypting/decrypting clipboard content
    encryptor: Arc<UnifiedEncryptor>,
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
        encryptor: Arc<UnifiedEncryptor>,
    ) -> Self {
        let (clipboard_tx, clipboard_rx) = tokio::sync::mpsc::channel(100);

        Self {
            network_command_tx,
            device_name,
            device_id,
            encryptor,
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

    /// Compute content hash for deduplication
    fn compute_content_hash(content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        format!("{:x}", hasher.finalize())
    }

    /// Encrypt content using the unified encryptor
    /// All paired devices share the same master key, so we encrypt once and broadcast to all
    async fn encrypt_content(&self, content: &[u8]) -> Result<Vec<u8>> {
        // Check if encryptor is ready
        if !self.encryptor.is_ready().await {
            return Err(anyhow!(
                "Unified encryptor not initialized. Please set encryption password first."
            ));
        }

        self.encryptor
            .encrypt(content)
            .await
            .map_err(|e| anyhow!("Failed to encrypt content: {}", e))
    }

    /// Decrypt content using the unified encryptor
    /// All devices use the same master key, so we don't need to know who sent it
    async fn decrypt_content(&self, content: &[u8]) -> Result<Vec<u8>> {
        // Check if encryptor is ready
        if !self.encryptor.is_ready().await {
            return Err(anyhow!(
                "Unified encryptor not initialized. Please set encryption password first."
            ));
        }

        self.encryptor
            .decrypt(content)
            .await
            .map_err(|e| anyhow!("Failed to decrypt content: {}", e))
    }

    /// Handle an incoming encrypted clipboard message from the network
    /// This is called when NetworkManager receives a message
    pub async fn handle_incoming_message(&self, msg: ClipboardMessage) -> Result<()> {
        // Decrypt using unified encryptor (no longer need origin_device_id)
        let decrypted = self
            .decrypt_content(&msg.encrypted_content)
            .await
            .map_err(|e| {
                anyhow!(
                    "Failed to decrypt clipboard message from {}: {}",
                    msg.origin_device_id,
                    e
                )
            })?;

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

        // Encrypt once using the unified encryptor (all devices share the same key)
        let encrypted_content = self.encrypt_content(&content_bytes).await.map_err(|e| {
            error!("Failed to encrypt clipboard content: {}", e);
            anyhow!("Failed to encrypt clipboard content: {}", e)
        })?;

        let content_hash = Self::compute_content_hash(&encrypted_content);

        let clipboard_msg = ClipboardMessage {
            id: message.message_id.clone(),
            content_hash,
            encrypted_content,
            timestamp: Utc::now(),
            origin_device_id: self.device_id.clone(),
            origin_device_name: self.device_name.clone(),
        };

        // Broadcast once (GossipSub delivers to all connected peers)
        if let Err(e) = self
            .network_command_tx
            .send(NetworkCommand::BroadcastClipboard {
                message: clipboard_msg,
            })
            .await
        {
            warn!("Failed to send command to network manager: {}", e);
            return Err(anyhow!("Failed to broadcast clipboard message: {}", e));
        }

        debug!("Clipboard message broadcasted successfully");
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
