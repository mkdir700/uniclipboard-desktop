//! Blob Materializer Tests
//! Blob 物化器测试

use uc_core::Blob;
use uc_core::BlobId;
use uc_core::ContentHash;
use uc_core::ports::{BlobMaterializerPort, BlobStorePort, BlobRepositoryPort, ClockPort};
use uc_infra::blob::BlobMaterializer;
use uc_infra::SystemClock;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::fs;
use async_trait::async_trait;

/// Test blob store that uses temporary directory
#[derive(Clone)]
struct TestBlobStore {
    temp_dir: Arc<tempfile::TempDir>,
}

impl TestBlobStore {
    fn new() -> Self {
        Self {
            temp_dir: Arc::new(TempDir::new().unwrap()),
        }
    }
}

#[async_trait]
impl BlobStorePort for TestBlobStore {
    async fn put(&self, blob_id: &BlobId, data: &[u8]) -> anyhow::Result<std::path::PathBuf> {
        let path = self.temp_dir.path().join(blob_id.as_str());
        fs::write(&path, data).await?;
        Ok(path)
    }

    async fn get(&self, blob_id: &BlobId) -> anyhow::Result<Vec<u8>> {
        let path = self.temp_dir.path().join(blob_id.as_str());
        Ok(fs::read(&path).await?)
    }
}

/// Test blob repository that stores in memory
#[derive(Clone)]
struct TestBlobRepository {
    blob_ids_by_hash: Arc<std::sync::Mutex<std::collections::HashMap<String, BlobId>>>,
}

impl TestBlobRepository {
    fn new() -> Self {
        Self {
            blob_ids_by_hash: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }
}

#[async_trait]
impl BlobRepositoryPort for TestBlobRepository {
    async fn insert_blob(&self, blob: &Blob) -> anyhow::Result<()> {
        let mut blob_ids = self.blob_ids_by_hash.lock().unwrap();
        let hash_key = format!("{:?}", blob.content_hash);
        blob_ids.insert(hash_key, blob.blob_id.clone());
        Ok(())
    }

    async fn find_by_hash(&self, hash: &ContentHash) -> anyhow::Result<Option<Blob>> {
        let hash_key = format!("{:?}", hash);
        let blob_ids = self.blob_ids_by_hash.lock().unwrap();
        if let Some(blob_id) = blob_ids.get(&hash_key) {
            Ok(Some(Blob::new(
                blob_id.clone(),
                uc_core::blob::BlobStorageLocator::new_local_fs(std::path::PathBuf::from("/dummy")),
                0,
                hash.clone(),
                0,
            )))
        } else {
            Ok(None)
        }
    }
}

#[tokio::test]
async fn test_blob_materializer_creates_new_blob() {
    let blob_store = TestBlobStore::new();
    let blob_repo = TestBlobRepository::new();
    let clock = SystemClock;

    let materializer = BlobMaterializer::new(blob_store, blob_repo.clone(), clock);

    let data = b"Hello, Blob!";
    let hash = ContentHash::from(&[1u8; 32][..]);

    let result: Blob = materializer.materialize(data, &hash).await.unwrap();

    // Verify blob was created
    assert_eq!(result.size_bytes, 12); // "Hello, Blob!" = 12 bytes
    assert_eq!(result.content_hash, hash);
}

#[tokio::test]
async fn test_blob_materializer_deduplicates() {
    let blob_store = TestBlobStore::new();
    let blob_repo = TestBlobRepository::new();
    let clock = SystemClock;

    let materializer = BlobMaterializer::new(blob_store, blob_repo, clock);

    let data = b"Deduplicate me!";
    let hash = ContentHash::from(&[2u8; 32][..]);

    // First call should create blob
    let result1: Blob = materializer.materialize(data, &hash).await.unwrap();
    let blob_id1 = result1.blob_id.clone();

    // Second call with same hash should return existing blob
    let result2: Blob = materializer.materialize(data, &hash).await.unwrap();
    let blob_id2 = result2.blob_id.clone();

    assert_eq!(blob_id1, blob_id2, "Should return same blob for same content");
}

#[tokio::test]
async fn test_blob_materializer_stores_data() {
    let blob_store = TestBlobStore::new();
    let blob_repo = TestBlobRepository::new();
    let clock = SystemClock;

    let materializer = BlobMaterializer::new(blob_store.clone(), blob_repo, clock);

    let data = b"Persist this data";
    let hash = ContentHash::from(&[3u8; 32][..]);

    let result: Blob = materializer.materialize(data, &hash).await.unwrap();

    // Verify data can be retrieved from blob store
    let retrieved = blob_store.get(&result.blob_id).await.unwrap();
    assert_eq!(retrieved, data);
}

#[tokio::test]
async fn test_blob_materializer_different_hashes_different_blobs() {
    let blob_store = TestBlobStore::new();
    let blob_repo = TestBlobRepository::new();
    let clock = SystemClock;

    let materializer = BlobMaterializer::new(blob_store, blob_repo, clock);

    let data = b"Test data";
    let hash1 = ContentHash::from(&[4u8; 32][..]);
    let hash2 = ContentHash::from(&[5u8; 32][..]);

    let result1: Blob = materializer.materialize(data, &hash1).await.unwrap();
    let result2: Blob = materializer.materialize(data, &hash2).await.unwrap();

    assert_ne!(result1.blob_id, result2.blob_id, "Different hashes should create different blobs");
}
