//! Network-related domain models.
//!
//! Contains network interface information, manual connection requests, etc.

use serde::{Deserialize, Serialize};

/// Network interface information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// Interface name (e.g., "en0", "Wi-Fi", "以太网")
    pub name: String,
    /// IP address
    pub ip: String,
    /// Is loopback address
    pub is_loopback: bool,
    /// Is IPv4
    pub is_ipv4: bool,
}

/// Manual connection request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualConnectionRequest {
    /// Target device IP address
    pub ip: String,
    /// Target device port
    pub port: u16,
}

/// Manual connection response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualConnectionResponse {
    /// Whether successful
    pub success: bool,
    /// Device ID (returned on success)
    pub device_id: Option<String>,
    /// Response message
    pub message: String,
}

/// Connection request message (sent to receiver)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionRequestMessage {
    /// Requester device ID
    pub requester_device_id: String,
    /// Requester IP address
    pub requester_ip: String,
    /// Requester device alias (optional)
    pub requester_alias: Option<String>,
    /// Requester platform (optional)
    pub requester_platform: Option<String>,
}

/// Connection response message (receiver returns to initiator)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionResponseMessage {
    /// Whether to accept connection
    pub accepted: bool,
    /// Responder device ID
    pub responder_device_id: String,
    /// Responder IP address (optional)
    pub responder_ip: Option<String>,
    /// Responder device alias (optional)
    pub responder_alias: Option<String>,
}

/// Connection request decision (frontend user confirmation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionRequestDecision {
    /// Whether to accept connection
    pub accept: bool,
    /// Requester device ID
    pub requester_device_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_interface_serialization() {
        let iface = NetworkInterface {
            name: "en0".to_string(),
            ip: "192.168.1.100".to_string(),
            is_loopback: false,
            is_ipv4: true,
        };

        let json = serde_json::to_string(&iface).unwrap();
        let deserialized: NetworkInterface = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "en0");
        assert_eq!(deserialized.ip, "192.168.1.100");
        assert!(!deserialized.is_loopback);
        assert!(deserialized.is_ipv4);
    }

    #[test]
    fn test_manual_connection_request() {
        let request = ManualConnectionRequest {
            ip: "192.168.1.100".to_string(),
            port: 29217,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: ManualConnectionRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.ip, "192.168.1.100");
        assert_eq!(deserialized.port, 29217);
    }

    #[test]
    fn test_connection_request_message() {
        let msg = ConnectionRequestMessage {
            requester_device_id: "123456".to_string(),
            requester_ip: "192.168.1.100".to_string(),
            requester_alias: Some("My Device".to_string()),
            requester_platform: Some("macos aarch64".to_string()),
        };

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: ConnectionRequestMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.requester_device_id, "123456");
        assert_eq!(deserialized.requester_ip, "192.168.1.100");
        assert_eq!(deserialized.requester_alias, Some("My Device".to_string()));
        assert_eq!(
            deserialized.requester_platform,
            Some("macos aarch64".to_string())
        );
    }
}
