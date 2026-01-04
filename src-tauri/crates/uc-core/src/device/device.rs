//! Device domain model

use serde::{Deserialize, Serialize};
use std::fmt;

use super::{DeviceStatus, Platform};

/// Device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    /// Device ID
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
    /// Last update timestamp
    pub updated_at: Option<i32>,
    /// Device platform
    pub platform: Option<Platform>,
    /// Device alias (user-set name)
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
    /// Create a new Device with required fields
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
    pub fn new_local_device(id: String, ip: String, webserver_port: u16) -> Self {
        Self {
            id,
            ip: Some(ip),
            port: None,
            server_port: Some(webserver_port),
            status: DeviceStatus::Unknown,
            self_device: true,
            updated_at: None,
            platform: None, // Will be set by the caller
            alias: None,
            peer_id: None,
            device_name: None,
            is_paired: false,
            last_seen: None,
        }
    }
}

impl PartialEq for Device {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Device {}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Device(id: {}, ip: {}, port: {}, server_port: {})",
            self.id,
            self.ip.as_ref().unwrap_or(&String::new()),
            self.port.map(|p| p.to_string()).unwrap_or_else(|| "None".to_string()),
            self.server_port
                .map(|p| p.to_string())
                .unwrap_or_else(|| "None".to_string())
        )
    }
}
