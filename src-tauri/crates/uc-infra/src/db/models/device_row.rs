use crate::db::schema::t_device;
use diesel::prelude::*;

#[derive(Debug, Queryable)]
#[diesel(table_name = t_device)]
pub struct DeviceRow {
    pub id: String,
    pub name: String,
    pub platform: String,
    pub is_local: bool,
    pub created_at: i64,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = t_device)]
pub struct NewDeviceRow {
    pub id: String,
    pub name: String,
    pub platform: String,
    pub is_local: bool,
    pub created_at: i64,
}
