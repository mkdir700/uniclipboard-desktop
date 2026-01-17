use crate::db::models::{DeviceRow, NewDeviceRow};
use crate::db::ports::{InsertMapper, RowMapper};
use anyhow::Result;
use uc_core::device::Device;

pub struct DeviceRowMapper;

impl InsertMapper<Device, NewDeviceRow> for DeviceRowMapper {
    fn to_row(&self, domain: &Device) -> Result<NewDeviceRow> {
        Ok(NewDeviceRow {
            id: domain.id().as_str().to_string(),
            name: domain.name().as_str().to_string(),
            platform: domain.platform().to_string(),
            is_local: domain.is_local(),
            created_at: chrono::Utc::now().timestamp(),
        })
    }
}

impl RowMapper<DeviceRow, Device> for DeviceRowMapper {
    fn to_domain(&self, row: &DeviceRow) -> Result<Device> {
        use uc_core::device::value_objects::{DeviceId, DeviceName};
        use uc_core::device::Platform;

        let platform = row
            .platform
            .parse::<Platform>()
            .unwrap_or(Platform::Unknown);

        Ok(Device::new(
            DeviceId::new(row.id.clone()),
            DeviceName::new(row.name.clone()),
            platform,
            row.is_local,
        ))
    }
}
