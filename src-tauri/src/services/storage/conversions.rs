//! Data conversions between domain and database models
//!
//! This module contains bidirectional conversion implementations between
//! domain models (pure data structures) and database models (persistence layer).

use crate::domain::device::{Device, DeviceStatus, Platform};
use crate::infrastructure::storage::db::models::device::DbDevice;
use std::str::FromStr;

// ========== Device ↔ DbDevice 转换 ==========

/// Convert a domain Device to a database DbDevice (by reference)
impl From<&Device> for DbDevice {
    fn from(device: &Device) -> Self {
        DbDevice {
            id: device.id.clone(),
            ip: device.ip.clone(),
            port: device.port.map(|p| p as i32),
            server_port: device.server_port.map(|p| p as i32),
            status: device.status.clone() as i32,
            self_device: device.self_device,
            updated_at: device.updated_at.unwrap_or(0) as i32,
            alias: device.alias.clone(),
            platform: device.platform.map(|p| p.to_string()),
            peer_id: device.peer_id.clone(),
            device_name: device.device_name.clone(),
            is_paired: device.is_paired,
            last_seen: device.last_seen,
        }
    }
}

/// Convert a database DbDevice to a domain Device (by reference)
impl From<&DbDevice> for Device {
    fn from(db_device: &DbDevice) -> Self {
        let mut device = Device::new(
            db_device.id.clone(),
            db_device.ip.clone(),
            db_device.port.map(|p| p as u16),
            db_device.server_port.map(|p| p as u16),
        );
        device.self_device = db_device.self_device;
        device.status = DeviceStatus::try_from(db_device.status).unwrap_or(DeviceStatus::Unknown);
        device.updated_at = Some(db_device.updated_at);
        device.alias = db_device.alias.clone();
        device.platform = db_device
            .platform
            .as_ref()
            .map(|p| Platform::from_str(p).unwrap_or(Platform::Unknown));
        device.peer_id = db_device.peer_id.clone();
        device.device_name = db_device.device_name.clone();
        device.is_paired = db_device.is_paired;
        device.last_seen = db_device.last_seen;
        device
    }
}

/// Convert a database DbDevice to a domain Device (by value)
impl From<DbDevice> for Device {
    fn from(db_device: DbDevice) -> Self {
        let mut device = Device::new(
            db_device.id,
            db_device.ip,
            db_device.port.map(|p| p as u16),
            db_device.server_port.map(|p| p as u16),
        );
        device.self_device = db_device.self_device;
        device.status = DeviceStatus::try_from(db_device.status).unwrap_or(DeviceStatus::Unknown);
        device.updated_at = Some(db_device.updated_at);
        device.alias = db_device.alias;
        device.platform = db_device
            .platform
            .map(|p| Platform::from_str(&p).unwrap_or(Platform::Unknown));
        device.peer_id = db_device.peer_id;
        device.device_name = db_device.device_name;
        device.is_paired = db_device.is_paired;
        device.last_seen = db_device.last_seen;
        device
    }
}

// ========== 辅助转换函数 ==========

/// Convert DeviceStatus enum to i32
pub fn device_status_to_i32(status: DeviceStatus) -> i32 {
    status as i32
}

/// Convert i32 to DeviceStatus enum (with Unknown fallback)
pub fn i32_to_device_status(value: i32) -> DeviceStatus {
    DeviceStatus::try_from(value).unwrap_or(DeviceStatus::Unknown)
}

/// Convert Option<Platform> to Option<String>
pub fn platform_to_string(platform: Option<Platform>) -> Option<String> {
    platform.map(|p| p.to_string())
}

/// Convert Option<String> to Option<Platform>
pub fn string_to_platform(s: Option<String>) -> Option<Platform> {
    s.and_then(|p| Platform::from_str(&p).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_to_db_device() {
        let device = Device::new(
            "test-id".to_string(),
            Some("192.168.1.1".to_string()),
            Some(8080u16),
            Some(9000u16),
        );

        let db_device: DbDevice = (&device).into();

        assert_eq!(db_device.id, "test-id");
        assert_eq!(db_device.ip, Some("192.168.1.1".to_string()));
        assert_eq!(db_device.port, Some(8080));
        assert_eq!(db_device.server_port, Some(9000));
    }

    #[test]
    fn test_db_device_to_device() {
        let db_device = DbDevice {
            id: "test-id".to_string(),
            ip: Some("192.168.1.1".to_string()),
            port: Some(8080),
            server_port: Some(9000),
            status: 0,
            self_device: false,
            updated_at: 123456,
            alias: Some("Test Device".to_string()),
            platform: Some("Windows".to_string()),
            peer_id: Some("peer-id".to_string()),
            device_name: Some("My Device".to_string()),
            is_paired: true,
            last_seen: Some(123456),
        };

        let device: Device = db_device.into();

        assert_eq!(device.id, "test-id");
        assert_eq!(device.ip, Some("192.168.1.1".to_string()));
        assert_eq!(device.port, Some(8080u16));
        assert_eq!(device.server_port, Some(9000u16));
        assert_eq!(device.status, DeviceStatus::Online);
        assert_eq!(device.alias, Some("Test Device".to_string()));
        assert_eq!(device.platform, Some(Platform::Windows));
        assert_eq!(device.is_paired, true);
    }

    #[test]
    fn test_device_status_conversion() {
        assert_eq!(device_status_to_i32(DeviceStatus::Online), 0);
        assert_eq!(device_status_to_i32(DeviceStatus::Offline), 1);
        assert_eq!(device_status_to_i32(DeviceStatus::Unknown), 2);

        assert_eq!(i32_to_device_status(0), DeviceStatus::Online);
        assert_eq!(i32_to_device_status(1), DeviceStatus::Offline);
        assert_eq!(i32_to_device_status(2), DeviceStatus::Unknown);
        assert_eq!(i32_to_device_status(999), DeviceStatus::Unknown); // Fallback
    }

    #[test]
    fn test_platform_conversion() {
        assert_eq!(platform_to_string(Some(Platform::Windows)), Some("Windows".to_string()));
        assert_eq!(platform_to_string(None), None);

        assert_eq!(string_to_platform(Some("Windows".to_string())), Some(Platform::Windows));
        assert_eq!(string_to_platform(Some("invalid".to_string())), None); // FromStr returns Err
        assert_eq!(string_to_platform(None), None);
    }
}
