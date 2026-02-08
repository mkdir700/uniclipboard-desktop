use std::sync::Arc;

use tokio::sync::Mutex;
use uc_core::network::{PairingBusy, PairingMessage, SessionId};
use uc_core::ports::space::SpaceAccessTransportPort;
use uc_core::ports::NetworkPort;

use super::context::SpaceAccessContext;

pub struct SpaceAccessNetworkAdapter {
    network: Arc<dyn NetworkPort>,
    context: Arc<Mutex<SpaceAccessContext>>,
}

impl SpaceAccessNetworkAdapter {
    pub fn new(network: Arc<dyn NetworkPort>, context: Arc<Mutex<SpaceAccessContext>>) -> Self {
        Self { network, context }
    }
}

#[async_trait::async_trait]
impl SpaceAccessTransportPort for SpaceAccessNetworkAdapter {
    async fn send_offer(&mut self, session_id: &SessionId) -> anyhow::Result<()> {
        let offer = {
            let context = self.context.lock().await;
            context
                .prepared_offer
                .clone()
                .ok_or_else(|| anyhow::anyhow!("missing prepared_offer in space access context"))?
        };

        let payload = serde_json::json!({
            "kind": "space_access_offer",
            "space_id": offer.space_id.as_str(),
            "nonce": offer.nonce,
            "keyslot": offer.keyslot,
        });
        let payload = serde_json::to_string(&payload)?;

        self.network
            .send_pairing_on_session(
                session_id.to_string(),
                PairingMessage::Busy(PairingBusy {
                    session_id: session_id.to_string(),
                    reason: Some(payload),
                }),
            )
            .await
    }

    async fn send_proof(&mut self, session_id: &SessionId) -> anyhow::Result<()> {
        let proof = {
            let context = self.context.lock().await;
            context
                .proof_artifact
                .clone()
                .ok_or_else(|| anyhow::anyhow!("missing proof_artifact in space access context"))?
        };

        let payload = serde_json::json!({
            "kind": "space_access_proof",
            "pairing_session_id": proof.pairing_session_id.as_str(),
            "space_id": proof.space_id.as_str(),
            "challenge_nonce": proof.challenge_nonce,
            "proof_bytes": proof.proof_bytes,
        });
        let payload = serde_json::to_string(&payload)?;

        self.network
            .send_pairing_on_session(
                session_id.to_string(),
                PairingMessage::Busy(PairingBusy {
                    session_id: session_id.to_string(),
                    reason: Some(payload),
                }),
            )
            .await
    }

    async fn send_result(&mut self, session_id: &SessionId) -> anyhow::Result<()> {
        let payload = {
            let context = self.context.lock().await;
            serde_json::json!({
                "kind": "space_access_result",
                "sponsor_peer_id": context.sponsor_peer_id.clone(),
                "prepared_offer_exists": context.prepared_offer.is_some(),
                "joiner_offer_exists": context.joiner_offer.is_some(),
                "proof_artifact_exists": context.proof_artifact.is_some(),
            })
        };
        let payload = serde_json::to_string(&payload)?;

        self.network
            .send_pairing_on_session(
                session_id.to_string(),
                PairingMessage::Busy(PairingBusy {
                    session_id: session_id.to_string(),
                    reason: Some(payload),
                }),
            )
            .await
    }
}
