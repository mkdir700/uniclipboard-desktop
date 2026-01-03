//! Network-related data models
//!
//! Pure data structures for network interfaces, connection requests, etc.

use serde::{Deserialize, Serialize};

/// Device connection status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceStatus {
    Online = 0,
    Offline = 1,
    Unknown = 2,
}

impl DeviceStatus {
    /// Convert to i32
    pub fn as_i32(self) -> i32 {
        self as i32
    }

    /// Convert from i32
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(DeviceStatus::Online),
            1 => Some(DeviceStatus::Offline),
            2 => Some(DeviceStatus::Unknown),
            _ => None,
        }
    }
}

/// Operating system platform
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
    Android,
    IOS,
    Browser,
    Unknown,
}

impl Platform {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::Windows => "windows",
            Platform::MacOS => "macos",
            Platform::Linux => "linux",
            Platform::Android => "android",
            Platform::IOS => "ios",
            Platform::Browser => "browser",
            Platform::Unknown => "unknown",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Self {
        match s {
            "windows" => Platform::Windows,
            "macos" => Platform::MacOS,
            "linux" => Platform::Linux,
            "android" => Platform::Android,
            "ios" => Platform::IOS,
            "browser" => Platform::Browser,
            _ => Platform::Unknown,
        }
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Network interface information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// Interface name (e.g., "en0", "Wi-Fi", "Ethernet")
    pub name: String,
    /// IP address
    pub ip: String,
    /// Whether this is a loopback address
    pub is_loopback: bool,
    /// Whether this is IPv4
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
    /// Whether the connection was successful
    pub success: bool,
    /// Device ID (returned on successful connection)
    pub device_id: Option<String>,
    /// Response message
    pub message: String,
}

/// Connection request message (sent to receiver)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionRequestMessage {
    /// Requester's device ID
    pub requester_device_id: String,
    /// Requester's IP address
    pub requester_ip: String,
    /// Requester's device alias (optional)
    pub requester_alias: Option<String>,
    /// Requester's platform (optional)
    pub requester_platform: Option<String>,
}

/// Connection response message (returned to initiator)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionResponseMessage {
    /// Whether the connection was accepted
    pub accepted: bool,
    /// Responder's device ID
    pub responder_device_id: String,
    /// Responder's IP address (optional)
    pub responder_ip: Option<String>,
    /// Responder's device alias (optional)
    pub responder_alias: Option<String>,
}

/// Connection request decision (frontend user confirmation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionRequestDecision {
    /// Whether to accept the connection
    pub accept: bool,
    /// Requester's device ID
    pub requester_device_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_status_conversion() {
        assert_eq!(DeviceStatus::Online.as_i32(), 0);
        assert_eq!(DeviceStatus::Offline.as_i32(), 1);
        assert_eq!(DeviceStatus::Unknown.as_i32(), 2);

        assert_eq!(DeviceStatus::from_i32(0), Some(DeviceStatus::Online));
        assert_eq!(DeviceStatus::from_i32(1), Some(DeviceStatus::Offline));
        assert_eq!(DeviceStatus::from_i32(2), Some(DeviceStatus::Unknown));
        assert_eq!(DeviceStatus::from_i32(999), None);
    }

    #[test]
    fn test_platform_conversion() {
        assert_eq!(Platform::MacOS.as_str(), "macos");
        assert_eq!(Platform::from_str("windows"), Platform::Windows);
        assert_eq!(Platform::from_str("invalid"), Platform::Unknown);
    }

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
