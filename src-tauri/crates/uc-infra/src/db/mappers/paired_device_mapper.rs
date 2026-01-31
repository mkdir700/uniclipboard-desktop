use crate::db::models::{NewPairedDeviceRow, PairedDeviceRow};
use crate::db::ports::{InsertMapper, RowMapper};
use anyhow::{anyhow, Result};
use chrono::{TimeZone, Utc};
use uc_core::network::{PairedDevice, PairingState};
use uc_core::PeerId;

pub struct PairedDeviceRowMapper;

impl InsertMapper<PairedDevice, NewPairedDeviceRow> for PairedDeviceRowMapper {
    fn to_row(&self, domain: &PairedDevice) -> Result<NewPairedDeviceRow> {
        Ok(NewPairedDeviceRow {
            peer_id: domain.peer_id.as_str().to_string(),
            pairing_state: pairing_state_to_str(&domain.pairing_state).to_string(),
            identity_fingerprint: domain.identity_fingerprint.clone(),
            paired_at: domain.paired_at.timestamp(),
            last_seen_at: domain.last_seen_at.map(|dt| dt.timestamp()),
            device_name: domain.device_name.clone(),
        })
    }
}

impl RowMapper<PairedDeviceRow, PairedDevice> for PairedDeviceRowMapper {
    fn to_domain(&self, row: &PairedDeviceRow) -> Result<PairedDevice> {
        let paired_at = timestamp_to_utc(row.paired_at, "paired_at")?;
        let last_seen_at = match row.last_seen_at {
            Some(ts) => Some(timestamp_to_utc(ts, "last_seen_at")?),
            None => None,
        };

        Ok(PairedDevice {
            peer_id: PeerId::from(row.peer_id.as_str()),
            pairing_state: pairing_state_from_str(&row.pairing_state)?,
            identity_fingerprint: row.identity_fingerprint.clone(),
            paired_at,
            last_seen_at,
            device_name: row.device_name.clone(),
        })
    }
}

fn pairing_state_to_str(state: &PairingState) -> &'static str {
    match state {
        PairingState::Pending => "Pending",
        PairingState::Trusted => "Trusted",
        PairingState::Revoked => "Revoked",
    }
}

fn pairing_state_from_str(value: &str) -> Result<PairingState> {
    match value {
        "Pending" => Ok(PairingState::Pending),
        "Trusted" => Ok(PairingState::Trusted),
        "Revoked" => Ok(PairingState::Revoked),
        _ => Err(anyhow!("invalid pairing_state: {}", value)),
    }
}

fn timestamp_to_utc(ts: i64, field: &str) -> Result<chrono::DateTime<Utc>> {
    Utc.timestamp_opt(ts, 0)
        .single()
        .ok_or_else(|| anyhow!("invalid {} timestamp: {}", field, ts))
}
