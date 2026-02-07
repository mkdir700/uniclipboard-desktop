use crate::network::SessionId;

#[async_trait::async_trait]
pub trait SpaceAccessTransportPort: Send {
    async fn send_offer(&mut self, session_id: &SessionId) -> anyhow::Result<()>;
    async fn send_proof(&mut self, session_id: &SessionId) -> anyhow::Result<()>;
    async fn send_result(&mut self, session_id: &SessionId) -> anyhow::Result<()>;
}
