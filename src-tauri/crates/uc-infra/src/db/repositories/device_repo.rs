use async_trait::async_trait;
use diesel::prelude::*;

use uc_core::device::{Device, DeviceId};
use uc_core::ports::{DeviceRepositoryError, DeviceRepositoryPort};

use crate::db::ports::{DbExecutor, InsertMapper, RowMapper};
use crate::db::models::{DeviceRow, NewDeviceRow};
use crate::db::schema::t_device::dsl::*;

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
    M: InsertMapper<Device, NewDeviceRow> + RowMapper<DeviceRow, Device> + Send + Sync,
{
    async fn find_by_id(
        &self,
        device_id: &DeviceId,
    ) -> Result<Option<Device>, DeviceRepositoryError> {
        let id_str = device_id.as_str().to_string();
        self.executor
            .run(move |conn| {
                let row = t_device
                    .filter(id.eq(&id_str))
                    .first::<DeviceRow>(conn)
                    .optional()
                    .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))?;

                match row {
                    Some(r) => {
                        let device = self.mapper.to_domain(&r)
                            .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))?;
                        Ok(Some(device))
                    }
                    None => Ok(None),
                }
            })
            .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))
    }

    async fn save(&self, device: Device) -> Result<(), DeviceRepositoryError> {
        let row = self
            .mapper
            .to_row(&device)
            .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))?;

        self.executor
            .run(move |conn| {
                diesel::insert_into(t_device)
                    .values(&row)
                    .on_conflict(id)
                    .do_update()
                    .set((
                        name.eq(row.name.clone()),
                        platform.eq(row.platform.clone()),
                        is_local.eq(row.is_local),
                    ))
                    .execute(conn)
                    .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))?;
                Ok(())
            })
            .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))
    }

    async fn delete(&self, device_id: &DeviceId) -> Result<(), DeviceRepositoryError> {
        let id_str = device_id.as_str().to_string();
        self.executor
            .run(move |conn| {
                diesel::delete(t_device.filter(id.eq(&id_str)))
                    .execute(conn)
                    .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))?;
                Ok(())
            })
            .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))
    }

    async fn list_all(&self) -> Result<Vec<Device>, DeviceRepositoryError> {
        self.executor
            .run(|conn| {
                let rows = t_device
                    .load::<DeviceRow>(conn)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))?;

                let devices: Result<Vec<Device>, _> = rows
                    .into_iter()
                    .map(|row| self.mapper.to_domain(&row))
                    .collect();
                devices.map_err(|e| anyhow::anyhow!(e.to_string()))
            })
            .map_err(|e| DeviceRepositoryError::Storage(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::mappers::device_mapper::DeviceRowMapper;
    use crate::db::pool::init_db_pool;
    use std::sync::Arc;
    use uc_core::device::value_objects::{DeviceId, DeviceName};
    use uc_core::device::Platform;

    /// In-memory test executor for testing repositories
    struct TestDbExecutor {
        pool: Arc<crate::db::pool::DbPool>,
    }

    impl TestDbExecutor {
        fn new() -> Self {
            let pool = Arc::new(
                init_db_pool(":memory:").expect("Failed to create test DB pool")
            );
            Self { pool }
        }
    }

    impl DbExecutor for TestDbExecutor {
        fn run<T>(&self, f: impl FnOnce(&mut diesel::SqliteConnection) -> anyhow::Result<T>) -> anyhow::Result<T> {
            let mut conn = self.pool.get()?;
            f(&mut conn)
        }
    }

    #[tokio::test]
    async fn test_save_and_find_by_id() {
        let executor = TestDbExecutor::new();
        let mapper = DeviceRowMapper;
        let repo = DieselDeviceRepository::new(executor, mapper);

        let device = Device::new(
            DeviceId::new("test-device-1"),
            DeviceName::new("Test Device"),
            Platform::MacOS,
            true,
        );

        // Save the device
        repo.save(device.clone()).await
            .expect("Failed to save device");

        // Find the device
        let found = repo.find_by_id(&device.id()).await
            .expect("Failed to find device");

        assert!(found.is_some(), "Device should be found");
        let found_device = found.unwrap();
        assert_eq!(found_device.id().as_str(), "test-device-1");
        assert_eq!(found_device.name().as_str(), "Test Device");
        assert_eq!(found_device.platform(), Platform::MacOS);
        assert_eq!(found_device.is_local(), true);
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let executor = TestDbExecutor::new();
        let mapper = DeviceRowMapper;
        let repo = DieselDeviceRepository::new(executor, mapper);

        let result = repo.find_by_id(&DeviceId::new("non-existent")).await
            .expect("Failed to execute find");

        assert!(result.is_none(), "Non-existent device should return None");
    }

    #[tokio::test]
    async fn test_save_update() {
        let executor = TestDbExecutor::new();
        let mapper = DeviceRowMapper;
        let repo = DieselDeviceRepository::new(executor, mapper);

        let device = Device::new(
            DeviceId::new("test-device-2"),
            DeviceName::new("Original Name"),
            Platform::Windows,
            false,
        );

        // Save the device
        repo.save(device.clone()).await
            .expect("Failed to save device");

        // Update the device
        let updated_device = Device::new(
            DeviceId::new("test-device-2"),
            DeviceName::new("Updated Name"),
            Platform::Linux,
            true,
        );
        repo.save(updated_device.clone()).await
            .expect("Failed to update device");

        // Verify the update
        let found = repo.find_by_id(&updated_device.id()).await
            .expect("Failed to find device");

        assert!(found.is_some());
        let found_device = found.unwrap();
        assert_eq!(found_device.name().as_str(), "Updated Name");
        assert_eq!(found_device.platform(), Platform::Linux);
        assert_eq!(found_device.is_local(), true);
    }

    #[tokio::test]
    async fn test_delete() {
        let executor = TestDbExecutor::new();
        let mapper = DeviceRowMapper;
        let repo = DieselDeviceRepository::new(executor, mapper);

        let device = Device::new(
            DeviceId::new("test-device-3"),
            DeviceName::new("Device to Delete"),
            Platform::Linux,
            false,
        );

        // Save the device
        repo.save(device.clone()).await
            .expect("Failed to save device");

        // Verify it exists
        let found = repo.find_by_id(&device.id()).await
            .expect("Failed to find device");
        assert!(found.is_some());

        // Delete the device
        repo.delete(&device.id()).await
            .expect("Failed to delete device");

        // Verify it's gone
        let found = repo.find_by_id(&device.id()).await
            .expect("Failed to execute find");
        assert!(found.is_none(), "Device should be deleted");
    }

    #[tokio::test]
    async fn test_list_all() {
        let executor = TestDbExecutor::new();
        let mapper = DeviceRowMapper;
        let repo = DieselDeviceRepository::new(executor, mapper);

        // Save multiple devices
        let device1 = Device::new(
            DeviceId::new("test-device-4"),
            DeviceName::new("Device 4"),
            Platform::MacOS,
            true,
        );
        let device2 = Device::new(
            DeviceId::new("test-device-5"),
            DeviceName::new("Device 5"),
            Platform::Windows,
            false,
        );
        let device3 = Device::new(
            DeviceId::new("test-device-6"),
            DeviceName::new("Device 6"),
            Platform::Linux,
            false,
        );

        repo.save(device1).await.expect("Failed to save device1");
        repo.save(device2).await.expect("Failed to save device2");
        repo.save(device3).await.expect("Failed to save device3");

        // List all devices
        let devices = repo.list_all().await
            .expect("Failed to list devices");

        assert_eq!(devices.len(), 3, "Should have 3 devices");
    }

    #[tokio::test]
    async fn test_list_all_empty() {
        let executor = TestDbExecutor::new();
        let mapper = DeviceRowMapper;
        let repo = DieselDeviceRepository::new(executor, mapper);

        // List all devices when none exist
        let devices = repo.list_all().await
            .expect("Failed to list devices");

        assert_eq!(devices.len(), 0, "Should have 0 devices");
    }
}
