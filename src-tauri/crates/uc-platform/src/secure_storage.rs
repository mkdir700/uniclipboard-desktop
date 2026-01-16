//! Secure storage selection and default keyring factory.

use std::{fs, path::PathBuf, sync::Arc};

use uc_core::ports::KeyringPort;

use crate::{
    capability::{detect_storage_capability, SecureStorageCapability},
    file_keyring::FileBasedKeyring,
    keyring::SystemKeyring,
};

#[derive(Debug, thiserror::Error)]
pub enum KeyringFactoryError {
    #[error("secure storage unsupported: {capability:?}")]
    Unsupported { capability: SecureStorageCapability },

    #[error("failed to initialize file-based keyring: {0}")]
    FileBasedInit(#[from] std::io::Error),
}

fn keyring_from_capability(
    capability: SecureStorageCapability,
) -> Result<Arc<dyn KeyringPort>, KeyringFactoryError> {
    keyring_from_capability_with_base_dir(capability, None)
}

fn keyring_from_capability_with_base_dir(
    capability: SecureStorageCapability,
    base_dir: Option<PathBuf>,
) -> Result<Arc<dyn KeyringPort>, KeyringFactoryError> {
    match capability {
        SecureStorageCapability::SystemKeyring => {
            Ok(Arc::new(SystemKeyring {}) as Arc<dyn KeyringPort>)
        }
        SecureStorageCapability::FileBasedKeystore => {
            if let Some(base_dir) = base_dir {
                fs::create_dir_all(&base_dir)?;
                Ok(Arc::new(FileBasedKeyring::with_base_dir(base_dir)) as Arc<dyn KeyringPort>)
            } else {
                Err(KeyringFactoryError::FileBasedInit(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "FileBasedKeyring requires app data root",
                )))
            }
        }
        SecureStorageCapability::Unsupported => {
            Err(KeyringFactoryError::Unsupported { capability })
        }
    }
}

pub fn create_default_keyring() -> Result<Arc<dyn KeyringPort>, KeyringFactoryError> {
    let capability = detect_storage_capability();
    log::debug!("Detected secure storage capability: {:?}", capability);

    match capability {
        SecureStorageCapability::SystemKeyring => {
            log::info!("Using system keyring for secure storage");
            keyring_from_capability(capability)
        }
        SecureStorageCapability::FileBasedKeystore => {
            log::warn!(
                "File-based keyring requires app data root; use create_default_keyring_in_app_data_root"
            );
            Err(KeyringFactoryError::FileBasedInit(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "FileBasedKeyring requires app data root",
            )))
        }
        SecureStorageCapability::Unsupported => {
            log::error!("Secure storage unsupported: {:?}", capability);
            Err(KeyringFactoryError::Unsupported { capability })
        }
    }
}

pub fn create_default_keyring_in_app_data_root(
    app_data_root: PathBuf,
) -> Result<Arc<dyn KeyringPort>, KeyringFactoryError> {
    let capability = detect_storage_capability();
    log::debug!("Detected secure storage capability: {:?}", capability);

    match capability {
        SecureStorageCapability::SystemKeyring => {
            log::info!("Using system keyring for secure storage");
            keyring_from_capability(capability)
        }
        SecureStorageCapability::FileBasedKeystore => {
            log::warn!(
                "Using file-based keyring (insecure dev fallback for WSL/headless environments)"
            );
            Ok(Arc::new(FileBasedKeyring::new_in_app_data_root(app_data_root)?)
                as Arc<dyn KeyringPort>)
        }
        SecureStorageCapability::Unsupported => {
            log::error!("Secure storage unsupported: {:?}", capability);
            Err(KeyringFactoryError::Unsupported { capability })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::capability::SecureStorageCapability;

    use super::*;

    #[test]
    fn keyring_from_capability_unsupported_returns_error() {
        let result =
            keyring_from_capability_with_base_dir(SecureStorageCapability::Unsupported, None);
        assert!(matches!(
            result,
            Err(KeyringFactoryError::Unsupported { .. })
        ));
    }

    #[test]
    fn keyring_from_capability_system_keyring_returns_ok() {
        let result =
            keyring_from_capability_with_base_dir(SecureStorageCapability::SystemKeyring, None);
        assert!(result.is_ok());
    }

    #[test]
    fn keyring_from_capability_file_based_keystore_returns_ok() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let result = keyring_from_capability_with_base_dir(
            SecureStorageCapability::FileBasedKeystore,
            Some(temp_dir.path().to_path_buf()),
        );
        assert!(result.is_ok());
    }
}
