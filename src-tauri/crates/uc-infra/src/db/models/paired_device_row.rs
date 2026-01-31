use crate::db::schema::paired_device;
use diesel::prelude::*;

#[derive(Debug, Queryable)]
#[diesel(table_name = paired_device)]
pub struct PairedDeviceRow {
    pub peer_id: String,
    pub pairing_state: String,
    pub identity_fingerprint: String,
    pub paired_at: i64,
    pub last_seen_at: Option<i64>,
    pub device_name: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = paired_device)]
pub struct NewPairedDeviceRow {
    pub peer_id: String,
    pub pairing_state: String,
    pub identity_fingerprint: String,
    pub paired_at: i64,
    pub last_seen_at: Option<i64>,
    pub device_name: String,
}
