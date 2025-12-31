use anyhow::Result;
use std::sync::Arc;

use crate::domain::content_type::ContentType;
use crate::infrastructure::storage::db::models::clipboard_record::{
    DbClipboardRecord, Filter, OrderBy,
};
use crate::infrastructure::storage::record_manager::ClipboardStats;
use crate::message::Payload;
use crate::{
    application::file_service::ContentProcessorService,
    infrastructure::uniclipboard::ClipboardSyncService,
};
use serde::{Deserialize, Serialize};

/// 文本摘要的最大长度
const MAX_TEXT_PREVIEW_LENGTH: usize = 1000;

#[derive(Serialize, Deserialize)]
pub struct TextItem {
    pub display_text: String,
    pub is_truncated: bool,
    pub size: usize,
}

#[derive(Serialize, Deserialize)]
pub struct ImageItem {
    pub thumbnail: String,
    pub size: usize,
    pub width: usize,
    pub height: usize,
}

#[derive(Serialize, Deserialize)]
pub struct FileItem {
    pub file_names: Vec<String>,
    pub file_sizes: Vec<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct LinkItem {
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct CodeItem {
    pub code: String,
}

#[derive(Serialize, Deserialize)]
pub enum ClipboardItem {
    #[serde(rename = "text")]
    Text(TextItem),
    #[serde(rename = "link")]
    Link(LinkItem),
    #[serde(rename = "code")]
    Code(CodeItem),
    #[serde(rename = "image")]
    Image(ImageItem),
    #[serde(rename = "file")]
    File(FileItem),
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Serialize, Deserialize)]
pub struct ClipboardItemResponse {
    pub id: String,
    pub device_id: String,
    pub is_downloaded: bool,
    pub is_favorited: bool,
    pub created_at: i32,
    pub updated_at: i32,
    pub active_time: i32,
    pub item: ClipboardItem,
}

/// 处理剪贴板内容，提取显示内容和相关元数据
pub fn process_clipboard_content(
    record: &DbClipboardRecord,
    full_content: bool,
) -> (String, usize, bool) {
    if let Some(local_file_path) = &record.local_file_path {
        match record.get_content_type() {
            Some(ContentType::Text) => {
                match ContentProcessorService::process_text_file(
                    local_file_path,
                    Some(MAX_TEXT_PREVIEW_LENGTH),
                ) {
                    Ok(result) => result,
                    Err(e) => (format!("无法读取文本内容: {}", e), 0, false),
                }
            }
            Some(ContentType::Image) => {
                match ContentProcessorService::process_image_file(local_file_path, full_content) {
                    Ok(result) => result,
                    Err(e) => (format!("无法读取图片内容: {}", e), 0, false),
                }
            }
            Some(ContentType::Link) => {
                // 使用 FileService::process_link_file 处理链接内容
                match ContentProcessorService::process_link_file(local_file_path, full_content) {
                    Ok(result) => result,
                    Err(e) => (format!("无法读取链接内容: {}", e), 0, false),
                }
            }
            Some(ContentType::CodeSnippet) => {
                match ContentProcessorService::process_text_file(local_file_path, None) {
                    Ok(result) => result,
                    Err(e) => (format!("无法读取代码片段内容: {}", e), 0, false),
                }
            }
            Some(ContentType::File) => {
                match ContentProcessorService::process_file(local_file_path) {
                    Ok(result) => result,
                    Err(e) => (format!("无法读取文件内容: {}", e), 0, false),
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

impl From<(DbClipboardRecord, bool)> for ClipboardItemResponse {
    fn from((record, full_content): (DbClipboardRecord, bool)) -> Self {
        let (display_content, content_size, is_truncated) =
            process_clipboard_content(&record, full_content);

        match record.get_content_type() {
            Some(ContentType::Text) | Some(ContentType::RichText) => {
                let item = ClipboardItem::Text(TextItem {
                    display_text: display_content,
                    is_truncated,
                    size: content_size,
                });
                ClipboardItemResponse {
                    id: record.id,
                    device_id: record.device_id,
                    is_downloaded: record.local_file_path.is_some(),
                    is_favorited: record.is_favorited,
                    created_at: record.created_at,
                    updated_at: record.updated_at,
                    active_time: record.active_time,
                    item,
                }
            }
            Some(ContentType::Link) => {
                let item = ClipboardItem::Link(LinkItem {
                    url: display_content,
                });
                ClipboardItemResponse {
                    id: record.id,
                    device_id: record.device_id,
                    is_downloaded: record.local_file_path.is_some(),
                    is_favorited: record.is_favorited,
                    created_at: record.created_at,
                    updated_at: record.updated_at,
                    active_time: record.active_time,
                    item,
                }
            }
            Some(ContentType::CodeSnippet) => {
                let item = ClipboardItem::Code(CodeItem {
                    code: display_content,
                });
                ClipboardItemResponse {
                    id: record.id,
                    device_id: record.device_id,
                    is_downloaded: record.local_file_path.is_some(),
                    is_favorited: record.is_favorited,
                    created_at: record.created_at,
                    updated_at: record.updated_at,
                    active_time: record.active_time,
                    item,
                }
            }
            Some(ContentType::Image) => {
                let (width, height) = record
                    .get_typed_extra_enhanced()
                    .and_then(|extra| extra.as_image().map(|img| (img.width, img.height)))
                    .unwrap_or((None, None));

                let item = ClipboardItem::Image(ImageItem {
                    thumbnail: display_content,
                    size: content_size,
                    width: width.unwrap_or(0) as usize,
                    height: height.unwrap_or(0) as usize,
                });
                ClipboardItemResponse {
                    id: record.id,
                    device_id: record.device_id,
                    is_downloaded: record.local_file_path.is_some(),
                    is_favorited: record.is_favorited,
                    created_at: record.created_at,
                    updated_at: record.updated_at,
                    active_time: record.active_time,
                    item,
                }
            }
            Some(ContentType::File) => {
                let is_downloaded = record.local_file_path.is_some();
                let (file_names, file_sizes) = record
                    .get_typed_extra_enhanced()
                    .and_then(|extra| {
                        extra
                            .as_file()
                            .map(|f| (f.file_names.clone(), f.file_sizes.clone()))
                    })
                    .unwrap_or((vec![], vec![]));

                ClipboardItemResponse {
                    id: record.id,
                    device_id: record.device_id,
                    is_downloaded,
                    is_favorited: record.is_favorited,
                    created_at: record.created_at,
                    updated_at: record.updated_at,
                    active_time: record.active_time,
                    item: ClipboardItem::File(FileItem {
                        file_names,
                        file_sizes,
                    }),
                }
            }
            None => ClipboardItemResponse {
                id: record.id,
                device_id: record.device_id,
                is_downloaded: record.local_file_path.is_some(),
                is_favorited: record.is_favorited,
                created_at: record.created_at,
                updated_at: record.updated_at,
                active_time: record.active_time,
                item: ClipboardItem::Unknown,
            },
        }
    }
}
pub struct ClipboardService {
    app: Arc<ClipboardSyncService>,
}

impl ClipboardService {
    pub fn new(app: Arc<ClipboardSyncService>) -> Self {
        Self { app }
    }

    /// 获取剪贴板统计信息
    pub async fn get_clipboard_stats(&self) -> Result<ClipboardStats> {
        let record_manager = self.app.get_record_manager();
        let stats = record_manager.get_stats().await?;
        Ok(stats)
    }

    /// 获取剪贴板历史记录
    pub async fn get_clipboard_items(
        &self,
        order_by: Option<OrderBy>,
        limit: Option<i64>,
        offset: Option<i64>,
        filter: Option<Filter>,
    ) -> Result<Vec<ClipboardItemResponse>> {
        let record_manager = self.app.get_record_manager();
        let records: Vec<DbClipboardRecord> = record_manager
            .get_records(order_by, limit, offset, filter)
            .await?;
        Ok(records
            .into_iter()
            .map(|r| ClipboardItemResponse::from((r, false)))
            .collect())
    }

    /// 获取单个剪贴板项目
    pub async fn get_clipboard_item(
        &self,
        id: &str,
        full_content: bool,
    ) -> Result<Option<ClipboardItemResponse>> {
        let record_manager = self.app.get_record_manager();
        let record: Option<DbClipboardRecord> = record_manager.get_record_by_id(id).await?;

        Ok(record.map(|r| ClipboardItemResponse::from((r, full_content))))
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
        log::info!("复制剪贴板项目: {}", id);
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
