use crate::clipboard::SystemClipboardSnapshot;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait PlatformClipboardPort: Send + Sync {
    fn read_snapshot(&self) -> Result<SystemClipboardSnapshot>;
}
