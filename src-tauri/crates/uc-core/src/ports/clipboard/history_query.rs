use anyhow::Result;
use async_trait::async_trait;

use crate::clipboard::{ClipboardDecisionSnapshot, ContentHash};

/// Domain-facing port used only for copy-from-history decision making.
/// Must not expose mutation APIs.
#[async_trait]
pub trait ClipboardHistoryPort {
    async fn get_snapshot_decision(
        &self,
        hash: &ContentHash,
    ) -> Result<Option<ClipboardDecisionSnapshot>>;
}
