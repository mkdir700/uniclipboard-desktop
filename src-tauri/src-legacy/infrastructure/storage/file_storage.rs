use anyhow::Result;
use bytes::Bytes;
use log::info;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tokio::fs::File as TokioFile;
use tokio::io::BufReader;
use tokio_util::io::ReaderStream;

use crate::config::get_config_dir;
use crate::message::Payload;

/// 文件存储管理器
///
/// 负责将剪贴板内容持久化到文件系统
#[derive(Clone)]
pub struct FileStorageManager {
    storage_dir: PathBuf,
}

impl FileStorageManager {
    /// 创建一个新的文件存储管理器
    pub fn new() -> Result<Self> {
        let storage_dir = get_config_dir()?.join("storage");

        // 确保存储目录存在
        if !storage_dir.exists() {
            fs::create_dir_all(&storage_dir)?;
        }

        Ok(Self { storage_dir })
    }

    /// 存储剪贴板内容到文件系统
    ///
    /// 返回存储路径
    pub async fn store(&self, payload: &Payload) -> Result<PathBuf> {
        match payload {
            Payload::Text(text) => self.store_text(&text.get_content(), payload.get_key()),
            Payload::Image(image) => {
                self.store_image(&image.get_content(), &image.format, payload.get_key())
            }
            Payload::File(file) => self.store_file(&file.get_file_paths(), payload.get_key()),
        }
    }

    /// 存储文本内容
    fn store_text(&self, content: &Bytes, key: String) -> Result<PathBuf> {
        let text_dir = self.storage_dir.join("text");
        if !text_dir.exists() {
            fs::create_dir_all(&text_dir)?;
        }

        let file_path = text_dir.join(format!("{}.txt", key));
        let mut file = File::create(&file_path)?;
        file.write_all(content)?;

        info!("Text content stored at: {:?}", file_path);
        Ok(file_path)
    }

    /// 存储图片内容
    fn store_image(&self, content: &Bytes, format: &str, key: String) -> Result<PathBuf> {
        let image_dir = self.storage_dir.join("image");
        if !image_dir.exists() {
            fs::create_dir_all(&image_dir)?;
        }

        let extension = match format.to_lowercase().as_str() {
            "png" => "png",
            "jpeg" | "jpg" => "jpg",
            "gif" => "gif",
            "bmp" => "bmp",
            "webp" => "webp",
            _ => "bin", // 默认二进制格式
        };

        let file_path = image_dir.join(format!("{}.{}", key, extension));
        let mut file = File::create(&file_path)?;
        file.write_all(content)?;

        info!("Image content stored at: {:?}", file_path);
        Ok(file_path)
    }

    /// 存储文件内容
    ///
    /// 实际存储的是文件路径
    /// 文件路径是绝对路径
    ///
    /// 参数:
    /// - file_path: 文件路径
    /// - key: 文件唯一标识
    ///
    /// 返回:
    /// - 存储路径
    fn store_file(&self, file_paths: &Vec<String>, key: String) -> Result<PathBuf> {
        let file_dir = self.storage_dir.join("file");
        if !file_dir.exists() {
            fs::create_dir_all(&file_dir)?;
        }

        let local_file_path = file_dir.join(key);
        let mut file = File::create(&local_file_path)?;
        file.write_all(file_paths.join("\n").as_bytes())?;

        info!("File content stored at: {:?}", local_file_path);
        Ok(local_file_path)
    }

    /// 读取文件内容
    pub async fn read(&self, path: &Path) -> Result<Bytes> {
        let content = fs::read(path)?;
        Ok(Bytes::from(content))
    }

    /// 删除文件
    pub async fn delete(&self, path: &Path) -> Result<()> {
        if path.exists() {
            fs::remove_file(path)?;
            info!("File deleted: {:?}", path);
        }
        Ok(())
    }

    /// 创建文件流用于下载
    ///
    /// 返回文件流和文件大小
    pub async fn create_stream(
        &self,
        path: &Path,
    ) -> Result<(ReaderStream<BufReader<TokioFile>>, u64)> {
        // 打开文件
        let file = TokioFile::open(path).await?;

        // 获取文件大小
        let metadata = file.metadata().await?;
        let file_size = metadata.len();

        // 创建文件流
        let stream = ReaderStream::new(BufReader::new(file));

        Ok((stream, file_size))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::Payload;
    use bytes::Bytes;
    use chrono::Utc;

    #[tokio::test]
    async fn test_store_and_read_text() {
        let manager = FileStorageManager::new().unwrap();

        let content = Bytes::from("Test content");
        let payload = Payload::new_text(content.clone(), "test-device".to_string(), Utc::now());

        let path = manager.store(&payload).await.unwrap();
        assert!(path.exists());

        let read_content = manager.read(&path).await.unwrap();
        assert_eq!(read_content, content);

        // 清理测试文件
        manager.delete(&path).await.unwrap();
        assert!(!path.exists());
    }
}
