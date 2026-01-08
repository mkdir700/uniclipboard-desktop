use anyhow::Result;
use async_trait::async_trait;

use crate::db::models::{
    ClipboardItemRow, ClipboardRecordRow, NewClipboardItemRow, NewClipboardRecordRow,
};

#[async_trait]
pub trait ClipboardDbPort: Send + Sync {
    async fn insert_record(&self, row: NewClipboardRecordRow) -> Result<()>;
    async fn insert_item(&self, row: NewClipboardItemRow) -> Result<()>;

    async fn find_record_by_hash(&self, hash: String) -> Result<Option<ClipboardRecordRow>>;
    async fn find_items_by_record(&self, record_id: String) -> Result<Vec<ClipboardItemRow>>;

    async fn list_recent_records(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<ClipboardRecordRow>>;

    async fn record_exists(&self, hash: String) -> Result<bool>;
    async fn soft_delete_record(&self, hash: String) -> Result<()>;
}
