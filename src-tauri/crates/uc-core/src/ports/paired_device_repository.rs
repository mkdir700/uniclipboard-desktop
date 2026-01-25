use crate::network::{PairedDevice, PairingState};
use crate::PeerId;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use super::errors::PairedDeviceRepositoryError;

#[async_trait]
pub trait PairedDeviceRepositoryPort: Send + Sync {
    async fn get_by_peer_id(
        &self,
        peer_id: &PeerId,
    ) -> Result<Option<PairedDevice>, PairedDeviceRepositoryError>;

    async fn list_all(&self) -> Result<Vec<PairedDevice>, PairedDeviceRepositoryError>;

    async fn upsert(&self, device: PairedDevice) -> Result<(), PairedDeviceRepositoryError>;

    async fn set_state(
        &self,
        peer_id: &PeerId,
        state: PairingState,
    ) -> Result<(), PairedDeviceRepositoryError>;

    async fn update_last_seen(
        &self,
        peer_id: &PeerId,
        last_seen_at: DateTime<Utc>,
    ) -> Result<(), PairedDeviceRepositoryError>;

    async fn delete(&self, peer_id: &PeerId) -> Result<(), PairedDeviceRepositoryError>;
}
