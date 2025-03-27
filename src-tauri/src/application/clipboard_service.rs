use anyhow::Result;
use std::sync::Arc;

use crate::core::transfer::ContentType;
use crate::infrastructure::storage::db::models::clipboard_record::{DbClipboardRecord, OrderBy};
use crate::message::Payload;
use crate::{application::file_service::FileService, core::UniClipboard};
use serde::{Deserialize, Serialize};

/// 文本摘要的最大长度
const MAX_TEXT_PREVIEW_LENGTH: usize = 1000;

#[derive(Serialize, Deserialize)]
pub struct ClipboardItemResponse {
    pub id: String,
    pub device_id: String,
    pub content_type: String,
    pub display_content: String,
    pub is_downloaded: bool,
    pub is_favorited: bool,
    pub created_at: i32,
    pub updated_at: i32,
    pub active_time: i32,
    pub content_size: usize,
    pub is_truncated: bool,
}

impl ClipboardItemResponse {
    /// 从数据库记录创建响应对象（带预览内容）
    pub fn from_record(record: DbClipboardRecord) -> Self {
        let content_type = match record.get_content_type() {
            Some(ct) => ct.as_str().to_string(),
            None => record.content_type.clone(),
        };

        let (display_content, content_size, is_truncated) = Self::process_content(&record, false);

        Self {
            id: record.id,
            device_id: record.device_id,
            content_type,
            display_content,
            is_downloaded: record.local_file_path.is_some(),
            is_favorited: record.is_favorited,
            created_at: record.created_at,
            updated_at: record.updated_at,
            active_time: record.active_time,
            content_size,
            is_truncated,
        }
    }

    /// 从数据库记录创建响应对象（带完整内容）
    pub fn from_record_full(record: DbClipboardRecord) -> Self {
        let content_type = match record.get_content_type() {
            Some(ct) => ct.as_str().to_string(),
            None => record.content_type.clone(),
        };

        let (display_content, content_size, is_truncated) = Self::process_content(&record, true);

        Self {
            id: record.id,
            device_id: record.device_id,
            content_type,
            display_content,
            is_downloaded: record.local_file_path.is_some(),
            is_favorited: record.is_favorited,
            created_at: record.created_at,
            updated_at: record.updated_at,
            active_time: record.active_time,
            content_size,
            is_truncated,
        }
    }

    /// 处理剪贴板内容
    fn process_content(record: &DbClipboardRecord, full_content: bool) -> (String, usize, bool) {
        if let Some(file_path) = &record.local_file_path {
            match record.get_content_type() {
                Some(ContentType::Text) => {
                    match FileService::read_text_file(
                        file_path,
                        if full_content {
                            None
                        } else {
                            Some(MAX_TEXT_PREVIEW_LENGTH)
                        },
                    ) {
                        Ok(result) => result,
                        Err(e) => (format!("无法读取文本内容: {}", e), 0, false),
                    }
                }
                Some(ContentType::Image) => {
                    match FileService::process_image_file(file_path, full_content) {
                        Ok(result) => result,
                        Err(e) => (format!("无法读取图片内容: {}", e), 0, false),
                    }
                }
                _ => (
                    format!("不支持的内容类型: {}", record.content_type),
                    0,
                    false,
                ),
            }
        } else if record.remote_record_id.is_some() {
            ("远程内容尚未下载".to_string(), 0, false)
        } else {
            ("无内容可显示".to_string(), 0, false)
        }
    }
}

pub struct ClipboardService {
    app: Arc<UniClipboard>,
}

impl ClipboardService {
    pub fn new(app: Arc<UniClipboard>) -> Self {
        Self { app }
    }

    /// 获取剪贴板历史记录
    pub async fn get_clipboard_items(
        &self,
        is_favorited: Option<bool>,
        order_by: Option<OrderBy>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<ClipboardItemResponse>> {
        let record_manager = self.app.get_record_manager();
        let records = record_manager
            .get_records(is_favorited, order_by, limit, offset)
            .await?;
        Ok(records
            .into_iter()
            .map(ClipboardItemResponse::from_record)
            .collect())
    }

    /// 获取单个剪贴板项目
    pub async fn get_clipboard_item(
        &self,
        id: &str,
        full_content: bool,
    ) -> Result<Option<ClipboardItemResponse>> {
        let record_manager = self.app.get_record_manager();
        let record = record_manager.get_record_by_id(id).await?;

        Ok(record.map(|r| {
            if full_content {
                ClipboardItemResponse::from_record_full(r)
            } else {
                ClipboardItemResponse::from_record(r)
            }
        }))
    }

    /// 删除指定的剪贴板项目
    pub async fn delete_clipboard_item(&self, id: &str) -> Result<bool> {
        let record_manager = self.app.get_record_manager();
        let file_storage = self.app.get_file_storage_manager();

        // 获取记录
        if let Some(record) = record_manager.get_record_by_id(id).await? {
            // 删除关联文件
            if let Some(path) = &record.local_file_path {
                if let Err(e) = file_storage.delete(&std::path::Path::new(&path)).await {
                    log::warn!("删除文件失败，但会继续删除记录: {}", e);
                }
            }

            // 删除记录
            record_manager.delete_record(id).await?;
        }

        Ok(true)
    }

    /// 清空所有剪贴板记录
    pub async fn clear_clipboard_items(&self) -> Result<usize> {
        let record_manager = self.app.get_record_manager();
        let file_storage = self.app.get_file_storage_manager();

        // 获取所有记录
        let records = record_manager.get_all_records().await?;

        // 删除所有关联文件
        for record in &records {
            if let Some(path) = &record.local_file_path {
                if let Err(e) = file_storage.delete(&std::path::Path::new(&path)).await {
                    log::warn!("删除文件失败: {}, 但会继续删除记录", e);
                }
            }
        }

        // 清空所有记录
        record_manager.clear_all_records().await
    }

    /// 复制剪贴板项目
    pub async fn copy_clipboard_item(&self, id: &str) -> Result<bool> {
        let record_manager = self.app.get_record_manager();
        let record = record_manager.get_record_by_id(id).await?;

        if let Some(record) = record {
            if let Err(e) = record_manager.update_record_active_time(id, None).await {
                log::warn!("更新活跃时间失败: {}", e);
            }

            // 将记录转换为 Payload
            let payload = Payload::try_from(record)?;

            // 写入本地剪贴板
            self.app.get_local_clipboard().write(payload).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// 收藏剪贴板内容
    pub async fn favorite_clipboard_item(&self, id: &str) -> Result<bool> {
        let record_manager = self.app.get_record_manager();
        record_manager.update_record_is_favorited(id, true).await?;
        Ok(true)
    }

    /// 取消收藏剪贴板内容
    pub async fn unfavorite_clipboard_item(&self, id: &str) -> Result<bool> {
        let record_manager = self.app.get_record_manager();
        record_manager.update_record_is_favorited(id, false).await?;
        Ok(true)
    }
}
