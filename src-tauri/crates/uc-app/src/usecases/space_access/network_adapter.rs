use uc_core::network::PairingMessage;
use uc_core::network::SessionId;
use uc_core::ports::space::SpaceAccessTransportPort;
use uc_core::ports::NetworkPort;

use super::context::SpaceAccessContext;

pub struct SpaceAccessNetworkAdapter<'a> {
    network: &'a dyn NetworkPort,
    ctx: &'a SpaceAccessContext,
}

#[async_trait::async_trait]
impl<'a> SpaceAccessTransportPort for SpaceAccessNetworkAdapter<'a> {
    async fn send_offer(&mut self, session_id: &SessionId) -> anyhow::Result<()> {
        let offer = self
            .ctx
            .offer
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("missing space access offer in context"))?;

        let msg = ProtocolMessage::Pairing(PairingMessage::SpaceAccessPayload(
            SpaceAccessCodec::encode_offer(offer)?,
        ));

        let bytes = msg.to_bytes()?;

        self.network
            .send_pairing_on_session(session_id.to_string(), bytes)
            .await
    }

    async fn send_proof(&mut self, session_id: &SessionId) -> anyhow::Result<()> {
        let proof = self
            .ctx
            .proof
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("missing space access proof in context"))?;

        let msg = ProtocolMessage::Pairing(PairingMessage::SpaceAccessPayload(
            SpaceAccessCodec::encode_proof(proof)?,
        ));

        let bytes = msg.to_bytes()?;

        self.network
            .send_pairing_on_session(session_id.to_string(), bytes)
            .await
    }

    async fn send_result(&mut self, session_id: &SessionId) -> anyhow::Result<()> {
        let result = self
            .ctx
            .result
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("missing space access result in context"))?;

        let msg = ProtocolMessage::Pairing(PairingMessage::SpaceAccessPayload(
            SpaceAccessCodec::encode_result(result)?,
        ));

        let bytes = msg.to_bytes()?;

        self.network
            .send_pairing_on_session(session_id.to_string(), bytes)
            .await
    }
}
