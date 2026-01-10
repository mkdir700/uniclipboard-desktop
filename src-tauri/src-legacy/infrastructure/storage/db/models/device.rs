use diesel::prelude::*;

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = crate::infrastructure::storage::db::schema::devices)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DbDevice {
    pub id: String,
    pub ip: Option<String>,
    pub port: Option<i32>,
    pub server_port: Option<i32>,
    pub status: i32,
    pub self_device: bool,
    pub updated_at: i32,
    pub alias: Option<String>,
    pub platform: Option<String>,
    /// libp2p PeerId for P2P networking
    pub peer_id: Option<String>,
    /// Human-readable device name
    pub device_name: Option<String>,
    /// Whether device has completed P2P pairing
    pub is_paired: bool,
    /// Timestamp of last contact (Unix timestamp)
    pub last_seen: Option<i32>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::infrastructure::storage::db::schema::devices)]
pub struct NewDevice<'a> {
    pub id: &'a str,
    pub ip: Option<&'a str>,
    pub port: Option<i32>,
    pub server_port: Option<i32>,
    pub status: i32,
    pub self_device: bool,
    pub updated_at: i32,
    pub alias: Option<&'a str>,
    pub platform: Option<&'a str>,
    pub peer_id: Option<&'a str>,
    pub device_name: Option<&'a str>,
    pub is_paired: bool,
    pub last_seen: Option<i32>,
}

#[derive(AsChangeset)]
#[diesel(table_name = crate::infrastructure::storage::db::schema::devices)]
pub struct UpdateDevice<'a> {
    pub ip: Option<&'a str>,
    pub port: Option<i32>,
    pub server_port: Option<i32>,
    pub status: i32,
    pub self_device: bool,
    pub updated_at: i32,
    pub alias: Option<&'a str>,
    pub platform: Option<&'a str>,
    pub peer_id: Option<&'a str>,
    pub device_name: Option<&'a str>,
    pub is_paired: Option<bool>,
    pub last_seen: Option<i32>,
}

impl<'a> From<&'a DbDevice> for NewDevice<'a> {
    fn from(device: &'a DbDevice) -> Self {
        NewDevice {
            id: &device.id,
            ip: device.ip.as_deref(),
            port: device.port,
            server_port: device.server_port,
            status: device.status,
            self_device: device.self_device,
            updated_at: device.updated_at,
            alias: device.alias.as_deref(),
            platform: device.platform.as_deref(),
            peer_id: device.peer_id.as_deref(),
            device_name: device.device_name.as_deref(),
            is_paired: device.is_paired,
            last_seen: device.last_seen,
        }
    }
}
