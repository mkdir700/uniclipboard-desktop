use crate::infrastructure::storage::db::models::device::DbDevice;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::{self, Display};
use crate::utils::helpers::generate_device_id;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DeviceStatus {
    Online = 0,
    Offline = 1,
    Unknown = 2,
}

impl PartialEq for DeviceStatus {
    fn eq(&self, other: &Self) -> bool {
        *self as i32 == *other as i32
    }
}

impl Eq for DeviceStatus {}

impl PartialOrd for DeviceStatus {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DeviceStatus {
    fn cmp(&self, other: &Self) -> Ordering {
        (*self as i32).cmp(&(*other as i32))
    }
}

impl From<DeviceStatus> for i32 {
    fn from(status: DeviceStatus) -> Self {
        status as i32
    }
}

impl TryFrom<i32> for DeviceStatus {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(DeviceStatus::Online),
            1 => Ok(DeviceStatus::Offline),
            2 => Ok(DeviceStatus::Unknown),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    /// 设备ID
    pub id: String,
    /// 设备IP
    pub ip: Option<String>,
    /// 设备端口
    pub port: Option<u16>,
    /// 设备服务端口
    pub server_port: Option<u16>,
    /// 设备状态
    pub status: DeviceStatus,
    /// 是否是本机设备
    pub self_device: bool,
    /// 设备更新时间(时间戳)
    pub updated_at: Option<i32>,
}

impl Device {
    pub fn new(
        id: String,
        ip: Option<String>,
        port: Option<u16>,
        server_port: Option<u16>,
    ) -> Self {
        Self {
            id,
            ip,
            port,
            server_port,
            status: DeviceStatus::Unknown,
            self_device: false,
            updated_at: None,
        }
    }

    pub fn new_local_device(ip: String, webserver_port: u16) -> Self {
        Self {
            id: generate_device_id(),
            ip: Some(ip),
            port: None,
            server_port: Some(webserver_port),
            status: DeviceStatus::Unknown,
            self_device: true,
            updated_at: None,
        }
    }
}

impl PartialEq for Device {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Device(id: {}, ip: {}, port: {}, server_port: {})",
            self.id,
            self.ip.clone().unwrap_or_default(),
            self.port.clone().unwrap_or_default(),
            self.server_port.clone().unwrap_or_default()
        )
    }
}

// 将 Device 转换为 DbDevice
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
        }
    }
}

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
        device
    }
}

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
        device
    }
}
