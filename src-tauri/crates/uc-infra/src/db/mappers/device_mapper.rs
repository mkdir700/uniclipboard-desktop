use crate::db::models::DeviceRow;
use crate::db::ports::InsertMapper;
use anyhow::Result;
use uc_core::device::Device;

pub struct DeviceRowMapper;

impl InsertMapper<Device, DeviceRow> for DeviceRowMapper {
    fn to_row(&self, domain: &Device) -> Result<DeviceRow> {
        Ok(DeviceRow {
            id: domain.id().as_str().to_string(),
            name: domain.name().as_str().to_string(),
            created_at: chrono::Utc::now().timestamp(),
        })
    }
}
