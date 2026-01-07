use async_trait::async_trait;
use diesel::prelude::*;

use uc_core::device::{Device, DeviceId};
use uc_core::ports::{DeviceRepositoryError, DeviceRepositoryPort};

use crate::db::{models::DeviceRow, pool::DbPool, schema::t_device::dsl::*};

pub struct DieselDeviceRepository {
    pool: DbPool,
}

impl DieselDeviceRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DeviceRepositoryPort for DieselDeviceRepository {
    async fn find_by_id(
        &self,
        device_id: &DeviceId,
    ) -> Result<Option<Device>, DeviceRepositoryError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))?;

        let row = t_device
            .filter(id.eq(device_id.as_str()))
            .first::<DeviceRow>(&mut conn)
            .optional()
            .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))?;

        Ok(row.map(Device::from))
    }

    async fn save(&self, device: Device) -> Result<(), DeviceRepositoryError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))?;

        let row = DeviceRow::from(&device);

        diesel::insert_into(t_device)
            .values(&row)
            .on_conflict(id)
            .do_update()
            .set(name.eq(row.name.clone()))
            .execute(&mut conn)
            .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, device_id: &DeviceId) -> Result<(), DeviceRepositoryError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))?;

        diesel::delete(t_device.filter(id.eq(device_id.as_str())))
            .execute(&mut conn)
            .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))?;

        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<Device>, DeviceRepositoryError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))?;

        let rows = t_device
            .load::<DeviceRow>(&mut conn)
            .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))?;

        Ok(rows.into_iter().map(Device::from).collect())
    }
}
