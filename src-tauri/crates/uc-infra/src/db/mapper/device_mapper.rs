use crate::db::models::DeviceRow;
use uc_core::device::{Device, DeviceId, DeviceName};

impl From<DeviceRow> for Device {
    fn from(row: DeviceRow) -> Self {
        Device::new(DeviceId::new(row.id), DeviceName::new(row.name))
    }
}

impl From<&Device> for DeviceRow {
    fn from(device: &Device) -> Self {
        DeviceRow {
            id: device.id().as_str().to_string(),
            name: device.name().as_str().to_string(),
            created_at: chrono::Utc::now().timestamp(),
        }
    }
}
