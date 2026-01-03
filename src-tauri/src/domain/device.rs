use crate::utils::helpers::{generate_device_id, get_current_platform};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::{self, Display};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DeviceStatus {
    Online = 0,
    Offline = 1,
    Unknown = 2,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
    Android,
    IOS,
    Browser,
    Unknown,
}

impl FromStr for Platform {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "windows" => Ok(Platform::Windows),
            "macos" => Ok(Platform::MacOS),
            "linux" => Ok(Platform::Linux),
            "android" => Ok(Platform::Android),
            "ios" => Ok(Platform::IOS),
            "browser" => Ok(Platform::Browser),
            _ => Ok(Platform::Unknown),
        }
    }
}

impl Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
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
    /// 设备平台
    pub platform: Option<Platform>,
    /// 设备别名
    pub alias: Option<String>,
    /// libp2p PeerId for P2P networking
    pub peer_id: Option<String>,
    /// Human-readable device name
    pub device_name: Option<String>,
    /// Whether device has completed P2P pairing
    pub is_paired: bool,
    /// Timestamp of last contact (Unix timestamp)
    pub last_seen: Option<i32>,
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
            platform: None,
            alias: None,
            peer_id: None,
            device_name: None,
            is_paired: false,
            last_seen: None,
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
            platform: Some(
                Platform::from_str(&get_current_platform()).unwrap_or(Platform::Unknown),
            ),
            alias: None,
            peer_id: None,
            device_name: None,
            is_paired: false,
            last_seen: None,
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
