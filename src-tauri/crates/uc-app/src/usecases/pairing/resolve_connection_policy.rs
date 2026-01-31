use std::sync::Arc;
use uc_core::network::{ConnectionPolicy, PairingState, ResolvedConnectionPolicy};
use uc_core::ports::{
    ConnectionPolicyResolverError, ConnectionPolicyResolverPort, PairedDeviceRepositoryPort,
};
use uc_core::PeerId;

pub struct ResolveConnectionPolicy {
    repo: Arc<dyn PairedDeviceRepositoryPort>,
}

#[derive(Debug, thiserror::Error)]
pub enum ResolveConnectionPolicyError {
    #[error("repository error: {0}")]
    Repository(String),
}

impl ResolveConnectionPolicy {
    pub fn new(repo: Arc<dyn PairedDeviceRepositoryPort>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        peer_id: PeerId,
    ) -> Result<ResolvedConnectionPolicy, ResolveConnectionPolicyError> {
        let state = match self.repo.get_by_peer_id(&peer_id).await {
            Ok(Some(device)) => device.pairing_state,
            Ok(None) => PairingState::Pending,
            Err(err) => return Err(ResolveConnectionPolicyError::Repository(err.to_string())),
        };

        Ok(ResolvedConnectionPolicy {
            pairing_state: state.clone(),
            allowed: ConnectionPolicy::allowed_protocols(state),
        })
    }
}

#[async_trait::async_trait]
impl ConnectionPolicyResolverPort for ResolveConnectionPolicy {
    async fn resolve_for_peer(
        &self,
        peer_id: &PeerId,
    ) -> Result<ResolvedConnectionPolicy, ConnectionPolicyResolverError> {
        self.execute(peer_id.clone())
            .await
            .map_err(|err| ConnectionPolicyResolverError::Repository(err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use uc_core::network::{PairedDevice, PairingState, ProtocolKind};
    use uc_core::ports::{PairedDeviceRepositoryError, PairedDeviceRepositoryPort};

    struct MockRepo {
        state: Option<PairingState>,
        should_fail: bool,
    }

    impl MockRepo {
        fn new(state: Option<PairingState>) -> Self {
            Self {
                state,
                should_fail: false,
            }
        }

        fn failing() -> Self {
            Self {
                state: None,
                should_fail: true,
            }
        }
    }

    #[async_trait::async_trait]
    impl PairedDeviceRepositoryPort for MockRepo {
        async fn get_by_peer_id(
            &self,
            peer_id: &PeerId,
        ) -> Result<Option<PairedDevice>, PairedDeviceRepositoryError> {
            if self.should_fail {
                return Err(PairedDeviceRepositoryError::Storage(
                    "repo failure".to_string(),
                ));
            }

            Ok(self.state.clone().map(|state| PairedDevice {
                peer_id: peer_id.clone(),
                pairing_state: state,
                identity_fingerprint: "fp".to_string(),
                paired_at: chrono::Utc::now(),
                last_seen_at: None,
                device_name: "Mock Device".to_string(),
            }))
        }

        async fn list_all(&self) -> Result<Vec<PairedDevice>, PairedDeviceRepositoryError> {
            Ok(Vec::new())
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
    async fn unpaired_peer_allows_pairing_only() {
        let repo = Arc::new(MockRepo::new(None));
        let uc = ResolveConnectionPolicy::new(repo);
        let resolved = uc.execute(PeerId::from("peer-1")).await.unwrap();
        assert_eq!(resolved.pairing_state, PairingState::Pending);
        assert!(resolved.allowed.allows(ProtocolKind::Pairing));
        assert!(!resolved.allowed.allows(ProtocolKind::Business));
    }

    #[tokio::test]
    async fn trusted_peer_allows_business() {
        let repo = Arc::new(MockRepo::new(Some(PairingState::Trusted)));
        let uc = ResolveConnectionPolicy::new(repo);
        let resolved = uc.execute(PeerId::from("peer-1")).await.unwrap();
        assert_eq!(resolved.pairing_state, PairingState::Trusted);
        assert!(resolved.allowed.allows(ProtocolKind::Business));
    }

    #[tokio::test]
    async fn repo_failure_returns_error() {
        let repo = Arc::new(MockRepo::failing());
        let uc = ResolveConnectionPolicy::new(repo);
        let result = uc.execute(PeerId::from("peer-1")).await;
        assert!(result.is_err());
    }
}
