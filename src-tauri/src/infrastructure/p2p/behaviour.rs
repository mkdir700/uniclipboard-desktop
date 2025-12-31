use async_trait::async_trait;
use futures::prelude::*;
use libp2p::{
    gossipsub, identify, mdns,
    request_response::{self, Codec, ProtocolSupport},
    swarm::NetworkBehaviour,
    StreamProtocol,
};
use std::time::Duration;

use super::protocol::ProtocolMessage;

const PROTOCOL_NAME: &str = "/uniclipboard/1.0.0";
const GOSSIPSUB_TOPIC: &str = "uniclipboard-clipboard";

/// Request-response codec for pairing protocol
#[derive(Debug, Clone, Default)]
pub struct UniClipboardCodec;

/// Pairing request wrapper for request-response protocol
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PairingRequest {
    pub message: Vec<u8>,
}

/// Pairing response wrapper for request-response protocol
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PairingResponse {
    pub message: Vec<u8>,
}

#[async_trait]
impl Codec for UniClipboardCodec {
    type Protocol = StreamProtocol;
    type Request = PairingRequest;
    type Response = PairingResponse;

    async fn read_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> std::io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        let mut limited = io.take(1024 * 64); // 64KB limit
        limited.read_to_end(&mut buf).await?;
        serde_json::from_slice(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    async fn read_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> std::io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        let mut limited = io.take(1024 * 64); // 64KB limit
        limited.read_to_end(&mut buf).await?;
        serde_json::from_slice(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    async fn write_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> std::io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let bytes = serde_json::to_vec(&req)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        io.write_all(&bytes).await?;
        io.close().await?;
        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> std::io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let bytes = serde_json::to_vec(&res)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        io.write_all(&bytes).await?;
        io.close().await?;
        Ok(())
    }
}

/// UniClipboard network behaviour combining libp2p protocols
#[derive(NetworkBehaviour)]
pub struct UniClipboardBehaviour {
    pub mdns: mdns::tokio::Behaviour,
    pub gossipsub: gossipsub::Behaviour,
    pub request_response: request_response::Behaviour<UniClipboardCodec>,
    pub identify: identify::Behaviour,
}

impl UniClipboardBehaviour {
    pub fn new(
        local_peer_id: libp2p::PeerId,
        keypair: &libp2p::identity::Keypair,
        device_name: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // mDNS for local device discovery
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?;

        // Gossipsub for clipboard broadcast
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            // Use 1-second heartbeat for faster mesh building after reconnection
            // This is important for quick clipboard sync after peer restart
            .heartbeat_interval(Duration::from_secs(1))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .message_id_fn(|message| {
                // For clipboard messages: use the message's own UUID as the MessageId
                // This makes each broadcast unique, allowing resending same content.
                // For other messages: use content hash for deduplication.
                if let Ok(ProtocolMessage::Clipboard(clipboard_msg)) =
                    ProtocolMessage::from_bytes(&message.data)
                {
                    return gossipsub::MessageId::from(clipboard_msg.id);
                }
                // Fallback: content-based hash for non-clipboard messages
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                std::hash::Hash::hash(&message.data, &mut hasher);
                gossipsub::MessageId::from(std::hash::Hasher::finish(&hasher).to_string())
            })
            .build()
            .map_err(|e| format!("Failed to create gossipsub config: {}", e))?;

        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(keypair.clone()),
            gossipsub_config,
        )
        .map_err(|e| format!("Failed to create gossipsub behaviour: {}", e))?;

        // Request-response for pairing protocol
        let request_response = request_response::Behaviour::new(
            [(StreamProtocol::new(PROTOCOL_NAME), ProtocolSupport::Full)],
            request_response::Config::default(),
        );

        // Identify for peer identification
        // Format: "uniclipboard/<version>/<device_name>"
        // The device name is included so other peers can display a human-readable name
        let identify = identify::Behaviour::new(
            identify::Config::new(PROTOCOL_NAME.to_string(), keypair.public()).with_agent_version(
                format!("uniclipboard/1.0.0/{}", device_name),
            ),
        );

        Ok(Self {
            mdns,
            gossipsub,
            request_response,
            identify,
        })
    }

    /// Subscribe to clipboard broadcast topic
    pub fn subscribe_clipboard(&mut self) -> Result<(), gossipsub::SubscriptionError> {
        let topic = gossipsub::IdentTopic::new(GOSSIPSUB_TOPIC);
        self.gossipsub.subscribe(&topic).map(|_| ())
    }

    /// Publish a message to the clipboard topic
    pub fn publish_clipboard(
        &mut self,
        message: &ProtocolMessage,
    ) -> Result<gossipsub::MessageId, gossipsub::PublishError> {
        let topic = gossipsub::IdentTopic::new(GOSSIPSUB_TOPIC);
        let data = message
            .to_bytes()
            .map_err(|e| gossipsub::PublishError::TransformFailed(std::io::Error::other(e)))?;
        self.gossipsub.publish(topic, data)
    }
}
