use super::value_objects::{DeviceId, DeviceName};

/// Device information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    /// Device ID
    pub id: DeviceId,
    pub name: DeviceName,
}

impl Device {
    pub fn new(id: DeviceId, name: DeviceName) -> Self {
        Self { id, name }
    }

    pub fn id(&self) -> &DeviceId {
        &self.id
    }

    pub fn name(&self) -> &DeviceName {
        &self.name
    }
}
