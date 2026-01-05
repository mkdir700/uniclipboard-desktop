use crate::{
    device::{Device, DeviceId},
    ports::errors::DeviceRepositoryError,
};
use async_trait::async_trait;

#[async_trait]
pub trait DeviceRepositoryPort: Send + Sync {
    /// 根据 device_id 查询设备
    async fn find_by_id(&self, id: &DeviceId) -> Result<Option<Device>, DeviceRepositoryError>;

    /// 保存（新增 or 覆盖）
    async fn save(&self, device: Device) -> Result<(), DeviceRepositoryError>;

    /// 删除设备
    async fn delete(&self, id: &DeviceId) -> Result<(), DeviceRepositoryError>;

    /// 查询所有已知设备
    async fn list_all(&self) -> Result<Vec<Device>, DeviceRepositoryError>;
}
