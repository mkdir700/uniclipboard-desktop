use super::platform::Platform;
use super::value_objects::{DeviceId, DeviceName};

/// Device information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    /// Device ID
    pub id: DeviceId,
    pub name: DeviceName,
    /// Platform identifier
    pub platform: Platform,
    /// Whether this is the local device
    pub is_local: bool,
}

impl Device {
    pub fn new(id: DeviceId, name: DeviceName, platform: Platform, is_local: bool) -> Self {
        Self {
            id,
            name,
            platform,
            is_local,
        }
    }

    pub fn id(&self) -> &DeviceId {
        &self.id
    }

    pub fn name(&self) -> &DeviceName {
        &self.name
    }

    pub fn platform(&self) -> Platform {
        self.platform
    }

    pub fn is_local(&self) -> bool {
        self.is_local
    }
}
