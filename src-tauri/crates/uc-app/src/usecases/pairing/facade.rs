use async_trait::async_trait;

use uc_core::network::pairing_state_machine::SessionId;
use uc_core::network::protocol::PairingChallengeResponse;

#[async_trait]
pub trait PairingFacade: Send + Sync {
    async fn initiate_pairing(&self, peer_id: String) -> anyhow::Result<SessionId>;
    async fn user_accept_pairing(&self, session_id: &str) -> anyhow::Result<()>;
    async fn user_reject_pairing(&self, session_id: &str) -> anyhow::Result<()>;
    async fn handle_challenge_response(
        &self,
        session_id: &str,
        peer_id: &str,
        response: PairingChallengeResponse,
    ) -> anyhow::Result<()>;
}
