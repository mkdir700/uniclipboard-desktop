use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use uc_core::ports::{NetworkPort, SettingsPort};

const DEFAULT_PAIRING_DEVICE_NAME: &str = "Uniclipboard Device";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalDeviceInfo {
    pub peer_id: String,
    pub device_name: String,
}

pub struct GetLocalDeviceInfo {
    network: Arc<dyn NetworkPort>,
    settings: Arc<dyn SettingsPort>,
}

impl GetLocalDeviceInfo {
    pub fn new(network: Arc<dyn NetworkPort>, settings: Arc<dyn SettingsPort>) -> Self {
        Self { network, settings }
    }

    pub async fn execute(&self) -> Result<LocalDeviceInfo> {
        let device_name = match self.settings.load().await {
            Ok(settings) => {
                let name = settings.general.device_name.unwrap_or_default();
                if name.trim().is_empty() {
                    DEFAULT_PAIRING_DEVICE_NAME.to_string()
                } else {
                    name
                }
            }
            Err(err) => {
                tracing::warn!(error = %err, "Failed to load settings for pairing device name");
                DEFAULT_PAIRING_DEVICE_NAME.to_string()
            }
        };

        Ok(LocalDeviceInfo {
            peer_id: self.network.local_peer_id(),
            device_name,
        })
    }
}
