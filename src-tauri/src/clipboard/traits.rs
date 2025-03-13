use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::message::Payload;

#[async_trait]
pub trait LocalClipboardTrait: Send + Sync {
    async fn pause(&self);
    async fn resume(&self);
    async fn read(&self) -> Result<Payload>;
    async fn write(&self, payload: Payload) -> Result<()>;
    async fn start_monitoring(&self) -> Result<mpsc::Receiver<Payload>>;
    async fn stop_monitoring(&self) -> Result<()>;
    async fn set_clipboard_content(&self, content: Payload) -> Result<()>;
}
