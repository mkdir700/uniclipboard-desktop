use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use bytes::Bytes;
use chrono::{TimeZone, Utc};
use image::{GenericImageView, ImageFormat};
use std::fs;
use std::io::Cursor;
use std::path::Path;

use crate::core::content_type::ContentType;
use crate::infrastructure::storage::db::models::clipboard_record::DbClipboardRecord;
use crate::message::Payload;

/// 缩略图最大尺寸（像素）
const MAX_THUMBNAIL_SIZE: u32 = 1200;
/// 最大图片大小，超过此大小将压缩（字节）
const MAX_IMAGE_SIZE: usize = 1024 * 1024; // 1MB

pub struct ContentProcessorService;

impl ContentProcessorService {
    /// 根据图片文件路径获取MIME类型
    pub fn get_image_mime_type(path: &str) -> &'static str {
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
                _ => "application/octet-stream",
            }
        } else {
            "image/jpeg"
        }
    }

    /// 压缩图片到合适的尺寸
    pub fn resize_image(image_data: &[u8], file_path: &str) -> Result<Vec<u8>> {
        let img = image::load_from_memory(image_data)?;
        let (width, height) = img.dimensions();

        let (new_width, new_height) = if width > height && width > MAX_THUMBNAIL_SIZE {
            let ratio = MAX_THUMBNAIL_SIZE as f32 / width as f32;
            (MAX_THUMBNAIL_SIZE, (height as f32 * ratio) as u32)
        } else if height > MAX_THUMBNAIL_SIZE {
            let ratio = MAX_THUMBNAIL_SIZE as f32 / height as f32;
            ((width as f32 * ratio) as u32, MAX_THUMBNAIL_SIZE)
        } else {
            (width, height)
        };

        if new_width == width && new_height == height {
            return Ok(image_data.to_vec());
        }

        let resized = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);

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
                _ => None,
            })
            .unwrap_or(ImageFormat::Jpeg);

        let mut output = Cursor::new(Vec::new());
        resized.write_to(&mut output, format)?;

        Ok(output.into_inner())
    }

    /// 读取并处理图片文件
    pub fn process_image_file(
        file_path: &str,
        force_original: bool,
    ) -> Result<(String, usize, bool)> {
        let bytes = fs::read(file_path)?;
        let file_size = bytes.len();

        let mime_type = Self::get_image_mime_type(file_path);

        let size_threshold = if force_original {
            MAX_IMAGE_SIZE * 2
        } else {
            MAX_IMAGE_SIZE
        };
        let (processed_bytes, is_resized) = if file_size > size_threshold {
            match Self::resize_image(&bytes, file_path) {
                Ok(resized) => (resized, true),
                Err(_) => (bytes, false),
            }
        } else {
            (bytes, false)
        };

        let base64_content = BASE64.encode(&processed_bytes);
        Ok((
            format!("data:{};base64,{}", mime_type, base64_content),
            file_size,
            is_resized,
        ))
    }

    /// 读取文本文件
    pub fn read_text_file(
        file_path: &str,
        max_length: Option<usize>,
    ) -> Result<(String, usize, bool)> {
        let content = fs::read_to_string(file_path)?;
        let original_length = content.len();

        if let Some(max_len) = max_length {
            if original_length > max_len {
                let truncated_content = content.chars().take(max_len).collect::<String>();
                Ok((format!("{}...", truncated_content), original_length, true))
            } else {
                Ok((content, original_length, false))
            }
        } else {
            Ok((content, original_length, false))
        }
    }

    /// 处理链接文件
    pub fn process_link_file(
        file_path: &str,
        _full_content: bool,
    ) -> Result<(String, usize, bool)> {
        // 链接文件通常是文本文件，包含URL和可能的标题
        // 这里简单实现为读取文本内容
        Self::read_text_file(file_path, None)
    }

    /// 读取文件内容为 Bytes
    pub fn read_file_as_bytes(file_path: &str) -> Result<Bytes> {
        let content = fs::read(file_path)?;
        Ok(Bytes::from(content))
    }

    /// 读取图片文件并返回图片信息
    pub fn read_image_file(file_path: &str) -> Result<(Bytes, (u32, u32), String)> {
        let content = Self::read_file_as_bytes(file_path)?;
        let img = image::load_from_memory(&content)?;
        let dimensions = img.dimensions();

        // 从文件路径获取图片格式
        let format = Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("png")
            .to_string();

        Ok((content, dimensions, format))
    }

    /// 从数据库记录创建Payload
    pub fn create_payload_from_record(record: &DbClipboardRecord) -> Result<Payload> {
        // 确保有本地文件路径
        let file_path = record
            .local_file_path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No local file path"))?;

        // 转换时间戳
        let timestamp = Utc
            .timestamp_opt(record.created_at as i64, 0)
            .single()
            .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?;

        // 根据内容类型创建对应的 Payload
        match record.get_content_type() {
            Some(ContentType::Text) => {
                let content = Self::read_file_as_bytes(file_path)?;
                Ok(Payload::new_text(
                    content,
                    record.device_id.clone(),
                    timestamp,
                ))
            }
            Some(ContentType::Image) => {
                let (content, dimensions, format) = Self::read_image_file(file_path)?;
                let size = content.len();
                Ok(Payload::new_image(
                    content,
                    record.device_id.clone(),
                    timestamp,
                    dimensions.0 as usize,
                    dimensions.1 as usize,
                    format,
                    size,
                ))
            }
            _ => Err(anyhow::anyhow!(
                "Unsupported content type: {}",
                record.content_type
            )),
        }
    }
}
