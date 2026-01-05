use crate::db::schema::t_device;
use diesel::prelude::*;

#[derive(Debug, Queryable, Insertable)]
#[diesel(table_name = t_device)]
pub struct DeviceRow {
    pub id: String,
    pub name: String,
    pub created_at: i64,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = t_device)]
pub struct NewDeviceRow<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub created_at: i64,
}
