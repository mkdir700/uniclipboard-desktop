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
        }
    }
}
