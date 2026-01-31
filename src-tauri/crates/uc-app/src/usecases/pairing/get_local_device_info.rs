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
                let trimmed = name.trim();
                if trimmed.is_empty() {
                    DEFAULT_PAIRING_DEVICE_NAME.to_string()
                } else {
                    trimmed.to_string()
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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use tokio::sync::mpsc;
    use uc_core::network::{
        ClipboardMessage, ConnectedPeer, DiscoveredPeer, NetworkEvent, PairingMessage,
    };
    use uc_core::settings::model::Settings;

    enum SettingsOutcome {
        Ok(Settings),
        Err(String),
    }

    struct TestSettings {
        outcome: SettingsOutcome,
    }

    #[async_trait]
    impl SettingsPort for TestSettings {
        async fn load(&self) -> anyhow::Result<Settings> {
            match &self.outcome {
                SettingsOutcome::Ok(settings) => Ok(settings.clone()),
                SettingsOutcome::Err(message) => Err(anyhow::anyhow!(message.clone())),
            }
        }

        async fn save(&self, _settings: &Settings) -> anyhow::Result<()> {
            Ok(())
        }
    }

    struct TestNetwork {
        peer_id: String,
    }

    #[async_trait]
    impl NetworkPort for TestNetwork {
        async fn send_clipboard(
            &self,
            _peer_id: &str,
            _encrypted_data: Vec<u8>,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        async fn broadcast_clipboard(&self, _encrypted_data: Vec<u8>) -> anyhow::Result<()> {
            Ok(())
        }

        async fn subscribe_clipboard(&self) -> anyhow::Result<mpsc::Receiver<ClipboardMessage>> {
            let (_tx, rx) = mpsc::channel(1);
            Ok(rx)
        }

        async fn get_discovered_peers(&self) -> anyhow::Result<Vec<DiscoveredPeer>> {
            Ok(Vec::new())
        }

        async fn get_connected_peers(&self) -> anyhow::Result<Vec<ConnectedPeer>> {
            Ok(Vec::new())
        }

        fn local_peer_id(&self) -> String {
            self.peer_id.clone()
        }

        async fn open_pairing_session(
            &self,
            _peer_id: String,
            _session_id: String,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        async fn send_pairing_on_session(
            &self,
            _session_id: String,
            _message: PairingMessage,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        async fn close_pairing_session(
            &self,
            _session_id: String,
            _reason: Option<String>,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        async fn unpair_device(&self, _peer_id: String) -> anyhow::Result<()> {
            Ok(())
        }

        async fn subscribe_events(&self) -> anyhow::Result<mpsc::Receiver<NetworkEvent>> {
            let (_tx, rx) = mpsc::channel(1);
            Ok(rx)
        }
    }

    #[tokio::test]
    async fn uses_device_name_from_settings() {
        let mut settings = Settings::default();
        settings.general.device_name = Some("Desk".to_string());

        let usecase = GetLocalDeviceInfo::new(
            Arc::new(TestNetwork {
                peer_id: "peer-1".to_string(),
            }),
            Arc::new(TestSettings {
                outcome: SettingsOutcome::Ok(settings),
            }),
        );

        let info = usecase.execute().await.expect("load device info");
        assert_eq!(info.peer_id, "peer-1");
        assert_eq!(info.device_name, "Desk");
    }

    #[tokio::test]
    async fn trims_device_name_from_settings() {
        let mut settings = Settings::default();
        settings.general.device_name = Some("  Desk  ".to_string());

        let usecase = GetLocalDeviceInfo::new(
            Arc::new(TestNetwork {
                peer_id: "peer-2".to_string(),
            }),
            Arc::new(TestSettings {
                outcome: SettingsOutcome::Ok(settings),
            }),
        );

        let info = usecase.execute().await.expect("load device info");
        assert_eq!(info.device_name, "Desk");
    }

    #[tokio::test]
    async fn uses_default_name_when_settings_missing_or_empty() {
        let mut settings = Settings::default();
        settings.general.device_name = Some("   ".to_string());

        let usecase = GetLocalDeviceInfo::new(
            Arc::new(TestNetwork {
                peer_id: "peer-3".to_string(),
            }),
            Arc::new(TestSettings {
                outcome: SettingsOutcome::Ok(settings),
            }),
        );

        let info = usecase.execute().await.expect("load device info");
        assert_eq!(info.device_name, DEFAULT_PAIRING_DEVICE_NAME);
    }

    #[tokio::test]
    async fn uses_default_name_when_settings_fail_to_load() {
        let usecase = GetLocalDeviceInfo::new(
            Arc::new(TestNetwork {
                peer_id: "peer-4".to_string(),
            }),
            Arc::new(TestSettings {
                outcome: SettingsOutcome::Err("load failed".to_string()),
            }),
        );

        let info = usecase.execute().await.expect("load device info");
        assert_eq!(info.device_name, DEFAULT_PAIRING_DEVICE_NAME);
    }
}
