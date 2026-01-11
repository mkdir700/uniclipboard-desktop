use async_trait::async_trait;
use diesel::prelude::*;

use uc_core::device::{Device, DeviceId};
use uc_core::ports::{DeviceRepositoryError, DeviceRepositoryPort};

use crate::db::ports::DbExecutor;
use crate::db::ports::InsertMapper;
use crate::db::{models::DeviceRow, schema::t_device::dsl::*};

pub struct DieselDeviceRepository<E, M> {
    executor: E,
    mapper: M,
}

impl<E, M> DieselDeviceRepository<E, M> {
    pub fn new(executor: E, mapper: M) -> Self {
        Self { executor, mapper }
    }
}

#[async_trait]
impl<E, M> DeviceRepositoryPort for DieselDeviceRepository<E, M>
where
    E: DbExecutor,
    M: InsertMapper<Device, DeviceRow>,
{
    async fn find_by_id(
        &self,
        device_id: &DeviceId,
    ) -> Result<Option<Device>, DeviceRepositoryError> {
        unimplemented!()
    }

    async fn save(&self, device: Device) -> Result<(), DeviceRepositoryError> {
        unimplemented!()
        // self.executor.run(|conn| {
        //     let row = DeviceRow::from(&device);
        //     diesel::insert_into(t_device)
        //         .values(&row)
        //         .on_conflict(id)
        //         .do_update()
        //         .set(name.eq(row.name.clone()))
        //         .execute(conn)
        //         .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))?;
        //     Ok(())
        // })?;
        // Ok(())
    }

    async fn delete(&self, device_id: &DeviceId) -> Result<(), DeviceRepositoryError> {
        unimplemented!()
        // self.executor.run(|conn| {
        //     diesel::delete(t_device.filter(id.eq(device_id.as_str())))
        //         .execute(conn)
        //         .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))?;
        //     Ok(())
        // })?;

        // Ok(())
    }

    async fn list_all(&self) -> Result<Vec<Device>, DeviceRepositoryError> {
        unimplemented!()
        // self.executor.run(|conn| {
        //     let rows = t_device
        //         .load::<DeviceRow>(&mut conn)
        //         .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))?;

        //     Ok(rows.into_iter().map(Device::from).collect())
        // })?
    }
}
