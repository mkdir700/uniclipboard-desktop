use crate::models::device::{DbDevice, NewDevice, UpdateDevice};
use crate::schema::devices;
use anyhow::{Context, Result};
use diesel::dsl::exists;
use diesel::prelude::*;
use diesel::select;
use diesel::sqlite::SqliteConnection;

pub fn is_exist(conn: &mut SqliteConnection, id: &str) -> Result<bool> {
    select(exists(devices::table.find(id)))
        .get_result(conn)
        .context("Failed to check if device exists")
}

pub fn insert_device(conn: &mut SqliteConnection, device: &DbDevice) -> Result<()> {
    let new_device = NewDevice {
        id: &device.id,
        ip: device.ip.as_deref(),
        port: device.port,
        server_port: device.server_port,
        status: device.status,
        self_device: device.self_device,
        updated_at: device.updated_at,
    };

    diesel::insert_into(devices::table)
        .values(&new_device)
        .execute(conn)
        .context("Failed to insert device")?;

    Ok(())
}

pub fn batch_insert_devices(conn: &mut SqliteConnection, devices: &[DbDevice]) -> Result<()> {
    let new_devices: Vec<NewDevice> = devices.iter().map(NewDevice::from).collect();

    diesel::insert_into(devices::table)
        .values(&new_devices)
        .execute(conn)
        .context("Failed to batch insert devices")?;
    Ok(())
}

pub fn get_device(conn: &mut SqliteConnection, id: &str) -> Result<Option<DbDevice>> {
    devices::table
        .find(id)
        .first(conn)
        .optional()
        .context("Failed to get device")
}

pub fn get_device_by_ip_and_port(
    conn: &mut SqliteConnection,
    ip: &str,
    port: i32,
) -> Result<Option<DbDevice>> {
    devices::table
        .filter(devices::ip.eq(ip))
        .filter(devices::port.eq(port))
        .first(conn)
        .optional()
        .context("Failed to get device by ip and port")
}

pub fn update_device(conn: &mut SqliteConnection, device: &DbDevice) -> Result<()> {
    let update_device = UpdateDevice {
        ip: device.ip.as_deref(),
        port: device.port,
        server_port: device.server_port,
        status: device.status,
        self_device: device.self_device,
        updated_at: device.updated_at,
    };

    diesel::update(devices::table.find(&device.id))
        .set(&update_device)
        .execute(conn)
        .context("Failed to update device")?;
    Ok(())
}

pub fn get_all_devices(conn: &mut SqliteConnection) -> Result<Vec<DbDevice>> {
    devices::table
        .load::<DbDevice>(conn)
        .context("Failed to get all devices")
}

pub fn delete_device(conn: &mut SqliteConnection, id: &str) -> Result<()> {
    diesel::delete(devices::table.find(id))
        .execute(conn)
        .context("Failed to delete device")?;
    Ok(())
}

pub fn clear_devices(conn: &mut SqliteConnection) -> Result<()> {
    diesel::delete(devices::table)
        .execute(conn)
        .context("Failed to clear devices")?;
    Ok(())
}

pub fn update_device_status(
    conn: &mut SqliteConnection,
    id: &str,
    status: i32,
) -> Result<()> {
    diesel::update(devices::table.find(id))
        .set(devices::status.eq(status))
        .execute(conn)
        .context("Failed to set device offline")?;
    Ok(())
}

