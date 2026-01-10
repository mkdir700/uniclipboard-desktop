use crate::db::models::blob::NewBlobRow;
use crate::db::ports::Mapper;
use std::path::PathBuf;
use uc_core::blob::BlobStorageLocator;
use uc_core::Blob;

pub struct BlobRowMapper;

impl Mapper<Blob, NewBlobRow> for BlobRowMapper {
    fn to_row(&self, domain: &Blob) -> NewBlobRow {
        let (storage_backend, storage_path, encryption_algo) = map_locator(&domain.locator);

        NewBlobRow {
            blob_id: domain.blob_id.to_string(),
            storage_backend,
            storage_path,
            encryption_algo,
            size_bytes: domain.size_bytes,
            content_hash: domain.content_hash.to_string(),
            created_at_ms: domain.created_at_ms,
        }
    }
}

fn map_locator(locator: &BlobStorageLocator) -> (String, String, Option<String>) {
    match locator {
        BlobStorageLocator::LocalFs { path } => {
            ("local_fs".to_string(), path_to_string(path), None)
        }
        BlobStorageLocator::EncryptedFs { path, algo } => (
            "encrypted_fs".to_string(),
            path_to_string(path),
            Some(algo.to_string()),
        ),
    }
}

fn path_to_string(path: &PathBuf) -> String {
    path.to_str()
        .expect("blob storage path must be valid UTF-8")
        .to_owned()
}
