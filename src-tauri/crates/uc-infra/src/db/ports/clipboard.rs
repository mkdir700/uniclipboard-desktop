use anyhow::Result;
use async_trait::async_trait;

use crate::db::models::{
    ClipboardItemRow, ClipboardRecordRow, NewClipboardItemRow, NewClipboardRecordRow,
};

#[async_trait]
pub trait ClipboardDbPort: Send + Sync {
    fn insert_record(&self, row: NewClipboardRecordRow) -> Result<()>;
    fn insert_item(&self, row: NewClipboardItemRow) -> Result<()>;

    fn find_record_by_hash(&self, hash: String) -> Result<Option<ClipboardRecordRow>>;
    fn find_items_by_record(&self, record_id: String) -> Result<Vec<ClipboardItemRow>>;

    fn list_recent_records(&self, limit: usize, offset: usize) -> Result<Vec<ClipboardRecordRow>>;

    fn record_exists(&self, hash: String) -> Result<bool>;
    fn soft_delete_record(&self, hash: String) -> Result<()>;
}
