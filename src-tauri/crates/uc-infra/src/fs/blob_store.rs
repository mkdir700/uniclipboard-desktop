use anyhow::Result;
use async_trait::async_trait;
use std::fs;
use uc_core::ports::{BlobMeta, BlobStorePort};

const BLOBS_DIR: &str = "blobs";
const BLOB_META_FILE_NAME: &str = "meta.json";
const BLOB_DATA_FILE_NAME: &str = "data.bin";

pub struct FsBlobStore {
    root: std::path::PathBuf,
}

impl FsBlobStore {
    pub fn new(root: std::path::PathBuf) -> Self {
        Self { root }
    }
}

#[async_trait]
impl BlobStorePort for FsBlobStore {
    async fn create(&self, meta: BlobMeta, bytes: Vec<u8>) -> Result<String> {
        let blob_id = uuid::Uuid::new_v4().to_string();
        let dir = self.root.join(BLOBS_DIR).join(&blob_id);
        fs::create_dir_all(&dir)?;

        let meta_path = dir.join(format!("{}", BLOB_META_FILE_NAME));
        fs::write(meta_path, serde_json::to_vec(&meta)?)?;

        let data_path = dir.join(format!("{}", BLOB_DATA_FILE_NAME));
        fs::write(data_path, bytes)?;

        Ok(blob_id)
    }

    async fn read_meta(&self, blob_id: &str) -> Result<BlobMeta> {
        let path = self
            .root
            .join(BLOBS_DIR)
            .join(blob_id)
            .join(BLOB_META_FILE_NAME);

        let meta_bytes = fs::read(path)?;
        let meta: BlobMeta = serde_json::from_slice(&meta_bytes)?;
        Ok(meta)
    }

    async fn read_data(&self, blob_id: &str) -> Result<Vec<u8>> {
        let path = self
            .root
            .join(BLOBS_DIR)
            .join(blob_id)
            .join(BLOB_DATA_FILE_NAME);

        Ok(fs::read(path)?)
    }

    async fn delete(&self, blob_id: &str) -> Result<()> {
        let path = self.root.join(BLOBS_DIR).join(blob_id);
        fs::remove_dir_all(path)?;
        Ok(())
    }
}
