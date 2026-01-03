// P2P transport module with QUIC + TCP fallback
// Provides transport configuration helpers for libp2p networking

use libp2p::{
    identity::Keypair,
    noise, tcp, yamux,
};
use log::{debug, info};

/// Transport configuration module for P2P networking
///
/// This module provides helper functions for configuring libp2p transports.
/// In libp2p 0.56, transports are configured via SwarmBuilder's chainable
/// methods rather than manual transport construction.
///
/// # Architecture
/// - QUIC transport: Configured via SwarmBuilder::with_quic()
/// - TCP transport: Configured via SwarmBuilder::with_tcp()
/// - Transport combination: Handled by SwarmBuilder internally

/// Build TCP configuration for SwarmBuilder
///
/// This function creates a TCP transport configuration with:
/// - nodelay enabled (disable Nagle's algorithm for lower latency)
///
/// # Usage Example
/// ```rust,no_run
/// use libp2p::{SwarmBuilder, identity::Keypair};
/// use crate::infrastructure::p2p::transport::build_tcp_config;
///
/// # async fn example(keypair: Keypair) {
/// let swarm = SwarmBuilder::with_existing_identity(keypair)
///     .with_tokio()
///     .with_tcp(
///         build_tcp_config(),
///         noise::Config::new,
///         yamux::Config::default,
///     )
///     .unwrap()
///     .with_behaviour(|_| MyBehaviour::default())
///     .unwrap()
///     .build();
/// # }
/// ```
///
/// # Returns
/// A configured TCP transport
pub fn build_tcp_config() -> tcp::Config {
    debug!("Building TCP transport configuration with nodelay=true");
    tcp::Config::default().nodelay(true)
}

/// Build Noise authentication configuration
///
/// This function creates a Noise protocol configuration for encryption
/// and authentication. Used with SwarmBuilder::with_tcp().
///
/// # Arguments
/// * `keypair` - The libp2p keypair for authentication
///
/// # Returns
/// A Noise configuration
///
/// # Errors
/// Returns error if Noise configuration fails
pub fn build_noise_config(
    keypair: &Keypair,
) -> Result<noise::Config, noise::Error> {
    info!("Building Noise authentication configuration");
    noise::Config::new(keypair)
}

/// Build Yamux multiplexing configuration
///
/// This function creates a Yamux configuration for stream multiplexing.
/// Used with SwarmBuilder::with_tcp().
///
/// # Returns
/// A Yamux configuration
pub fn build_yamux_config() -> yamux::Config {
    debug!("Building Yamux multiplexing configuration");
    yamux::Config::default()
}

/// Build a complete TCP + Noise + Yamux transport configuration
///
/// This is a convenience function that returns all three configurations
/// needed for SwarmBuilder::with_tcp().
///
/// # Arguments
/// * `keypair` - The libp2p keypair for authentication
///
/// # Returns
/// A tuple of (tcp::Config, noise::Config, yamux::Config)
///
/// # Errors
/// Returns error if Noise configuration fails
pub fn build_transport_config(
    keypair: &Keypair,
) -> Result<(tcp::Config, noise::Config, yamux::Config), noise::Error> {
    info!("Building complete transport configuration (TCP + Noise + Yamux)");
    let tcp = build_tcp_config();
    let noise = build_noise_config(keypair)?;
    let yamux = build_yamux_config();
    debug!("Transport configuration built successfully");
    Ok((tcp, noise, yamux))
}

/// Configure QUIC transport for SwarmBuilder
///
/// This function provides QUIC configuration parameters for use with
/// SwarmBuilder::with_quic_config() method.
///
/// # QUIC Configuration
/// Note: libp2p 0.56's QUIC implementation uses default Quinn configuration
/// internally. Custom configuration should be done directly in swarm.rs
/// when calling SwarmBuilder::with_quic_config().
///
/// # Usage Example
/// ```rust,no_run
/// use libp2p::{SwarmBuilder, identity::Keypair};
///
/// # async fn example(keypair: Keypair) {
/// // Configure QUIC with custom parameters
/// let swarm = SwarmBuilder::with_existing_identity(keypair)
///     .with_tokio()
///     .with_quic_config(|config| {
///         // Apply custom QUIC configuration here
///         config
///     })
///     .with_behaviour(|_| MyBehaviour::default())
///     .unwrap()
///     .build();
/// # }
/// ```
///
/// # Arguments
/// * `config` - A function that configures the QUIC transport
///
/// # Returns
/// The identity function (pass-through for SwarmBuilder)
///
/// # Note
/// This function is currently a placeholder for future QUIC customization.
/// For now, use SwarmBuilder::with_quic() directly in swarm.rs.
pub fn configure_quic<F>(config: F) -> F
where
    F: FnOnce(libp2p::quic::Config) -> libp2p::quic::Config,
{
    info!("Configuring QUIC transport parameters");
    debug!("QUIC configuration function registered");
    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_tcp_config() {
        let tcp_config = build_tcp_config();
        // Just verify it doesn't panic - TCP config is opaque
        let _ = tcp_config;
    }

    #[test]
    fn test_build_noise_config() {
        let keypair = Keypair::generate_ed25519();
        let result = build_noise_config(&keypair);
        assert!(result.is_ok(), "Noise config build should succeed");
    }

    #[test]
    fn test_build_yamux_config() {
        let yamux_config = build_yamux_config();
        // Just verify it doesn't panic - Yamux config is opaque
        let _ = yamux_config;
    }

    #[test]
    fn test_build_transport_config() {
        let keypair = Keypair::generate_ed25519();
        let result = build_transport_config(&keypair);
        assert!(result.is_ok(), "Transport config build should succeed");
    }

    #[test]
    fn test_configure_quic() {
        // Test that configure_quic is a valid pass-through function
        use libp2p::quic::Config;

        let keypair = Keypair::generate_ed25519();
        let identity_fn = |config: Config| config;
        let configured_fn = configure_quic(identity_fn);

        // Verify the function works by creating a config and passing it through
        let base_config = Config::new(&keypair);
        let _ = configured_fn(base_config);
        // If we got here without panicking, the pass-through works
    }
}
