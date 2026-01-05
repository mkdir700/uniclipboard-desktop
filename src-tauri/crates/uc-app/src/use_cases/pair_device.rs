//! PairDevice use case - handles device pairing flow

use anyhow::Result;
use log::info;
use std::sync::Arc;
use uc_core::{
    pairing::domain::PairingDomain,
    ports::{NetworkPort, StoragePort},
};

/// PairDevice use case - handles device pairing flow
pub struct PairDevice<N, S>
where
    N: NetworkPort,
    S: StoragePort,
{
    domain: PairingDomain,
    network: Arc<N>,
    storage: Arc<S>,
}

impl<N, S> PairDevice<N, S>
where
    N: NetworkPort,
    S: StoragePort,
{
    pub fn new(domain: PairingDomain, network: Arc<N>, storage: Arc<S>) -> Self {
        Self {
            domain,
            network,
            storage,
        }
    }

    /// Initiate pairing with a peer
    pub async fn initiate_pairing(&self, peer_id: String, device_name: String) -> Result<String> {
        info!("Initiating pairing with peer: {}", peer_id);
        self.network.initiate_pairing(peer_id, device_name).await
    }

    /// Verify PIN for pairing
    pub async fn verify_pin(&self, session_id: String, pin_matches: bool) -> Result<()> {
        info!("Verifying PIN for session: {}", session_id);
        self.network
            .send_pin_response(session_id, pin_matches)
            .await
    }

    /// Accept pairing request (responder side)
    pub async fn accept_pairing(&self, session_id: String) -> Result<()> {
        info!("Accepting pairing: session={}", session_id);
        self.network.accept_pairing(session_id).await
    }

    /// Reject pairing request
    pub async fn reject_pairing(&self, session_id: String, peer_id: String) -> Result<()> {
        info!(
            "Rejecting pairing: session={}, peer={}",
            session_id, peer_id
        );
        self.network
            .send_pairing_rejection(session_id, peer_id)
            .await
    }

    /// Unpair a device
    pub async fn unpair_device(&self, peer_id: String) -> Result<()> {
        info!("Unpairing device: {}", peer_id);
        self.network.unpair_device(peer_id.clone()).await?;
        self.storage.remove_paired_device(&peer_id).await
    }
}
