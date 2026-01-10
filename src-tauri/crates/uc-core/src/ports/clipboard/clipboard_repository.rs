use anyhow::Result;
use async_trait::async_trait;

use crate::clipboard::{NewClipboardEntry, NewClipboardSelection};

#[async_trait]
pub trait ClipboardRepositoryPort: Send + Sync {
    async fn save_entry(entry: NewClipboardEntry, selection: NewClipboardSelection, snapshot_ref: &str) -> Result<()>;
}