use crate::db::dao;
use crate::db::DB_POOL;
use crate::models::DbDevice;
use anyhow::Result;
use chrono::Utc;
use log::warn;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{self, Display};
use tokio::sync::broadcast;

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

#[derive(Clone)]
pub struct DeviceManager {}

pub static GLOBAL_DEVICE_MANAGER: Lazy<DeviceManager> = Lazy::new(|| DeviceManager::new());

pub static NEW_DEVICE_BROADCASTER: Lazy<broadcast::Sender<Device>> = Lazy::new(|| {
    let (sender, _) = broadcast::channel(20);
    sender
});

// 可选：添加一个便捷函数来获取 DeviceManager 的引用
pub fn get_device_manager() -> &'static DeviceManager {
    &GLOBAL_DEVICE_MANAGER
}

// 新增：全局函数用于订阅新设备
pub fn subscribe_new_devices() -> broadcast::Receiver<Device> {
    NEW_DEVICE_BROADCASTER.subscribe()
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

impl DeviceManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn set_self_device(&self, device: &Device) -> Result<()> {
        let mut conn = DB_POOL.get_connection()?;
        let mut db_device = DbDevice::from(device);
        if dao::device::is_exist(&mut conn, &db_device.id)? {
            db_device.self_device = true;
            println!("Updating device: {:?}", db_device);
            dao::device::update_device(&mut conn, &db_device)?;
        } else {
            return Err(anyhow::anyhow!("Device not found"));
        }
        Ok(())
    }

    /// 设置设备在线
    pub fn set_online(&self, device_id: &str) -> Result<()> {
        let mut conn = DB_POOL.get_connection()?;
        dao::device::update_device_status(&mut conn, device_id, DeviceStatus::Online as i32)?;
        Ok(())
    }

    /// 设置设备离线
    pub fn set_offline(&self, device_id: &str) -> Result<()> {
        let mut conn = DB_POOL.get_connection()?;
        dao::device::update_device_status(&mut conn, device_id, DeviceStatus::Offline as i32)?;
        Ok(())
    }

    /// 添加设备，如果设备已存在，则更新设备
    pub fn add(&self, device: Device) -> Result<()> {
        let mut conn = DB_POOL.get_connection()?;
        let mut db_device = DbDevice::from(&device);
        db_device.updated_at = Utc::now().timestamp() as i32;
        if dao::device::is_exist(&mut conn, &db_device.id)? {
            warn!("Device will be overwritten: {}", db_device.id);
            dao::device::update_device(&mut conn, &db_device)?;
        } else {
            dao::device::insert_device(&mut conn, &db_device)?;
        }
        let _ = NEW_DEVICE_BROADCASTER.send(device);
        Ok(())
    }

    /// 合并
    /// 如果设备已存在，判断设备的时间戳，如果时间戳大于当前设备的时间戳，则更新设备
    /// 被新增的设备将通过广播通知
    pub fn merge(&self, devices: &Vec<Device>) -> Result<()> {
        let mut conn = DB_POOL.get_connection()?;

        let exist_devices: HashMap<String, DbDevice> = dao::device::get_all_devices(&mut conn)?
            .into_iter()
            .map(|d| (d.id.clone(), d))
            .collect();

        let new_devices: Vec<DbDevice> = devices
            .iter()
            .filter(|d| !exist_devices.contains_key(&d.id))
            .map(|d| DbDevice::from(d))
            .collect();
        dao::device::batch_insert_devices(&mut conn, &new_devices)?;

        // 通知新设备
        for device in new_devices {
            let _ = NEW_DEVICE_BROADCASTER.send(device.into());
        }

        // 获取 devices 中 id 相同但 ip 和 server_port 不同的设备
        let new_devices: Vec<&Device> = devices
            .iter()
            .filter(|d| {
                if let Some(exist_device) = exist_devices.get(&d.id) {
                    d.ip != exist_device.ip
                        || d.server_port.unwrap_or(0) as i32
                            != exist_device.server_port.unwrap_or(0)
                } else {
                    false
                }
            })
            .filter(|d| {
                if let Some(exist_device) = exist_devices.get(&d.id) {
                    d.updated_at.unwrap_or(0) > exist_device.updated_at
                } else {
                    false
                }
            })
            .collect();

        for device in new_devices {
            dao::device::update_device(&mut conn, &DbDevice::from(device))?;
        }
        Ok(())
    }

    /// 获取离线设备
    pub fn get_offline_devices(&self) -> Result<Vec<Device>> {
        let devices = self.get_all_devices()?;
        let offline_devices = devices
            .into_iter()
            .filter(|d| d.status == DeviceStatus::Offline)
            .collect();
        Ok(offline_devices)
    }

    #[allow(dead_code)]
    pub fn remove(&self, device_id: &str) -> Result<()> {
        let mut conn = DB_POOL.get_connection()?;
        dao::device::delete_device(&mut conn, device_id)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get(&self, device_id: &str) -> Result<Option<Device>> {
        let mut conn = DB_POOL.get_connection()?;
        let device = dao::device::get_device(&mut conn, device_id)?;
        Ok(device.map(|d| d.into()))
    }

    #[allow(dead_code)]
    pub fn has(&self, device_id: &str) -> Result<bool> {
        let mut conn = DB_POOL.get_connection()?;
        Ok(dao::device::is_exist(&mut conn, device_id)?)
    }

    pub fn get_all_devices(&self) -> Result<Vec<Device>> {
        let mut conn = DB_POOL.get_connection()?;
        let devices = dao::device::get_all_devices(&mut conn)?;
        Ok(devices.into_iter().map(|d| (&d).into()).collect())
    }

    // 获取除了自己的所有设备
    pub fn get_all_devices_except_self(&self) -> Result<Vec<Device>> {
        let devices = self
            .get_all_devices()?
            .into_iter()
            .filter(|d| !d.self_device)
            .collect();
        Ok(devices)
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) -> Result<()> {
        let mut conn = DB_POOL.get_connection()?;
        dao::device::clear_devices(&mut conn)?;
        Ok(())
    }

    /// 通过 ip 和 port 获取设备
    #[allow(dead_code)]
    pub fn get_device_by_ip_and_port(&self, ip: &str, port: u16) -> Result<Option<Device>> {
        let mut conn = DB_POOL.get_connection()?;
        let device = dao::device::get_device_by_ip_and_port(&mut conn, ip, port as i32)?;
        Ok(device.map(|d| d.into()))
    }
}

#[cfg(test)]
use serial_test::serial;

#[cfg(test)]
#[serial]
mod tests {
    use super::*;
    use chrono::Utc;
    use dotenv::dotenv;
    use serial_test::serial;
    use std::env;
    use std::sync::Once;
    static INIT: Once = Once::new();
    static mut DB_NAME: Option<String> = None;

    #[ctor::ctor]
    fn setup() {
        INIT.call_once(|| {
            dotenv().ok();
            let db_name = format!("uniclipboard_test_{}.db", uuid::Uuid::new_v4());
            env::set_var("DATABASE_URL", &db_name);
            DB_POOL.run_migrations().unwrap();

            // 安全地存储数据库名称
            unsafe {
                DB_NAME = Some(db_name);
            }

            // 注册清理函数
            std::panic::set_hook(Box::new(|_| {
                teardown();
            }));
        });
    }

    fn teardown() {
        // 清空数据库
        let mut conn = DB_POOL.get_connection().unwrap();
        dao::device::clear_devices(&mut conn).unwrap();
    }

    #[ctor::dtor]
    fn cleanup() {
        // 删除数据库文件
        if let Some(db_name) = unsafe { DB_NAME.as_ref() } {
            std::fs::remove_file(db_name).unwrap_or_else(|e| {
                println!("Failed to remove database file: {}", e);
            });
        }
    }

    #[test]
    #[serial]
    fn test_device_manager() {
        let manager = DeviceManager::new();
        let device = Device::new("test".to_string(), None, None, None);
        manager.add(device.clone()).unwrap();
        assert_eq!(manager.get("test").unwrap(), Some(device));
        assert_eq!(manager.get("test1").unwrap(), None);
        let device = manager.get("test").unwrap().unwrap();
        assert_ne!(device.updated_at, None);
        teardown()
    }

    #[test]
    #[serial]
    fn test_device_eq() {
        let device1 = Device::new("test".to_string(), None, None, None);
        let device2 = Device::new("test".to_string(), None, None, None);
        assert_eq!(device1, device2);
        teardown()
    }

    #[test]
    #[serial]
    fn test_device_manager_has() {
        let manager = DeviceManager::new();
        let device = Device::new("test".to_string(), None, None, None);
        manager.add(device.clone()).unwrap();
        assert_eq!(manager.has("test").unwrap(), true);
        assert_eq!(manager.has("test1").unwrap(), false);
        teardown()
    }

    #[test]
    #[serial]
    fn test_merge() {
        let manager = DeviceManager::new();
        let devices = vec![
            Device::new("test".to_string(), None, None, None),
            Device::new("test1".to_string(), None, None, None),
        ];
        manager.merge(&devices).unwrap();
        let devices_from_db = manager.get_all_devices().unwrap();
        assert_eq!(devices_from_db.len(), 2);
        for d in devices_from_db {
            assert_ne!(d.updated_at, None);
        }

        let mut device = Device::new(
            "test".to_string(),
            Some("127.0.0.1".to_string()),
            Some(12345),
            None,
        );
        let ts = Utc::now().timestamp() as i32;
        // 设置当前时间戳
        device.updated_at = Some(ts);
        let devices = vec![device];
        manager.merge(&devices).unwrap();
        let device = manager.get("test").unwrap().unwrap();
        println!("Device: {:?}", device);
        assert_eq!(device.ip, Some("127.0.0.1".to_string()));
        assert_eq!(device.port, Some(12345));
        assert_eq!(device.updated_at, Some(ts));
        teardown()
    }

    #[test]
    #[serial]
    fn test_set_self_device() {
        let manager = DeviceManager::new();
        let mut device = Device::new("test".to_string(), None, None, None);

        println!("Adding device: {:?}", device);
        if let Err(e) = manager.add(device.clone()) {
            println!("Failed to add device: {}", e);
            panic!("Failed to add device: {}", e);
        }

        device.server_port = Some(12345);
        device.status = DeviceStatus::Online;
        println!("Setting self device");
        match manager.set_self_device(&device) {
            Ok(_) => println!("Self device set successfully"),
            Err(e) => {
                println!("Failed to set self device: {}", e);
                panic!("Failed to set self device: {}", e);
            }
        }

        println!("Getting device");
        match manager.get("test") {
            Ok(Some(d)) => {
                println!("Retrieved device: {:?}", d);
                assert_eq!(d.self_device, true, "Self device flag not set correctly");
            }
            Ok(None) => {
                println!("Device not found after setting");
                panic!("Device not found after setting");
            }
            Err(e) => {
                println!("Failed to get device: {}", e);
                panic!("Failed to get device: {}", e);
            }
        }
        teardown()
    }

    #[test]
    #[serial]
    fn test_remove() {
        let manager = DeviceManager::new();
        let device = Device::new("test".to_string(), None, None, None);
        manager.add(device.clone()).unwrap();
        manager.remove("test").unwrap();
        assert_eq!(manager.get("test").unwrap(), None);
        teardown()
    }

    #[test]
    #[serial]
    fn test_get() {
        let manager = DeviceManager::new();
        let device = Device::new("test".to_string(), None, None, None);
        manager.add(device.clone()).unwrap();
        assert_eq!(manager.get("test").unwrap(), Some(device));
        teardown()
    }

    #[test]
    #[serial]
    fn test_get_all_devices() {
        let manager = DeviceManager::new();
        let device = Device::new("test".to_string(), None, None, None);
        manager.add(device.clone()).unwrap();
        assert_eq!(manager.get_all_devices().unwrap().len(), 1);
        teardown()
    }

    #[test]
    #[serial]
    fn test_get_all_devices_except_self() {
        let manager = DeviceManager::new();
        let device = Device::new("test".to_string(), None, None, None);
        manager.add(device.clone()).unwrap();
        manager.set_self_device(&device).unwrap();

        let devices = manager.get_all_devices_except_self().unwrap();
        assert_eq!(devices.len(), 0);
        teardown()
    }

    #[test]
    #[serial]
    fn test_get_device_by_ip_and_port() {
        let manager = DeviceManager::new();
        let device = Device::new("test".to_string(), None, None, None);
        manager.add(device.clone()).unwrap();
        assert_eq!(
            manager
                .get_device_by_ip_and_port("127.0.0.1", 12345)
                .unwrap(),
            None
        );

        let device = Device::new(
            "test1".to_string(),
            Some("127.0.0.1".to_string()),
            Some(12346),
            None,
        );
        manager.add(device.clone()).unwrap();
        assert_eq!(
            manager
                .get_device_by_ip_and_port("127.0.0.1", 12346)
                .unwrap(),
            Some(device)
        );
        teardown()
    }
}
