use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::fs;
use std::io::Cursor;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use image::{GenericImageView, ImageFormat};

use crate::infrastructure::storage::db::models::clipboard_record::{DbClipboardRecord, ClipboardItemMetadata};
use crate::core::transfer::ContentType;
use crate::core::UniClipboard;

/// 文本摘要的最大长度
const MAX_TEXT_PREVIEW_LENGTH: usize = 1000;
/// 缩略图最大尺寸（像素）
const MAX_THUMBNAIL_SIZE: u32 = 1200;
/// 最大图片大小，超过此大小将压缩（字节）
const MAX_IMAGE_SIZE: usize = 1024 * 1024; // 1MB

#[derive(Serialize, Deserialize)]
pub struct ClipboardItemResponse {
    pub id: String,
    pub device_id: String,
    pub content_type: String,
    pub display_content: String, // 显示内容
    pub is_downloaded: bool,
    pub is_favorited: bool,
    pub created_at: i32,
    pub updated_at: i32,
    pub content_size: usize,  // 内容大小（字节或字符数量）
    pub is_truncated: bool,   // 内容是否被截断
}

impl From<DbClipboardRecord> for ClipboardItemResponse {
    fn from(record: DbClipboardRecord) -> Self {
        // 获取内容类型
        let content_type = match record.get_content_type() {
            Some(ct) => ct.as_str().to_string(),
            None => record.content_type.clone(),
        };
        
        // 处理结果
        let (display_content, content_size, is_truncated) = if let Some(file_path) = &record.local_file_path {
            match record.get_content_type() {
                // 文本类型 - 直接读取文件内容
                Some(ContentType::Text) => {
                    match fs::read_to_string(file_path) {
                        Ok(content) => {
                            let original_length = content.len();
                            let truncated = original_length > MAX_TEXT_PREVIEW_LENGTH;
                            let display = if truncated {
                                // 如果文本太长，只显示前面的部分，并添加省略号
                                let truncated_content = content.chars()
                                    .take(MAX_TEXT_PREVIEW_LENGTH)
                                    .collect::<String>();
                                format!("{}...", truncated_content)
                            } else {
                                content
                            };
                            (display, original_length, truncated)
                        },
                        Err(e) => (format!("无法读取文本内容: {}", e), 0, false)
                    }
                },
                // 图片类型 - 转换为base64
                Some(ContentType::Image) => {
                    match fs::read(file_path) {
                        Ok(bytes) => {
                            let file_size = bytes.len();
                            
                            // 获取图片MIME类型
                            let mime_type = get_image_mime_type(file_path);
                            
                            // 如果图片太大，进行压缩处理
                            let (processed_bytes, is_resized) = if file_size > MAX_IMAGE_SIZE {
                                match resize_image(&bytes, file_path) {
                                    Ok(resized) => (resized, true),
                                    Err(_) => (bytes, false) // 压缩失败，使用原图
                                }
                            } else {
                                (bytes, false)
                            };
                            
                            // 将图片转换为base64字符串
                            let base64_content = BASE64.encode(&processed_bytes);
                            
                            // 创建data URI
                            (format!("data:{};base64,{}", mime_type, base64_content), file_size, is_resized)
                        },
                        Err(e) => (format!("无法读取图片内容: {}", e), 0, false)
                    }
                },
                _ => (format!("不支持的内容类型: {}", record.content_type), 0, false)
            }
        } else if record.remote_record_id.is_some() {
            // 远程记录，尚未下载
            ("远程内容尚未下载".to_string(), 0, false)
        } else {
            // 既没有本地路径也没有远程ID
            ("无内容可显示".to_string(), 0, false)
        };
        
        Self {
            id: record.id,
            device_id: record.device_id,
            content_type,
            display_content,
            is_downloaded: record.local_file_path.is_some(),
            is_favorited: record.is_favorited,
            created_at: record.created_at,
            updated_at: record.updated_at,
            content_size,
            is_truncated,
        }
    }
}

/// 根据图片文件路径获取MIME类型
fn get_image_mime_type(path: &str) -> &'static str {
    let path = Path::new(path);
    if let Some(extension) = path.extension() {
        match extension.to_str().unwrap_or("").to_lowercase().as_str() {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "svg" => "image/svg+xml",
            "bmp" => "image/bmp",
            "tiff" | "tif" => "image/tiff",
            "ico" => "image/x-icon",
            _ => "application/octet-stream" // 默认二进制流
        }
    } else {
        // 没有扩展名，尝试从文件头检测（这里简化处理）
        "image/jpeg" // 默认使用jpeg
    }
}

/// 压缩图片到合适的尺寸
fn resize_image(image_data: &[u8], file_path: &str) -> Result<Vec<u8>, anyhow::Error> {
    // 尝试解码图片
    let img = image::load_from_memory(image_data)?;
    
    // 获取原始尺寸
    let (width, height) = img.dimensions();
    
    // 计算新尺寸，保持宽高比
    let (new_width, new_height) = if width > height && width > MAX_THUMBNAIL_SIZE {
        let ratio = MAX_THUMBNAIL_SIZE as f32 / width as f32;
        (MAX_THUMBNAIL_SIZE, (height as f32 * ratio) as u32)
    } else if height > MAX_THUMBNAIL_SIZE {
        let ratio = MAX_THUMBNAIL_SIZE as f32 / height as f32;
        ((width as f32 * ratio) as u32, MAX_THUMBNAIL_SIZE)
    } else {
        // 图片尺寸已经小于阈值，不需要缩放
        (width, height)
    };
    
    // 如果不需要缩放，直接返回原图数据
    if new_width == width && new_height == height {
        return Ok(image_data.to_vec());
    }
    
    // 缩放图片
    let resized = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);
    
    // 获取图片格式
    let format = Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .and_then(|ext| match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
            "png" => Some(ImageFormat::Png),
            "gif" => Some(ImageFormat::Gif),
            "webp" => Some(ImageFormat::WebP),
            "bmp" => Some(ImageFormat::Bmp),
            "tiff" | "tif" => Some(ImageFormat::Tiff),
            _ => None
        })
        .unwrap_or(ImageFormat::Jpeg); // 默认使用JPEG
    
    // 保存到内存中
    let mut output = Cursor::new(Vec::new());
    
    resized.write_to(&mut output, ImageFormat::from(format))?;
    
    Ok(output.into_inner())
}

// 获取剪贴板历史记录
#[tauri::command]
pub async fn get_clipboard_items(
    state: tauri::State<'_, Arc<Mutex<Option<Arc<UniClipboard>>>>>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<ClipboardItemResponse>, String> {
    // 在作用域内获取锁，确保在await前释放
    let record_manager = {
        let app = state.lock().unwrap();
        if let Some(app) = app.as_ref() {
            app.get_record_manager()
        } else {
            return Err("应用未初始化".to_string());
        }
    };
    
    // 锁已释放，可以安全地使用await
    match record_manager.get_records(limit, offset).await {
        Ok(records) => Ok(records.into_iter().map(ClipboardItemResponse::from).collect()),
        Err(e) => Err(format!("获取剪贴板历史记录失败: {}", e)),
    }
}

// 删除指定ID的剪贴板记录
#[tauri::command]
pub async fn delete_clipboard_item(
    state: tauri::State<'_, Arc<Mutex<Option<Arc<UniClipboard>>>>>,
    id: String,
) -> Result<bool, String> {
    // 在作用域内获取锁，确保在await前释放
    let record_manager = {
        let app = state.lock().unwrap();
        if let Some(app) = app.as_ref() {
            app.get_record_manager()
        } else {
            return Err("应用未初始化".to_string());
        }
    };

    let record = record_manager.get_record_by_id(&id).await
        .map_err(|e| format!("获取剪贴板记录失败: {}", e))?;
    if record.is_none() {
        return Ok(true);
    }
    let record = record.unwrap();

    let file_storage = {
        let app = state.lock().unwrap();
        if let Some(app) = app.as_ref() {
            app.get_file_storage_manager()
        } else {
            return Err("应用未初始化".to_string());
        }
    };
    
    // 删除文件（如果存在）
    if let Some(path) = &record.local_file_path {
        match file_storage.delete(&Path::new(&path)).await {
            Ok(_) => {},
            Err(e) => {
                log::warn!("删除文件失败，但会继续删除记录: {}", e);
            }
        }
    }
    
    // 删除记录
    match record_manager.delete_record(&id).await {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("删除剪贴板记录失败: {}", e)),
    }
}

// 清空所有剪贴板历史记录
#[tauri::command]
pub async fn clear_clipboard_items(
    state: tauri::State<'_, Arc<Mutex<Option<Arc<UniClipboard>>>>>,
) -> Result<usize, String> {
    // 获取记录管理器和文件存储管理器
    let (record_manager, file_storage) = {
        let app = state.lock().unwrap();
        if let Some(app) = app.as_ref() {
            (app.get_record_manager(), app.get_file_storage_manager())
        } else {
            return Err("应用未初始化".to_string());
        }
    };
    
    // 先获取所有记录
    let records = match record_manager.get_all_records().await {
        Ok(records) => records,
        Err(e) => return Err(format!("获取剪贴板记录失败: {}", e)),
    };
    
    // 删除所有文件
    for record in &records {
        if let Some(path) = &record.local_file_path {
            match file_storage.delete(&Path::new(&path)).await {
                Ok(_) => {},
                Err(e) => {
                    log::warn!("删除文件失败: {}, 但会继续删除记录", e);
                }
            }
        }
    }
    
    // 清空所有记录
    match record_manager.clear_all_records().await {
        Ok(count) => Ok(count),
        Err(e) => Err(format!("清空剪贴板历史记录失败: {}", e)),
    }
}

// 获取单个剪贴板项目
#[tauri::command]
pub async fn get_clipboard_item(
    state: tauri::State<'_, Arc<Mutex<Option<Arc<UniClipboard>>>>>,
    id: String,
    full_content: Option<bool>,
) -> Result<Option<ClipboardItemResponse>, String> {
    // 在作用域内获取锁，确保在await前释放
    let record_manager = {
        let app = state.lock().unwrap();
        if let Some(app) = app.as_ref() {
            app.get_record_manager()
        } else {
            return Err("应用未初始化".to_string());
        }
    };
    
    // 锁已释放，可以安全地使用await
    match record_manager.get_record_by_id(&id).await {
        Ok(Some(record)) => {
            if full_content.unwrap_or(false) {
                Ok(Some(get_full_content_response(record)))
            } else {
                Ok(Some(ClipboardItemResponse::from(record)))
            }
        },
        Ok(None) => Ok(None),
        Err(e) => Err(format!("获取剪贴板记录失败: {}", e)),
    }
}

/// 获取带有完整内容的响应（不截断）
fn get_full_content_response(record: DbClipboardRecord) -> ClipboardItemResponse {
    // 获取内容类型
    let content_type = match record.get_content_type() {
        Some(ct) => ct.as_str().to_string(),
        None => record.content_type.clone(),
    };
    
    // 处理结果，不限制文本长度
    let (display_content, content_size, is_truncated) = if let Some(file_path) = &record.local_file_path {
        match record.get_content_type() {
            // 文本类型 - 直接读取文件内容（完整）
            Some(ContentType::Text) => {
                match fs::read_to_string(file_path) {
                    Ok(content) => {
                        let content_len = content.len();
                        (content, content_len, false)
                    },
                    Err(e) => (format!("无法读取文本内容: {}", e), 0, false)
                }
            },
            // 图片类型 - 处理方式同普通响应
            Some(ContentType::Image) => {
                match fs::read(file_path) {
                    Ok(bytes) => {
                        let file_size = bytes.len();
                        
                        // 获取图片MIME类型
                        let mime_type = get_image_mime_type(file_path);
                        
                        // 判断是否需要压缩（即使是完整内容请求，也要限制过大的图片）
                        let (processed_bytes, is_resized) = if file_size > MAX_IMAGE_SIZE * 2 {
                            // 对于完整内容请求，我们使用更大的阈值
                            match resize_image(&bytes, file_path) {
                                Ok(resized) => (resized, true),
                                Err(_) => (bytes, false) // 压缩失败，使用原图
                            }
                        } else {
                            (bytes, false)
                        };
                        
                        // 将图片转换为base64字符串
                        let base64_content = BASE64.encode(&processed_bytes);
                        
                        // 创建data URI
                        (format!("data:{};base64,{}", mime_type, base64_content), file_size, is_resized)
                    },
                    Err(e) => (format!("无法读取图片内容: {}", e), 0, false)
                }
            },
            _ => (format!("不支持的内容类型: {}", record.content_type), 0, false)
        }
    } else if record.remote_record_id.is_some() {
        // 远程记录，尚未下载
        ("远程内容尚未下载".to_string(), 0, false)
    } else {
        // 既没有本地路径也没有远程ID
        ("无内容可显示".to_string(), 0, false)
    };
    
    ClipboardItemResponse {
        id: record.id,
        device_id: record.device_id,
        content_type,
        display_content,
        is_downloaded: record.local_file_path.is_some(),
        is_favorited: record.is_favorited,
        created_at: record.created_at,
        updated_at: record.updated_at,
        content_size,
        is_truncated,
    }
}
