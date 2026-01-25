use anyhow::Result;
use std::sync::Arc;
use uc_core::network::PairingState;
use uc_core::ports::PairedDeviceRepositoryPort;
use uc_core::PeerId;

pub struct SetPairingState {
    repo: Arc<dyn PairedDeviceRepositoryPort>,
}

impl SetPairingState {
    pub fn new(repo: Arc<dyn PairedDeviceRepositoryPort>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, peer_id: PeerId, state: PairingState) -> Result<()> {
        self.repo
            .set_state(&peer_id, state)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to set pairing state: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use uc_core::network::{PairedDevice, PairingState};
    use uc_core::ports::{PairedDeviceRepositoryError, PairedDeviceRepositoryPort};
    use uc_core::PeerId;

    #[derive(Default)]
    struct MockPairedDeviceRepo {
        states: Arc<Mutex<HashMap<String, PairingState>>>,
    }

    impl MockPairedDeviceRepo {
        async fn last_state(&self, peer_id: &str) -> Option<PairingState> {
            let guard = self.states.lock().await;
            guard.get(peer_id).cloned()
        }
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
            Ok(Vec::new())
        }

        async fn upsert(&self, _device: PairedDevice) -> Result<(), PairedDeviceRepositoryError> {
            Ok(())
        }

        async fn set_state(
            &self,
            peer_id: &PeerId,
            state: PairingState,
        ) -> Result<(), PairedDeviceRepositoryError> {
            let mut guard = self.states.lock().await;
            guard.insert(peer_id.as_str().to_string(), state);
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
    async fn test_set_pairing_state_updates_repo() {
        let repo = Arc::new(MockPairedDeviceRepo::default());
        let uc = SetPairingState::new(repo.clone());

        uc.execute(PeerId::from("peer"), PairingState::Trusted)
            .await
            .unwrap();

        let state = repo.last_state("peer").await;
        assert_eq!(state, Some(PairingState::Trusted));
    }
}
