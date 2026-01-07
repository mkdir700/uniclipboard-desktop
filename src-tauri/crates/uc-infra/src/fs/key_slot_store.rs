use async_trait::async_trait;
use std::path::PathBuf;
use uc_core::security::model::{EncryptionError, KeySlotFile};

#[async_trait]
pub trait KeySlotStore: Send + Sync {
    async fn load(&self) -> Result<KeySlotFile, EncryptionError>;
    async fn store(&self, slot: &KeySlotFile) -> Result<(), EncryptionError>;
    async fn delete(&self) -> Result<(), EncryptionError>;
}

pub struct JsonKeySlotStore {
    path: PathBuf,
}

impl JsonKeySlotStore {
    pub fn new(config_dir: PathBuf) -> Self {
        Self {
            path: config_dir.join("keyslot.json"),
        }
    }
}

#[async_trait]
impl KeySlotStore for JsonKeySlotStore {
    async fn load(&self) -> Result<KeySlotFile, EncryptionError> {
        if !self.path.exists() {
            return Err(EncryptionError::KeyNotFound);
        }

        let content = tokio::fs::read_to_string(&self.path)
            .await
            .map_err(|_| EncryptionError::IoFailure)?;

        let slot: KeySlotFile =
            serde_json::from_str(&content).map_err(|_| EncryptionError::KeyMaterialCorrupt)?;

        Ok(slot)
    }

    async fn store(&self, slot: &KeySlotFile) -> Result<(), EncryptionError> {
        let tmp = self.path.with_extension("json.tmp");

        let json =
            serde_json::to_string_pretty(slot).map_err(|_| EncryptionError::KeyMaterialCorrupt)?;

        tokio::fs::write(&tmp, json)
            .await
            .map_err(|_| EncryptionError::IoFailure)?;

        tokio::fs::rename(&tmp, &self.path)
            .await
            .map_err(|_| EncryptionError::IoFailure)?;

        Ok(())
    }

    async fn delete(&self) -> Result<(), EncryptionError> {
        if self.path.exists() {
            tokio::fs::remove_file(&self.path)
                .await
                .map_err(|_| EncryptionError::IoFailure)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uc_core::security::model::{
        AeadAlgorithm, EncryptionFormatVersion, KeyScope, KeySlotVersion,
    };

    fn make_temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("uc-keyslot-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn sample_keyslot_file() -> KeySlotFile {
        KeySlotFile {
            version: KeySlotVersion::V1,
            scope: KeyScope {
                profile_id: "test-profile".to_string(),
            },
            wrapped_master_key: uc_core::security::model::EncryptedBlob {
                version: EncryptionFormatVersion::V1,
                aead: AeadAlgorithm::XChaCha20Poly1305,
                nonce: vec![1u8; 24],
                ciphertext: vec![2u8; 32],
                aad_fingerprint: None,
            },
            created_at: None,
            updated_at: None,
        }
    }

    #[tokio::test]
    async fn load_missing_returns_key_not_found() {
        let dir = make_temp_dir();
        let store = JsonKeySlotStore::new(dir.clone());

        let err = store.load().await.expect_err("expected KeyNotFound");
        assert!(matches!(err, EncryptionError::KeyNotFound));

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[tokio::test]
    async fn store_then_load_round_trip() {
        let dir = make_temp_dir();
        let store = JsonKeySlotStore::new(dir.clone());
        let slot = sample_keyslot_file();

        store.store(&slot).await.expect("store keyslot");

        let loaded = store.load().await.expect("load keyslot");
        assert_eq!(loaded, slot);

        let tmp = dir.join("keyslot.json.tmp");
        assert!(!tmp.exists(), "tmp file should be removed after rename");

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[tokio::test]
    async fn load_corrupt_json_returns_key_material_corrupt() {
        let dir = make_temp_dir();
        let store = JsonKeySlotStore::new(dir.clone());
        let path = dir.join("keyslot.json");

        tokio::fs::write(&path, "not-json")
            .await
            .expect("write corrupt json");

        let err = store.load().await.expect_err("expected KeyMaterialCorrupt");
        assert!(matches!(err, EncryptionError::KeyMaterialCorrupt));

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[tokio::test]
    async fn delete_removes_file_and_is_idempotent() {
        let dir = make_temp_dir();
        let store = JsonKeySlotStore::new(dir.clone());
        let slot = sample_keyslot_file();

        store.store(&slot).await.expect("store keyslot");
        store.delete().await.expect("delete keyslot");
        store.delete().await.expect("delete keyslot again");

        let path = dir.join("keyslot.json");
        assert!(!path.exists(), "keyslot.json should be removed");

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }
}
