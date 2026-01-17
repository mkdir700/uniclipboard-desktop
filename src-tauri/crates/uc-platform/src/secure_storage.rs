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

/// Create a keyring instance matching the provided secure storage capability.
///
/// If `capability` is `SystemKeyring`, returns a `SystemKeyring` wrapped in `Arc<dyn KeyringPort>`.
/// If `capability` is `FileBasedKeystore`, returns a `FileBasedKeyring` using the provided
/// `base_dir`. If `base_dir` is `None`, returns `KeyringFactoryError::FileBasedInit` with
/// `std::io::ErrorKind::NotFound`. If `capability` is `Unsupported`, returns
/// `KeyringFactoryError::Unsupported` containing the provided capability.
///
/// The `base_dir` argument supplies the application data root required for file-based storage;
/// when present the directory will be created if it does not exist.
///
/// # Examples
///
/// ```
/// # use std::sync::Arc;
/// # use std::path::PathBuf;
/// # use crate::{keyring_from_capability_with_base_dir, SecureStorageCapability};
/// // System keyring
/// let keyring = keyring_from_capability_with_base_dir(SecureStorageCapability::SystemKeyring, None);
/// assert!(keyring.is_ok());
/// ```
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

/// Create a default secure keyring based on the detected storage capability.
///
/// The function selects an appropriate keyring implementation for the current
/// environment:
/// - If a system keyring is available, returns a system-backed keyring.
/// - If only a file-based keystore is available, returns a `FileBasedInit`
///   error indicating an application data root is required.
/// - If secure storage is unsupported, returns an `Unsupported` error.
///
/// # Returns
///
/// `Ok(Arc<dyn KeyringPort>)` with the selected keyring on success; otherwise
/// an appropriate `KeyringFactoryError` describing why a keyring could not be
/// created (`FileBasedInit` when an app data root is required, or
/// `Unsupported` when no secure storage is available).
///
/// # Examples
///
/// ```
/// let _ = create_default_keyring();
/// ```
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

/// Create a default keyring using `app_data_root` when a file-based keystore is required.
///
/// Detects the platform's secure storage capability and returns an appropriate `KeyringPort`:
/// - If the system keyring is available, returns the system keyring.
/// - If a file-based keystore is detected, initializes a `FileBasedKeyring` rooted at `app_data_root`.
/// - If secure storage is unsupported, returns `KeyringFactoryError::Unsupported`.
///
/// # Parameters
///
/// - `app_data_root`: Path to the application's data root used to initialize a file-based keyring.
///
/// # Errors
///
/// Returns `KeyringFactoryError::Unsupported` when secure storage is not available.
/// Returns `KeyringFactoryError::FileBasedInit` if initialization of a file-based keyring fails.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// let app_data_root = std::env::temp_dir().join("my_app_keyring");
/// let res = create_default_keyring_in_app_data_root(app_data_root);
/// // On platforms with system keyring support this may still return Ok with a system keyring.
/// assert!(res.is_ok() || matches!(res.unwrap_err(), crate::KeyringFactoryError::Unsupported));
/// ```
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
            Ok(
                Arc::new(FileBasedKeyring::new_in_app_data_root(app_data_root)?)
                    as Arc<dyn KeyringPort>,
            )
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
