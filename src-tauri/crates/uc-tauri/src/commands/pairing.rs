//! Pairing-related Tauri commands
//! 配对相关的 Tauri 命令

use crate::bootstrap::AppRuntime;
use std::sync::Arc;
use tauri::State;
use tracing::{info_span, Instrument};
use uc_core::network::{PairedDevice, PairingState};
use uc_core::PeerId;

/// List paired devices
/// 列出已配对设备
#[tauri::command]
pub async fn list_paired_devices(
    runtime: State<'_, Arc<AppRuntime>>,
) -> Result<Vec<PairedDevice>, String> {
    let span = info_span!("command.pairing.list", device_id = %runtime.deps.device_identity.current_device_id());
    async {
        let uc = runtime.usecases().list_paired_devices();
        let devices = uc.execute().await.map_err(|e| {
            tracing::error!(error = %e, "Failed to list paired devices");
            e.to_string()
        })?;
        Ok(devices)
    }
    .instrument(span)
    .await
}

/// Update pairing state for a peer
/// 更新对等端配对状态
#[tauri::command]
pub async fn set_pairing_state(
    runtime: State<'_, Arc<AppRuntime>>,
    peer_id: String,
    state: PairingState,
) -> Result<(), String> {
    let span = info_span!("command.pairing.set_state", device_id = %runtime.deps.device_identity.current_device_id(), peer_id = %peer_id);
    async {
        let uc = runtime.usecases().set_pairing_state();
        uc.execute(PeerId::from(peer_id.as_str()), state)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to set pairing state");
                e.to_string()
            })?;
        Ok(())
    }
    .instrument(span)
    .await
}
