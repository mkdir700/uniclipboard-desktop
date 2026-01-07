use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::ipc::PlatformEvent;

#[async_trait]
pub trait ClipboardRuntimePort: Send + Sync {
    async fn start(&self, tx: mpsc::Sender<PlatformEvent>) -> Result<()>;
    async fn stop(&self) -> Result<()>;
}
