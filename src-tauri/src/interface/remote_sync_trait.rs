use crate::domain::transfer_message::ClipboardTransferMessage;
use anyhow::Result;
use async_trait::async_trait;
use std::time::Duration;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait RemoteClipboardSync: Send + Sync {
    async fn push(&self, message: ClipboardTransferMessage) -> Result<()>;
    async fn pull(&self, timeout: Option<Duration>) -> Result<ClipboardTransferMessage>;
    async fn sync(&self) -> Result<()>;
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
    async fn pause(&self) -> Result<()>;
    async fn resume(&self) -> Result<()>;
}
