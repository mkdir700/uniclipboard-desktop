// P2P transport module with QUIC + TCP fallback
// Provides transport configuration helpers for libp2p networking

use libp2p::{identity::Keypair, noise, quic, tcp, yamux};
use log::info;
use std::time::Duration;

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
    info!("Building TCP transport configuration with nodelay=true");
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
pub fn build_noise_config(keypair: &Keypair) -> Result<noise::Config, noise::Error> {
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
    info!("Building Yamux multiplexing configuration");
    yamux::Config::default()
}

/// A minimal, demo-friendly QUIC configuration profile.
///
/// 目标：
/// - 在 NAT/睡眠唤醒/移动网络下更稳（keep-alive）
/// - 避免过小窗口导致吞吐差（stream/connection window）
/// - 限制并发流避免滥用（max_concurrent_stream_limit）
///
/// 你可以直接：
/// `.with_quic_config(transport::configure_quic_for_demo)`
pub fn configure_quic(cfg: quic::Config) -> quic::Config {
    info!("╔══════════════════════════════════════════╗");
    info!("║     QUIC Transport Configuration          ║");
    info!("╚══════════════════════════════════════════╝");
    info!("Handshake timeout: 10s");
    info!("Idle timeout: 60s");
    info!("Keep-alive interval: 15s");
    info!("MTU upper bound: 1452 bytes");
    info!("Max concurrent streams: 64");
    info!("Max stream data: 16 MiB");
    info!("Max connection data: 64 MiB");

    // MTU: 给个上界，通常对跨网段/复杂网络更稳。
    // 如果你更想追求吞吐，可以去掉 mtu_upper_bound，让 PMTUD 自己探测。
    let mut cfg = cfg.mtu_upper_bound(1452);

    // 握手超时：demo 里给到 10s，避免弱网偶发失败
    cfg.handshake_timeout = Duration::from_secs(10);

    // idle 超时：字段单位是 ms（u32）
    // 60s 比较保守；如果你希望更"不断线"，可以拉到 120s~300s
    cfg.max_idle_timeout = 60_000;

    // keep-alive：必须小于双方 idle_timeout 才有效
    cfg.keep_alive_interval = Duration::from_secs(15);

    // 并发双向流上限：避免一个 peer 开太多流把你打爆
    cfg.max_concurrent_stream_limit = 64;

    // 窗口：避免默认值偏小导致大 blob 的吞吐受限
    // 这里给到 16MiB / stream，64MiB / connection，足够 clipboard demo
    cfg.max_stream_data = 16 * 1024 * 1024;
    cfg.max_connection_data = 64 * 1024 * 1024;

    info!("╔══════════════════════════════════════════╗");
    info!("║     QUIC Configuration Applied           ║");
    info!("╚══════════════════════════════════════════╝");

    cfg
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
}
