use anyhow::Result;
use async_trait::async_trait;
use diesel::prelude::*;
use log::warn;
use std::sync::Arc;
use uc_core::clipboard::ClipboardContent;
use uuid::Uuid;

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
use uc_core::ports::clipboard_repository::ClipboardRepositoryPort;
use uc_core::ports::BlobStorePort;

use log::{error, info};

pub struct DieselClipboardRepository {
    pool: DbPool,
    blob_store: Arc<dyn BlobStorePort>,
}

impl DieselClipboardRepository {
    /// 创建新的 repository 实例
    ///
    /// # 参数
    /// - `pool`: 数据库连接池
    /// - `storage_dir`: 文件存储目录
    /// - `device_id`: 当前设备 ID
    /// - `origin_val`: 来源标识 ("local" 或 "remote")
    pub fn new(pool: DbPool, blob_store: Arc<dyn BlobStorePort>) -> Result<Self> {
        Ok(Self { pool, blob_store })
    }
}

#[async_trait]
impl ClipboardRepositoryPort for DieselClipboardRepository {
    /// 保存剪切板快照
    ///
    /// 实现要点：
    /// 1. 先检查 content_hash 是否已存在（幂等性）
    /// 2. 使用事务确保原子性
    /// 3. 将每个 item 的数据存储到文件系统
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

    /// 检查 content_hash 是否已存在
    async fn exists(&self, hash_val: &str) -> Result<bool> {
        let mut conn = self.pool.get()?;

        let count = dsl_record::t_clipboard_record
            .filter(dsl_record::record_hash.eq(hash_val))
            .count()
            .get_result::<i64>(&mut conn)?;

        Ok(count > 0)
    }

    /// 获取最近的剪切板快照列表（仅元数据）
    ///
    /// 返回的 ClipboardContent.items 为空
    async fn list_recent(&self, limit: usize, offset: usize) -> Result<Vec<ClipboardContent>> {
        let mut conn = self.pool.get()?;

        let records = dsl_record::t_clipboard_record
            .filter(dsl_record::deleted_at.is_null())
            .order(dsl_record::created_at.desc())
            .limit(limit as i64)
            .offset(offset as i64)
            .load::<ClipboardRecordRow>(&mut conn)?;

        // 转换为 ClipboardContent（不包含 items 数据）
        let contents: Vec<ClipboardContent> = records
            .into_iter()
            .map(|row| map_record_row_to_content(&row))
            .collect();

        Ok(contents)
    }

    /// 根据 content_hash 获取完整剪切板快照
    ///
    /// 包含所有 items 的实际数据
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

    /// 软删除剪切板快照
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
