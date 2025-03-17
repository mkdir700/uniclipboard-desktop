// use crate::db::dao;
// use crate::db::DB_POOL;
// use crate::models::DbDevice;
use anyhow::Result;
use chrono::Utc;
use log::warn;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use tokio::sync::broadcast;

use crate::domain::device::{Device, DeviceStatus};
use crate::infrastructure::storage::db::dao;
use crate::infrastructure::storage::db::models::device::DbDevice;
use crate::infrastructure::storage::db::pool::DB_POOL;

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

impl DeviceManager {
    pub fn new() -> Self {
        Self {}
    }

    /// 注册本地设备
    pub fn register_self_device(&self, ip: String, webserver_port: u16) -> Result<String> {
        let device = Device::new_local_device(ip, webserver_port);
        let mut conn = DB_POOL.get_connection()?;
        let db_device = DbDevice::from(&device);
        if !dao::device::is_exist(&mut conn, &db_device.id)? {
            let db_device_clone = db_device.clone();
            self.add(db_device_clone.into())?;
            self.set_online(&db_device.id)?;
            Ok(db_device.id)
        } else {
            Err(anyhow::anyhow!("Device already exists"))
        }
    }

    // 获取当前设备的设备信息
    pub fn get_self_device(&self) -> Result<Option<Device>> {
        let mut conn = DB_POOL.get_connection()?;
        let db_device = dao::device::get_self_device(&mut conn)?;
        Ok(db_device.map(|d| d.into()))
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
