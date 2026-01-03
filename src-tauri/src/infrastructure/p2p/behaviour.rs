use libp2p::{
    identify, mdns,
    request_response::{self, ProtocolSupport},
    swarm::NetworkBehaviour,
    StreamProtocol,
};
use std::time::Duration;

use super::codec::UniClipboardCodec;

const PROTOCOL_NAME: &str = "/uniclipboard/1.0.0";

/// UniClipboard network behaviour combining libp2p protocols
///
/// This behaviour combines:
/// - mDNS: For local device discovery
/// - Request-Response: For device pairing protocol
/// - Identify: For peer identification and agent version info
///
/// Note: GossipSub has been removed in v2.0.0 as we now use BlobStream
/// for clipboard content transfer instead of pub/sub broadcasting.
#[derive(NetworkBehaviour)]
pub struct UniClipboardBehaviour {
    /// mDNS device discovery
    pub mdns: mdns::tokio::Behaviour,
    /// Request-Response protocol for device pairing
    pub request_response: request_response::Behaviour<UniClipboardCodec>,
    /// Identify protocol for peer information
    pub identify: identify::Behaviour,
}

impl UniClipboardBehaviour {
    /// Create a new UniClipboard behaviour instance
    ///
    /// # Arguments
    ///
    /// * `local_peer_id` - The local peer's libp2p PeerId
    /// * `key` - The local peer's keypair
    /// * `device_name` - Human-readable device name (e.g., "MacBook Pro")
    /// * `device_id` - 6-digit stable device ID from database
    ///
    /// # Errors
    ///
    /// Returns an error if mDNS behaviour creation fails
    pub fn new(
        local_peer_id: libp2p::PeerId,
        key: &libp2p::identity::Keypair,
        device_name: &str,
        device_id: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // mDNS for local device discovery
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?;

        // Identify for peer identification
        // Agent version format: "uniclipboard/2.0.0/<device_id>/<device_name>"
        // Updated to v2.0.0 to reflect removal of GossipSub and addition of BlobStream
        let identify = identify::Behaviour::new(
            identify::Config::new(
                format!("uniclipboard/2.0.0/{}/{}", device_id, device_name),
                key.public(),
            )
            .with_push_listen_addr_updates(true),
        );

        // Request-Response for pairing protocol
        let request_response = request_response::Behaviour::new(
            [(StreamProtocol::new(PROTOCOL_NAME), ProtocolSupport::Full)],
            request_response::Config::default()
                .with_request_timeout(Duration::from_secs(30)),
        );

        Ok(Self {
            mdns,
            request_response,
            identify,
        })
    }
}
