use crate::db::{
    models::{NewClipboardItemRow, NewClipboardRecordRow},
    ports::ClipboardDbPort,
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use log::warn;
use std::sync::Arc;
use uc_core::{
    clipboard::{
        ClipboardContent, ClipboardContentView, ClipboardDecisionSnapshot, ClipboardItemView,
        ContentHash, DuplicationHint, TimestampMs,
    },
    ports::{blob::port::ClipboardBlobPort, ClipboardHistoryPort, ClipboardRepositoryPort},
};

pub struct ClipboardRepository {
    db: Arc<dyn ClipboardDbPort>,
    blob: Arc<dyn ClipboardBlobPort>,
}

#[async_trait]
impl ClipboardRepositoryPort for ClipboardRepository {
    async fn save(&self, content: ClipboardContent) -> Result<()> {
        let hash = content.content_hash();
        if self.exists(&hash).await? {
            todo!()
        }
        let record_id = uuid::Uuid::new_v4().to_string();
        let record_row = NewClipboardRecordRow::from((&content, record_id.as_str()));

        self.db.insert_record(record_row)?;

        for (idx, item) in content.items.iter().enumerate() {
            let blob_id = self.blob.write(item.clone()).await?;

            let item_row =
                NewClipboardItemRow::from((item, record_id.as_str(), blob_id.as_str(), idx as i32));

            self.db.insert_item(item_row)?;
        }

        Ok(())
    }

    async fn duplication_hint(&self, content_hash: &ContentHash) -> Result<DuplicationHint> {
        let hash = content_hash.to_string();

        if self.db.record_exists(hash.clone())? {
            Ok(DuplicationHint::Repeated)
        } else {
            Ok(DuplicationHint::New)
        }
    }

    async fn exists(&self, content_hash: &ContentHash) -> Result<bool> {
        self.db.record_exists(content_hash.to_string())
    }

    async fn list_recent_views(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<ClipboardContentView>> {
        let record_rows = self.db.list_recent_records(limit, offset)?;
        let mut views = Vec::new();

        for record_row in record_rows {
            let item_rows = self.db.find_items_by_record(record_row.id.clone())?;
            let mut items = Vec::new();
            for item_row in item_rows {
                let item_view = ClipboardItemView {
                    mime: item_row.mime,
                    size: item_row.size.and_then(|v| v.try_into().ok()),
                };
                items.push(item_view);
            }

            let created_at = DateTime::<Utc>::from_timestamp_millis(record_row.created_at.clone())
                .ok_or_else(|| anyhow!("Invalid timestamp"))?;

            let content_view = ClipboardContentView {
                id: record_row.id.into(),
                source_device_id: record_row.source_device_id,
                origin: record_row.origin.into(),
                record_hash: record_row.record_hash,
                item_count: record_row.item_count,
                created_at,
                items,
            };
            views.push(content_view);
        }

        Ok(views)
    }

    async fn read(&self, content_hash: &ContentHash) -> Result<Option<ClipboardContent>> {
        let hash_val = content_hash.to_string();
        let record_row = self.db.find_record_by_hash(hash_val)?.ok_or_else(|| {
            anyhow::anyhow!("Clipboard content with hash {:?} not found", content_hash)
        })?;

        let item_rows = self.db.find_items_by_record(record_row.id)?;
        let mut items = Vec::new();
        for item_row in item_rows {
            let blob_id = match &item_row.blob_id {
                Some(id) => id,
                None => {
                    warn!("Clipboard item {} has no blob_id", item_row.id);
                    continue;
                }
            };
            let item = self.blob.read(blob_id).await?;
            items.push(item);
        }

        // 构建完整的 ClipboardContent

        let mut content = ClipboardContent {
            v: record_row.version as u32,
            occurred_at: TimestampMs::from_epoch_millis(record_row.occurred_at),
            items,
        };
        content.items = items;

        Ok(Some(content))
    }

    async fn soft_delete(&self, content_hash: &ContentHash) -> Result<()> {
        self.db.soft_delete_record(content_hash.to_string())
    }
}

#[async_trait]
impl ClipboardHistoryPort for ClipboardRepository {
    /// Determines whether all blobs referenced by the non-deleted clipboard record identified by `hash` exist.
    ///
    /// Checks the record (excluding soft-deleted records) and inspects each associated item to verify that a blob id is present and that the blob store contains the blob.
    ///
    /// # Returns
    ///
    /// `Some(ClipboardDecisionSnapshot)` where `blobs_exist` is `true` if every item references an existing blob and `false` if any item is missing a blob id or the blob does not exist; `None` if no matching non-deleted record is found.
    ///
    /// # Examples
    ///
    /// ```
    /// # use uc_infra::db::repositories::DieselClipboardRepository;
    /// # use uc_core::clipboard::ClipboardDecisionSnapshot;
    /// # async fn example(repo: &DieselClipboardRepository) -> Result<(), Box<dyn std::error::Error>> {
    /// let decision = repo.get_snapshot_decision("some-hash").await?;
    /// if let Some(ClipboardDecisionSnapshot { blobs_exist }) = decision {
    ///     println!("All blobs present: {}", blobs_exist);
    /// }
    /// # Ok(()) }
    /// ```
    async fn get_snapshot_decision(
        &self,
        hash: &ContentHash,
    ) -> Result<Option<ClipboardDecisionSnapshot>> {
        let record_row = self.db.find_record_by_hash(hash.to_string())?;

        let record = match record_row {
            Some(r) => r,
            None => return Ok(None),
        };

        let item_rows = self.db.find_items_by_record(record.id)?;

        // 检查所有 blob 是否存在以确定 can_read 权限
        let mut blobs_exist = true;
        for item_row in &item_rows {
            let blob_id = match &item_row.blob_id {
                Some(id) => id,
                None => {
                    warn!("Clipboard item {} has no blob_id", item_row.id);
                    blobs_exist = false;
                    break;
                }
            };
            if !self.blob.exists(blob_id).await? {
                warn!(
                    "Clipboard item {} references non-existent blob {}",
                    item_row.id, blob_id
                );
                blobs_exist = false;
                break;
            }
        }

        let decision_snapshot = ClipboardDecisionSnapshot { blobs_exist };

        Ok(Some(decision_snapshot))
    }
}
