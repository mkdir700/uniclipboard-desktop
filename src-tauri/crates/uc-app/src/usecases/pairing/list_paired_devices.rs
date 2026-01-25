use anyhow::Result;
use std::sync::Arc;
use uc_core::network::PairedDevice;
use uc_core::ports::PairedDeviceRepositoryPort;

pub struct ListPairedDevices {
    repo: Arc<dyn PairedDeviceRepositoryPort>,
}

impl ListPairedDevices {
    pub fn new(repo: Arc<dyn PairedDeviceRepositoryPort>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self) -> Result<Vec<PairedDevice>> {
        self.repo
            .list_all()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list paired devices: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use uc_core::network::{PairedDevice, PairingState};
    use uc_core::ports::{PairedDeviceRepositoryError, PairedDeviceRepositoryPort};
    use uc_core::PeerId;

    struct MockPairedDeviceRepo {
        devices: Vec<PairedDevice>,
    }

    #[async_trait::async_trait]
    impl PairedDeviceRepositoryPort for MockPairedDeviceRepo {
        async fn get_by_peer_id(
            &self,
            _peer_id: &PeerId,
        ) -> Result<Option<PairedDevice>, PairedDeviceRepositoryError> {
            Ok(None)
        }

        async fn list_all(&self) -> Result<Vec<PairedDevice>, PairedDeviceRepositoryError> {
            Ok(self.devices.clone())
        }

        async fn upsert(&self, _device: PairedDevice) -> Result<(), PairedDeviceRepositoryError> {
            Ok(())
        }

        async fn set_state(
            &self,
            _peer_id: &PeerId,
            _state: PairingState,
        ) -> Result<(), PairedDeviceRepositoryError> {
            Ok(())
        }

        async fn update_last_seen(
            &self,
            _peer_id: &PeerId,
            _last_seen_at: chrono::DateTime<chrono::Utc>,
        ) -> Result<(), PairedDeviceRepositoryError> {
            Ok(())
        }

        async fn delete(&self, _peer_id: &PeerId) -> Result<(), PairedDeviceRepositoryError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_list_paired_devices_returns_devices() {
        let repo = MockPairedDeviceRepo {
            devices: vec![PairedDevice {
                peer_id: PeerId::from("peer-1"),
                pairing_state: PairingState::Trusted,
                identity_fingerprint: "fp".to_string(),
                paired_at: chrono::Utc::now(),
                last_seen_at: None,
            }],
        };

        let uc = ListPairedDevices::new(Arc::new(repo));
        let devices = uc.execute().await.unwrap();

        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].peer_id.as_str(), "peer-1");
    }
}
