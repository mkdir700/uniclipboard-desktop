//! Disk spool manager for representation bytes.
//! 表示字节的磁盘缓存管理器。

use std::io;
use std::path::PathBuf;

use anyhow::{Context, Result};
use tokio::fs;
use uc_core::ids::RepresentationId;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Disk spool manager with size limits.
/// 具备容量上限的磁盘缓存管理器。
pub struct SpoolManager {
    spool_dir: PathBuf,
    max_bytes: usize,
}

/// Spool entry metadata.
/// 缓存条目元数据。
pub struct SpoolEntry {
    pub representation_id: RepresentationId,
    pub file_path: PathBuf,
    pub size: usize,
}

impl SpoolManager {
    /// Create a new spool manager and ensure directory exists.
    /// 创建新的磁盘缓存管理器并确保目录存在。
    pub fn new(spool_dir: impl Into<PathBuf>, max_bytes: usize) -> Result<Self> {
        let spool_dir = spool_dir.into();

        std::fs::create_dir_all(&spool_dir)
            .with_context(|| format!("Failed to create spool dir: {}", spool_dir.display()))?;

        let metadata = std::fs::metadata(&spool_dir).with_context(|| {
            format!("Failed to read spool dir metadata: {}", spool_dir.display())
        })?;
        if !metadata.is_dir() {
            return Err(anyhow::anyhow!(
                "Spool path is not a directory: {}",
                spool_dir.display()
            ));
        }

        #[cfg(unix)]
        {
            let perms = std::fs::Permissions::from_mode(0o700);
            std::fs::set_permissions(&spool_dir, perms).with_context(|| {
                format!(
                    "Failed to set spool dir permissions: {}",
                    spool_dir.display()
                )
            })?;
        }

        Ok(Self {
            spool_dir,
            max_bytes,
        })
    }

    /// Write bytes to spool, returning the entry metadata.
    /// 写入缓存并返回条目元数据。
    pub async fn write(&self, rep_id: &RepresentationId, bytes: &[u8]) -> Result<SpoolEntry> {
        let file_path = self.spool_dir.join(rep_id.to_string());

        fs::write(&file_path, bytes)
            .await
            .with_context(|| format!("Failed to write spool file: {}", file_path.display()))?;

        #[cfg(unix)]
        {
            let perms = std::fs::Permissions::from_mode(0o600);
            fs::set_permissions(&file_path, perms)
                .await
                .with_context(|| {
                    format!(
                        "Failed to set spool file permissions: {}",
                        file_path.display()
                    )
                })?;
        }

        Ok(SpoolEntry {
            representation_id: rep_id.clone(),
            file_path,
            size: bytes.len(),
        })
    }

    /// Read bytes from spool. Returns None if missing.
    /// 读取缓存字节，若不存在则返回 None。
    pub async fn read(&self, rep_id: &RepresentationId) -> Result<Option<Vec<u8>>> {
        let file_path = self.spool_dir.join(rep_id.to_string());
        match fs::read(&file_path).await {
            Ok(bytes) => Ok(Some(bytes)),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err)
                .with_context(|| format!("Failed to read spool file: {}", file_path.display())),
        }
    }

    /// Delete spool entry. Missing file is treated as success.
    /// 删除缓存条目，若不存在则视为成功。
    pub async fn delete(&self, rep_id: &RepresentationId) -> Result<()> {
        let file_path = self.spool_dir.join(rep_id.to_string());
        match fs::remove_file(&file_path).await {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err)
                .with_context(|| format!("Failed to delete spool file: {}", file_path.display())),
        }
    }

    /// Maximum bytes configured for the spool.
    /// 配置的最大字节数。
    pub fn max_bytes(&self) -> usize {
        self.max_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_spool_write_read() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let spool = SpoolManager::new(temp_dir.path(), 1_000_000)?;

        let rep_id = RepresentationId::new();
        let bytes = vec![1, 2, 3];

        spool.write(&rep_id, &bytes).await?;
        let retrieved = spool.read(&rep_id).await?;
        assert_eq!(retrieved, Some(bytes));
        Ok(())
    }

    #[tokio::test]
    async fn test_spool_delete_after_blob() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let spool = SpoolManager::new(temp_dir.path(), 1_000_000)?;

        let rep_id = RepresentationId::new();
        let bytes = vec![1, 2, 3];

        spool.write(&rep_id, &bytes).await?;
        spool.delete(&rep_id).await?;

        let retrieved = spool.read(&rep_id).await?;
        assert!(retrieved.is_none());
        Ok(())
    }
}
