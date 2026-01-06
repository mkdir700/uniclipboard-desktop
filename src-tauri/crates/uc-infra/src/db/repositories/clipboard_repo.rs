use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use log::warn;
use std::sync::Arc;
use uc_core::clipboard::{
    ClipboardContent, ClipboardContentView, ClipboardDecisionSnapshot, ClipboardItemView,
};
use uuid::Uuid;

/// Extension trait for converting i64 timestamps to DateTime<Utc>
trait TimestampExt {
    fn to_datetime(self) -> DateTime<Utc>;
}

impl TimestampExt for i64 {
    /// Converts an integer representing milliseconds since the Unix epoch into a `DateTime<Utc>`,
    /// falling back to `DateTime::UNIX_EPOCH` if the milliseconds value is out of range or invalid.
    ///
    /// # Returns
    ///
    /// A `DateTime<Utc>` corresponding to `self` milliseconds since the Unix epoch, or `DateTime::UNIX_EPOCH` if conversion fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::{DateTime, Utc};
    ///
    /// let dt = 0i64.to_datetime();
    /// assert_eq!(dt, DateTime::UNIX_EPOCH);
    ///
    /// let dt = 1_000i64.to_datetime(); // 1 second after epoch
    /// assert_eq!(dt.timestamp_millis(), 1_000);
    /// ```
    fn to_datetime(self) -> DateTime<Utc> {
        DateTime::from_timestamp_millis(self).unwrap_or_else(|| DateTime::UNIX_EPOCH)
    }
}

use crate::db::schema::t_clipboard_item::dsl as dsl_item;
use crate::db::schema::t_clipboard_record::dsl as dsl_record;
use crate::db::{
    mapper::clipboard_mapper::*,
    models::{
        ClipboardItemRow, ClipboardRecordRow, NewClipboardItemRow, NewClipboardItemRowOwned,
        NewClipboardRecordRow, NewClipboardRecordRowOwned,
    },
    pool::DbPool,
};
use crate::fs::clipboard_item_hydrator;
use uc_core::ports::{BlobStorePort, ClipboardHistoryPort, ClipboardRepositoryPort};

use log::{error, info};

pub struct DieselClipboardRepository {
    pool: DbPool,
    blob_store: Arc<dyn BlobStorePort>,
}

impl DieselClipboardRepository {
    /// Create a new `DieselClipboardRepository` with the given database pool and blob store.
    ///
    /// # Parameters
    ///
    /// - `pool`: Database connection pool used for repository operations.
    /// - `blob_store`: Shared blob store implementation for storing and retrieving item blobs.
    ///
    /// # Returns
    ///
    /// `Ok(Self)` with the constructed repository on success.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// // let pool: DbPool = ...;
    /// // let blob_store: Arc<dyn BlobStorePort> = Arc::new(...);
    /// // let repo = DieselClipboardRepository::new(pool, blob_store).unwrap();
    /// ```
    pub fn new(pool: DbPool, blob_store: Arc<dyn BlobStorePort>) -> Result<Self> {
        Ok(Self { pool, blob_store })
    }
}

#[async_trait]
impl ClipboardRepositoryPort for DieselClipboardRepository {
    /// Saves a clipboard snapshot and its items, storing item blobs in the configured blob store.
    ///
    /// Performs an idempotent insert by checking the content hash, writes the clipboard record inside
    /// a database transaction, stores each item's blob, and rolls back database changes if any blob
    /// storage fails.
    ///
    /// # Errors
    ///
    /// Returns an error if database operations or blob storage fail.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::sync::Arc;
    /// # use uc_infra::db::repositories::clipboard::DieselClipboardRepository;
    /// # use uc_infra::port::blob::BlobStorePort;
    /// # use uc_core::models::clipboard::ClipboardContent;
    /// # async fn example(repo: &DieselClipboardRepository, content: ClipboardContent) -> anyhow::Result<()> {
    /// repo.save(content).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn save(&self, content: ClipboardContent) -> Result<()> {
        let hash_val = content.content_hash();
        let mut conn = self.pool.get()?;

        // 幂等性检查：如果 content_hash 已存在，直接返回成功
        let exists = dsl_record::t_clipboard_record
            .filter(dsl_record::record_hash.eq(&hash_val))
            .first::<ClipboardRecordRow>(&mut conn)
            .optional()?;

        if exists.is_some() {
            log::debug!(
                "Clipboard content with hash {} already exists, skipping save",
                hash_val
            );
            return Ok(());
        }

        // 使用事务写入
        let record_id = conn.transaction::<_, diesel::result::Error, _>(|conn| {
            let record_row_owned: NewClipboardRecordRowOwned =
                NewClipboardRecordRowOwned::from(&content);

            let record_row: NewClipboardRecordRow<'_> = (&record_row_owned).into();

            diesel::insert_into(dsl_record::t_clipboard_record)
                .values(&record_row)
                .execute(conn)?;

            Ok(record_row_owned.id)
        })?;

        let mut err = None;

        for (index, item) in content.items.iter().enumerate() {
            let new_item_id = Uuid::new_v4().to_string();

            // TODO: 这里使用的同步编码函数，可能需要改为异步以避免阻塞
            let (blob_meta, data) = clipboard_item_hydrator::dehydrate(item)?;
            // 存储文件数据
            match self.blob_store.create(blob_meta, data).await {
                Ok(blob_id) => {
                    let mut item_row_owned: NewClipboardItemRowOwned =
                        NewClipboardItemRowOwned::from((
                            item,
                            record_id.as_str(),
                            blob_id.as_str(),
                            index as i32,
                        ));
                    item_row_owned.id = new_item_id.clone();

                    let item_row: NewClipboardItemRow<'_> = (&item_row_owned).into();

                    diesel::insert_into(dsl_item::t_clipboard_item)
                        .values(&item_row)
                        .execute(&mut conn)?;
                    info!(
                        "Stored clipboard item {} with blob_id {}",
                        new_item_id, blob_id
                    );
                }
                Err(e) => {
                    err = Some(e);
                    break;
                }
            }
        }

        if let Some(e) = err {
            error!(
                "Error storing clipboard items for record {}: {}",
                record_id, e
            );
            // 回滚已插入的记录
            diesel::delete(dsl_record::t_clipboard_record.filter(dsl_record::id.eq(&record_id)))
                .execute(&mut conn)?;
            diesel::delete(dsl_item::t_clipboard_item.filter(dsl_item::record_id.eq(&record_id)))
                .execute(&mut conn)?;
            return Err(e);
        }

        log::info!("Saved clipboard content with hash: {}", hash_val);
        Ok(())
    }

    /// Checks whether a clipboard snapshot with the given content hash exists.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // Assume `repo` is an initialized `DieselClipboardRepository`.
    /// # async fn _example(repo: &crate::DieselClipboardRepository) {
    /// let found = repo.exists("some_content_hash").await.unwrap();
    /// assert!(found == true || found == false);
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// `true` if a record with the provided content hash exists, `false` otherwise.
    async fn exists(&self, hash_val: &str) -> Result<bool> {
        let mut conn = self.pool.get()?;

        let count = dsl_record::t_clipboard_record
            .filter(dsl_record::record_hash.eq(hash_val))
            .count()
            .get_result::<i64>(&mut conn)?;

        Ok(count > 0)
    }

    /// List recent clipboard content views ordered by newest first.
    ///
    /// Returns a vector of `ClipboardContentView` containing record-level metadata and per-item
    /// view entries (mime and size), limited by `limit` and offset by `offset`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example(repo: &impl ClipboardRepositoryPort) -> Result<(), Box<dyn std::error::Error>> {
    /// let recent = repo.list_recent_views(10, 0).await?;
    /// assert!(recent.len() <= 10);
    /// # Ok(()) }
    /// ```
    async fn list_recent_views(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<ClipboardContentView>> {
        let mut conn = self.pool.get()?;

        // 查询 record
        let record_rows: Vec<ClipboardRecordRow> = dsl_record::t_clipboard_record
            .filter(dsl_record::deleted_at.is_null())
            .order(dsl_record::created_at.desc())
            .limit(limit as i64)
            .offset(offset as i64)
            .load(&mut conn)?;

        let mut views = Vec::new();

        for record in record_rows {
            // 查询关联的 items
            let item_rows = dsl_item::t_clipboard_item
                .filter(dsl_item::record_id.eq(&record.id))
                .order(dsl_item::index_in_record.asc())
                .load::<ClipboardItemRow>(&mut conn)?;

            let mut items = Vec::new();
            for item_row in item_rows {
                let mime = &item_row.mime;
                let item_view = ClipboardItemView {
                    mime: item_row.mime,
                    size: item_row.size as u64,
                };
                items.push(item_view);
            }

            let content_view = ClipboardContentView {
                id: record.id.into(),
                source_device_id: record.source_device_id,
                origin: record.origin.into(),
                record_hash: record.record_hash,
                item_count: record.item_count,
                created_at: record.created_at.to_datetime(),
                items,
            };
            views.push(content_view);
        }

        Ok(views)
    }

    /// Fetches a complete clipboard snapshot for the given content hash, including all items with hydrated data.   ///
    /// This returns the full ClipboardContent when a non-deleted record with the specified `hash_val` exists; otherwise returns `None`.
    ///
    /// # Parameters
    ///
    /// - `hash_val`: The content hash used to look up the clipboard record.
    ///
    /// # Returns
    ///
    /// `Some(ClipboardContent)` containing the record metadata and a vector of hydrated items when found, `None` if no matching non-deleted record exists.
    ///
    /// # Examples
    ///
    /// ```
    /// // Example usage (synchronous wrapper for the async method)
    /// # use futures::executor::block_on;
    /// # // `repo` should be a prepared DieselClipboardRepository instance
    /// # let repo = /* DieselClipboardRepository::new(...) */ unimplemented!();
    /// let result = block_on(repo.get_by_hash("some-content-hash"));
    /// // `result` is Ok(Some(content)) if found, Ok(None) if not
    /// ```
    async fn get_by_hash(&self, hash_val: &str) -> Result<Option<ClipboardContent>> {
        let mut conn = self.pool.get()?;

        // 查询 record
        let record_row = dsl_record::t_clipboard_record
            .filter(dsl_record::record_hash.eq(hash_val))
            .filter(dsl_record::deleted_at.is_null())
            .first::<ClipboardRecordRow>(&mut conn)
            .optional()?;

        let record = match record_row {
            Some(r) => r,
            None => return Ok(None),
        };

        // 查询关联的 items
        let item_rows = dsl_item::t_clipboard_item
            .filter(dsl_item::record_id.eq(&record.id))
            .order(dsl_item::index_in_record.asc())
            .load::<ClipboardItemRow>(&mut conn)?;

        // 加载每个 item 的数据并转换为 ClipboardItem
        let mut items = Vec::new();
        for item_row in item_rows {
            let blob_id = match &item_row.blob_id {
                Some(id) => id,
                None => {
                    warn!("Clipboard item {} has no blob_id, skipping", item_row.id);
                    continue;
                }
            };
            let blob_meta = self.blob_store.read_meta(blob_id).await?;
            let data = self.blob_store.read_data(blob_id).await?;
            let item = clipboard_item_hydrator::hydrate(blob_meta, data)?;
            items.push(item);
        }

        // 构建完整的 ClipboardContent
        let mut content = map_record_row_to_content(&record);
        content.items = items;

        Ok(Some(content))
    }

    /// Marks the clipboard snapshot identified by `hash_val` as deleted by setting its `deleted_at` timestamp to the current time.
    /// If no record matches `hash_val`, no changes are made.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // Async context where `repo` implements `ClipboardRepositoryPort`
    /// # async fn example(repo: &impl ClipboardRepositoryPort) -> anyhow::Result<()> {
    /// repo.soft_delete("content-hash-123").await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn soft_delete(&self, hash_val: &str) -> Result<()> {
        let mut conn = self.pool.get()?;

        let now = chrono::Utc::now().timestamp_millis();

        let affected = diesel::update(
            dsl_record::t_clipboard_record.filter(dsl_record::record_hash.eq(hash_val)),
        )
        .set(dsl_record::deleted_at.eq(now))
        .execute(&mut conn)?;

        if affected == 0 {
            log::warn!(
                "No record found with content_hash {} for soft delete",
                hash_val
            );
        } else {
            log::info!("Soft deleted clipboard content with hash: {}", hash_val);
        }

        Ok(())
    }
}

#[async_trait]
impl ClipboardHistoryPort for DieselClipboardRepository {
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
    async fn get_snapshot_decision(&self, hash: &str) -> Result<Option<ClipboardDecisionSnapshot>> {
        let mut conn = self.pool.get()?;

        // 查询 record
        let record_row = dsl_record::t_clipboard_record
            .filter(dsl_record::record_hash.eq(hash))
            .filter(dsl_record::deleted_at.is_null())
            .first::<ClipboardRecordRow>(&mut conn)
            .optional()?;

        let record = match record_row {
            Some(r) => r,
            None => return Ok(None),
        };

        // 查询关联的 items
        let item_rows = dsl_item::t_clipboard_item
            .filter(dsl_item::record_id.eq(&record.id))
            .order(dsl_item::index_in_record.asc())
            .load::<ClipboardItemRow>(&mut conn)?;

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
            if !self.blob_store.exists(blob_id).await? {
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