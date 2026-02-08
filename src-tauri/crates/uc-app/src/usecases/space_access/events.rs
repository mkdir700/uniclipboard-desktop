use async_trait::async_trait;
use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceAccessCompletedEvent {
    pub session_id: String,
    pub peer_id: String,
    pub success: bool,
    pub reason: Option<String>,
    pub ts: i64,
}

#[async_trait]
pub trait SpaceAccessEventPort: Send + Sync {
    async fn subscribe(&self) -> anyhow::Result<mpsc::Receiver<SpaceAccessCompletedEvent>>;
}
