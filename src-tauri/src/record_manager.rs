use crate::db::dao::clipboard_record;
use anyhow::Result;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use log::{error, info};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::db::DB_POOL;
use crate::message::Payload;
use crate::models::clipboard_record::DbClipboardRecord;
use crate::schema::clipboard_records;

/// 剪贴板历史记录管理器
pub struct ClipboardRecordManager {
    max_records: usize,
}

impl ClipboardRecordManager {
    /// 创建一个新的剪贴板历史记录管理器
    pub fn new(max_records: usize) -> Self {
        Self { max_records }
    }

    /// 添加一条剪贴板记录
    pub async fn add_record(&self, payload: &Payload) -> Result<()> {
        let device_id = payload.get_device_id().to_string();
        let file_url = self.generate_file_url(payload);
        let content_type = payload.get_content_type().to_string();
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp() as i32;

        let record = DbClipboardRecord {
            id,
            device_id,
            remote_file_url: None,
            local_file_url: None,
            content_type,
            is_favorited: false,
            created_at: now,
            updated_at: now,
        };

        let mut conn = DB_POOL.get_connection()?;
        clipboard_record::insert_clipboard_record(&mut conn, &record)?;

        // 清理旧记录
        self.cleanup_old_records().await;

        Ok(())
    }

    /// 生成文件URL
    fn generate_file_url(&self, payload: &Payload) -> String {
        match payload {
            Payload::Text(_) => format!(
                "uniclipboard://{}/text/{}",
                payload.get_device_id(),
                payload.get_key()
            ),
            Payload::Image(_) => format!(
                "uniclipboard://{}/image/{}",
                payload.get_device_id(),
                payload.get_key()
            ),
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use chrono::Utc;

    #[tokio::test]
    async fn test_add_and_get_records() {
        // 初始化数据库
        DB_POOL.init().unwrap();

        let manager = ClipboardRecordManager::new(100);

        // 清空所有记录
        manager.clear_all_records().await.unwrap();

        // 创建测试数据
        let payload = Payload::new_text(
            Bytes::from("Test content"),
            "test-device".to_string(),
            Utc::now(),
        );

        // 添加记录
        manager.add_record(&payload).await.unwrap();

        // 获取记录
        let records = manager.get_records(Some(10), Some(0)).await.unwrap();

        assert!(!records.is_empty());
        assert_eq!(records[0].device_id, "test-device");
    }
}
