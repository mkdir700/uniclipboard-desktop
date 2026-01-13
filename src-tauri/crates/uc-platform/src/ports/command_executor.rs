use anyhow::Result;
use async_trait::async_trait;

use crate::ipc::PlatformCommand;

#[async_trait]
pub trait PlatformCommandExecutorPort: Send + Sync {
    async fn execute(&self, command: PlatformCommand) -> Result<()>;
}
