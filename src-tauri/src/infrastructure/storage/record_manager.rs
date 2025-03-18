use crate::core::transfer::{ClipboardMetadata, ClipboardTransferMessage};
use crate::infrastructure::storage::db::schema::clipboard_records;
use anyhow::Result;
use chrono::Utc;
use diesel::prelude::*;
use log::{error, info};
use uuid::Uuid;

use super::db::dao::clipboard_record;
use super::db::models::clipboard_record::DbClipboardRecord;
use super::db::pool::DB_POOL;

/// 剪贴板历史记录管理器
#[derive(Clone)]
pub struct ClipboardRecordManager {
    max_records: usize,
}

impl ClipboardRecordManager {
    /// 创建一个新的剪贴板历史记录管理器
    pub fn new(max_records: usize) -> Self {
        Self { max_records }
    }

    /// 添加一条剪贴板记录
    ///
    /// # Arguments
    /// * `metadata` - 剪贴板内容元数据
    ///
    /// # Returns
    /// * `Result<String>` - 返回记录ID
    pub async fn add_record_with_metadata(&self, metadata: &ClipboardMetadata) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp() as i32;

        let record = DbClipboardRecord {
            id: id.clone(),
            device_id: metadata.get_device_id().to_string(),
            local_file_path: Some(metadata.get_storage_path().to_string()),
            remote_record_id: None,
            content_type: metadata.get_content_type().to_string(),
            is_favorited: false,
            created_at: now,
            updated_at: now,
        };

        let mut conn = DB_POOL.get_connection()?;
        clipboard_record::insert_clipboard_record(&mut conn, &record)?;

        // 清理旧记录
        self.cleanup_old_records().await;

        Ok(id)
    }

    pub async fn add_record_with_transfer_message(
        &self,
        message: &ClipboardTransferMessage,
    ) -> Result<String> {
        let content_type = message.metadata.get_content_type().to_string();
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp() as i32;

        let record = DbClipboardRecord {
            id: id.clone(),
            device_id: message.sender_id.clone(),
            local_file_path: None,
            remote_record_id: Some(message.record_id.clone()),
            content_type,
            is_favorited: false,
            created_at: now,
            updated_at: now,
        };

        let mut conn = DB_POOL.get_connection()?;
        clipboard_record::insert_clipboard_record(&mut conn, &record)?;

        Ok(id)
    }

    /// 获取历史记录列表
    pub async fn get_records(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<DbClipboardRecord>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);
        let mut conn = DB_POOL.get_connection()?;
        let records =
            clipboard_record::query_clipboard_records(&mut conn, Some(limit), Some(offset))?;
        Ok(records)
    }

    /// 获取所有的记录
    pub async fn get_all_records(
        &self
    ) -> Result<Vec<DbClipboardRecord>> {
        let mut conn = DB_POOL.get_connection()?;
        let records = clipboard_record::query_clipboard_records(&mut conn, None, None)?;
        Ok(records)
    }

    /// 获取指定ID的历史记录
    pub async fn get_record_by_id(&self, id: &str) -> Result<Option<DbClipboardRecord>> {
        let mut conn = DB_POOL.get_connection()?;
        let record = clipboard_record::get_clipboard_record_by_id(&mut conn, id)?;
        Ok(record)
    }

    /// 删除指定ID的历史记录
    pub async fn delete_record(&self, id: &str) -> Result<bool> {
        let mut conn = DB_POOL.get_connection()?;
        clipboard_record::delete_clipboard_record(&mut conn, id)?;
        Ok(true)
    }

    /// 清空所有历史记录
    pub async fn clear_all_records(&self) -> Result<usize> {
        let mut conn = DB_POOL.get_connection()?;
        let count = clipboard_record::clear_all_records(&mut conn)?;
        Ok(count)
    }

    /// 清理旧记录，保持记录数量不超过最大限制
    async fn cleanup_old_records(&self) {
        let max_records = self.max_records;
        tokio::spawn(async move {
            if let Err(e) = ClipboardRecordManager::do_cleanup_old_records(max_records).await {
                error!("Failed to cleanup old records: {:?}", e);
            }
        });
    }

    async fn do_cleanup_old_records(max_records: usize) -> Result<()> {
        let mut conn = DB_POOL.get_connection()?;
        // 获取当前记录总数
        let count: i64 = clipboard_record::get_record_count(&mut conn)?;

        if count <= max_records as i64 {
            return Ok(());
        }

        // 需要删除的记录数
        let to_delete = count - max_records as i64;

        // 获取需要删除的记录ID
        let ids: Vec<String> = clipboard_records::table
            .order_by(clipboard_records::created_at.asc())
            .limit(to_delete)
            .select(clipboard_records::id)
            .load(&mut conn)?;

        // 删除这些记录
        let deleted =
            diesel::delete(clipboard_records::table.filter(clipboard_records::id.eq_any(ids)))
                .execute(&mut conn)?;

        info!("Cleaned up {} old clipboard records", deleted);

        Ok(())
    }
}
