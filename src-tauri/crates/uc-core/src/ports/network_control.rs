//!
//! Network lifecycle control port.
//!
//! This port allows the application layer to request network startup
//! without depending on concrete network implementations.

use anyhow::Result;
use async_trait::async_trait;

/// Network control port - starts network runtime.
#[async_trait]
pub trait NetworkControlPort: Send + Sync {
    /// Start the network runtime.
    async fn start_network(&self) -> Result<()>;
}
