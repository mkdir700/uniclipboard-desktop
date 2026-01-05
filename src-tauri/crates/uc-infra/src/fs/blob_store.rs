use anyhow::Result;
use async_trait::async_trait;
use tokio::fs;
use uc_core::ports::{BlobMeta, BlobStorePort};

const BLOBS_DIR: &str = "blobs";
const BLOB_META_FILE_NAME: &str = "meta.json";
const BLOB_DATA_FILE_NAME: &str = "data.bin";

pub struct FsBlobStore {
    root: std::path::PathBuf,
}

impl FsBlobStore {
    /// Create a new FsBlobStore rooted at the given filesystem path.
    ///
    /// The provided `root` path will be used as the base directory under which blobs
    /// are stored (e.g., `<root>/blobs/<blob_id>`).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// let store = uc_infra::fs::blob_store::FsBlobStore::new(PathBuf::from("/tmp/my_store"));
    /// ```
    pub fn new(root: std::path::PathBuf) -> Self {
        Self { root }
    }
}

fn validate_blob_id(blob_id: &str) -> Result<()> {
    uuid::Uuid::parse_str(blob_id)?;
    Ok(())
}

#[async_trait]
impl BlobStorePort for FsBlobStore {
    /// Stores a blob by creating a new UUID-named directory under the store root and persisting its metadata and data.
    ///
    /// Writes `meta.json` (JSON-serialized `BlobMeta`) and `data.bin` (raw bytes) into a new directory named by a generated UUID, and returns the generated blob id.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::PathBuf;
    /// # use futures::executor;
    /// # async fn _example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = FsBlobStore::new(PathBuf::from("/tmp"));
    /// let meta = BlobMeta { /* fill fields appropriately */ };
    /// let id = executor::block_on(store.create(meta, vec![1, 2, 3]))?;
    /// assert!(!id.is_empty());
    /// # Ok(()) }
    /// ```
    async fn create(&self, meta: BlobMeta, bytes: Vec<u8>) -> Result<String> {
        let blob_id = uuid::Uuid::new_v4().to_string();
        let dir = self.root.join(BLOBS_DIR).join(&blob_id);
        fs::create_dir_all(&dir).await?;

        let meta_path = dir.join(format!("{}", BLOB_META_FILE_NAME));
        fs::write(meta_path, serde_json::to_vec(&meta)?).await?;

        let data_path = dir.join(format!("{}", BLOB_DATA_FILE_NAME));
        fs::write(data_path, bytes).await?;

        Ok(blob_id)
    }

    /// Loads and deserializes the metadata for the specified blob from the store's filesystem.
    ///
    /// Returns the deserialized `BlobMeta` for the given blob ID wrapped in `Ok`, or an error if the
    /// metadata file cannot be read or parsed.
    ///
    /// # Examples
    ///
    /// ```
    /// use futures::executor::block_on;
    ///
    /// let store = FsBlobStore::new(std::path::PathBuf::from("/tmp"));
    /// let meta = block_on(store.read_meta("some-blob-id")).unwrap();
    /// // use `meta`
    /// ```
    async fn read_meta(&self, blob_id: &str) -> Result<BlobMeta> {
        validate_blob_id(blob_id)?;
        let path = self
            .root
            .join(BLOBS_DIR)
            .join(blob_id)
            .join(BLOB_META_FILE_NAME);

        let meta_bytes = fs::read(path).await?;
        let meta: BlobMeta = serde_json::from_slice(&meta_bytes)?;
        Ok(meta)
    }

    /// Reads the binary data for a blob identified by `blob_id`.
    ///
    /// # Returns
    ///
    /// The contents of the blob's data file as a `Vec<u8>`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::PathBuf;
    /// # use uc_infra::fs::blob_store::FsBlobStore;
    /// # use tokio::runtime::Runtime;
    /// let rt = Runtime::new().unwrap();
    /// let store = FsBlobStore::new(PathBuf::from("/tmp"));
    /// // `read_data` returns the raw bytes stored for the blob id.
    /// // This example assumes a blob with id "example" exists at `/tmp/blobs/example/data.bin`.
    /// let bytes = rt.block_on(async { store.read_data("example").await }).unwrap();
    /// assert!(!bytes.is_empty());
    /// ```
    async fn read_data(&self, blob_id: &str) -> Result<Vec<u8>> {
        validate_blob_id(blob_id)?;
        let path = self
            .root
            .join(BLOBS_DIR)
            .join(blob_id)
            .join(BLOB_DATA_FILE_NAME);

        Ok(fs::read(path).await?)
    }

    /// Removes the blob directory and all its contents for the specified blob ID from the store.
    ///
    /// The directory removed is `<root>/blobs/<blob_id>`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::path::PathBuf;
    /// # use futures::executor::block_on;
    /// # use uc_infra::fs::blob_store::FsBlobStore;
    /// let store = FsBlobStore::new(PathBuf::from("/tmp"));
    /// let _ = block_on(store.delete("example-blob-id"));
    /// ```
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an `Err` if the directory cannot be removed.
    async fn delete(&self, blob_id: &str) -> Result<()> {
        validate_blob_id(blob_id)?;
        let path = self.root.join(BLOBS_DIR).join(blob_id);
        fs::remove_dir_all(path).await?;
        Ok(())
    }
}