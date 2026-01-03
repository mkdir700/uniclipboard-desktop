//! Device data model
//!
//! Pure device model without infrastructure dependencies.

use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

use super::network::{DeviceStatus, Platform};

/// Device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    /// Device ID (6-character hex string)
    pub id: String,
    /// Device IP address
    pub ip: Option<String>,
    /// Device port
    pub port: Option<u16>,
    /// Device server port
    pub server_port: Option<u16>,
    /// Device status
    pub status: DeviceStatus,
    /// Whether this is the local device
    pub self_device: bool,
    /// Device update timestamp (Unix timestamp)
    pub updated_at: Option<i32>,
    /// Device platform
    pub platform: Option<Platform>,
    /// Device alias (custom name)
    pub alias: Option<String>,
    /// libp2p PeerId for P2P networking
    pub peer_id: Option<String>,
    /// Human-readable device name
    pub device_name: Option<String>,
    /// Whether device has completed P2P pairing
    pub is_paired: bool,
    /// Timestamp of last contact (Unix timestamp)
    pub last_seen: Option<i32>,
}

impl Device {
    /// Create a new Device with basic information
    pub fn new(
        id: String,
        ip: Option<String>,
        port: Option<u16>,
        server_port: Option<u16>,
    ) -> Self {
        Self {
            id,
            ip,
            port,
            server_port,
            status: DeviceStatus::Unknown,
            self_device: false,
            updated_at: None,
            platform: None,
            alias: None,
            peer_id: None,
            device_name: None,
            is_paired: false,
            last_seen: None,
        }
    }

    /// Create a new local device
    ///
    /// # Arguments
    /// * `id` - Device ID (6-character hex string)
    /// * `ip` - Local IP address
    /// * `webserver_port` - Web server port
    /// * `platform` - Operating system platform
    pub fn new_local_device(
        id: String,
        ip: String,
        webserver_port: u16,
        platform: Platform,
    ) -> Self {
        Self {
            id,
            ip: Some(ip),
            port: None,
            server_port: Some(webserver_port),
            status: DeviceStatus::Unknown,
            self_device: true,
            updated_at: None,
            platform: Some(platform),
            alias: None,
            peer_id: None,
            device_name: None,
            is_paired: false,
            last_seen: None,
        }
    }

    /// Get display name for this device
    ///
    /// Returns device_name > alias > "Device {id}"
    pub fn display_name(&self) -> String {
        self.device_name
            .clone()
            .or_else(|| self.alias.clone())
            .unwrap_or_else(|| format!("Device {}", &self.id))
    }

    /// Check if device is considered online
    pub fn is_online(&self) -> bool {
        self.status == DeviceStatus::Online
    }

    /// Get the device's short ID (first 6 characters)
    pub fn short_id(&self) -> &str {
        &self.id[..self.id.len().min(6)]
    }
}

impl PartialEq for Device {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Device {}

impl Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Device(id: {}, ip: {:?}, port: {:?}, server_port: {:?})",
            self.id, self.ip, self.port, self.server_port
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_new() {
        let device = Device::new(
            "abc123".to_string(),
            Some("192.168.1.100".to_string()),
            Some(8080),
            Some(29217),
        );

        assert_eq!(device.id, "abc123");
        assert_eq!(device.ip, Some("192.168.1.100".to_string()));
        assert_eq!(device.port, Some(8080));
        assert_eq!(device.server_port, Some(29217));
        assert_eq!(device.status, DeviceStatus::Unknown);
        assert!(!device.self_device);
        assert!(!device.is_paired);
    }

    #[test]
    fn test_device_new_local() {
        let device = Device::new_local_device(
            "def456".to_string(),
            "192.168.1.101".to_string(),
            29217,
            Platform::MacOS,
        );

        assert_eq!(device.id, "def456");
        assert_eq!(device.ip, Some("192.168.1.101".to_string()));
        assert_eq!(device.server_port, Some(29217));
        assert!(device.self_device);
        assert_eq!(device.platform, Some(Platform::MacOS));
    }

    #[test]
    fn test_device_display_name() {
        let mut device = Device::new("abc123".to_string(), None, None, None);

        // Default display name
        assert_eq!(device.display_name(), "Device abc123");

        // Alias takes priority
        device.alias = Some("My Device".to_string());
        assert_eq!(device.display_name(), "My Device");

        // device_name takes highest priority
        device.device_name = Some("Real Name".to_string());
        assert_eq!(device.display_name(), "Real Name");
    }

    #[test]
    fn test_device_equality() {
        let device1 = Device::new("abc123".to_string(), None, None, None);
        let device2 = Device::new("abc123".to_string(), Some("1.2.3.4".to_string()), Some(80), None);
        let device3 = Device::new("def456".to_string(), None, None, None);

        assert_eq!(device1, device2);
        assert_ne!(device1, device3);
    }

    #[test]
    fn test_device_is_online() {
        let mut device = Device::new("abc123".to_string(), None, None, None);

        device.status = DeviceStatus::Online;
        assert!(device.is_online());

        device.status = DeviceStatus::Offline;
        assert!(!device.is_online());

        device.status = DeviceStatus::Unknown;
        assert!(!device.is_online());
    }

    #[test]
    fn test_device_serialization() {
        let device = Device::new(
            "abc123".to_string(),
            Some("192.168.1.100".to_string()),
            Some(8080),
            Some(29217),
        );

        let json = serde_json::to_string(&device).unwrap();
        let deserialized: Device = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, "abc123");
        assert_eq!(deserialized.ip, Some("192.168.1.100".to_string()));
        assert_eq!(deserialized.port, Some(8080));
        assert_eq!(deserialized.server_port, Some(29217));
    }

    #[test]
    fn test_short_id() {
        let device = Device::new("abc123def456".to_string(), None, None, None);
        assert_eq!(device.short_id(), "abc123");

        let short_device = Device::new("a1b2c3".to_string(), None, None, None);
        assert_eq!(short_device.short_id(), "a1b2c3");
    }
}
