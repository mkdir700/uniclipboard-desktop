//! Secure storage selection and default secure storage factory.

use std::{fs, path::PathBuf, sync::Arc};
use tracing::{debug, error, info, warn};

use uc_core::ports::SecureStoragePort;

use crate::{
    capability::{detect_storage_capability, SecureStorageCapability},
    file_secure_storage::FileSecureStorage,
    system_secure_storage::SystemSecureStorage,
};

#[derive(Debug, thiserror::Error)]
pub enum SecureStorageFactoryError {
    #[error("secure storage unsupported: {capability:?}")]
    Unsupported { capability: SecureStorageCapability },

    #[error("failed to initialize file-based secure storage: {0}")]
    FileBasedInit(#[from] std::io::Error),
}

fn secure_storage_from_capability(
    capability: SecureStorageCapability,
) -> Result<Arc<dyn SecureStoragePort>, SecureStorageFactoryError> {
    secure_storage_from_capability_with_base_dir(capability, None)
}

/// Create a secure storage instance matching the provided secure storage capability.
///
/// If `capability` indicates system storage, returns a system-backed implementation wrapped in
/// `Arc<dyn SecureStoragePort>`. If `capability` is `FileBasedKeystore`, returns a file-backed
/// implementation using the provided `base_dir`. If `base_dir` is `None`,
/// returns `SecureStorageFactoryError::FileBasedInit` with `std::io::ErrorKind::NotFound`.
/// If `capability` is `Unsupported`, returns `SecureStorageFactoryError::Unsupported` containing
/// the provided capability.
///
/// The `base_dir` argument supplies the application data root required for file-based storage;
/// when present the directory will be created if it does not exist.
///
/// # Examples
///
/// ```ignore
/// # use std::sync::Arc;
/// # use std::path::PathBuf;
/// # use uc_platform::capability::SecureStorageCapability;
/// # use uc_platform::secure_storage::secure_storage_from_capability_with_base_dir;
/// let temp_dir = std::env::temp_dir();
/// let storage = secure_storage_from_capability_with_base_dir(
///     SecureStorageCapability::FileBasedKeystore,
///     Some(temp_dir),
/// );
/// assert!(storage.is_ok());
/// ```
fn secure_storage_from_capability_with_base_dir(
    capability: SecureStorageCapability,
    base_dir: Option<PathBuf>,
) -> Result<Arc<dyn SecureStoragePort>, SecureStorageFactoryError> {
    match capability {
        SecureStorageCapability::SystemKeyring => {
            Ok(Arc::new(SystemSecureStorage::new()) as Arc<dyn SecureStoragePort>)
        }
        SecureStorageCapability::FileBasedKeystore => {
            if let Some(base_dir) = base_dir {
                fs::create_dir_all(&base_dir)?;
                Ok(Arc::new(FileSecureStorage::with_base_dir(base_dir))
                    as Arc<dyn SecureStoragePort>)
            } else {
                Err(SecureStorageFactoryError::FileBasedInit(
                    std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "File-based secure storage requires app data root",
                    ),
                ))
            }
        }
        SecureStorageCapability::Unsupported => {
            Err(SecureStorageFactoryError::Unsupported { capability })
        }
    }
}

/// Create a default secure storage implementation based on the detected storage capability.
///
/// The function selects an appropriate secure storage implementation for the current
/// environment:
/// - If system secure storage is available, returns a system-backed implementation.
/// - If only a file-based keystore is available, returns a `FileBasedInit` error
///   indicating an application data root is required.
/// - If secure storage is unsupported, returns an `Unsupported` error.
///
/// # Returns
///
/// `Ok(Arc<dyn SecureStoragePort>)` with the selected storage on success; otherwise
/// an appropriate `SecureStorageFactoryError` describing why storage could not be
/// created (`FileBasedInit` when an app data root is required, or
/// `Unsupported` when no secure storage is available).
///
/// # Examples
///
/// ```
/// use uc_platform::secure_storage::create_default_secure_storage;
/// let _ = create_default_secure_storage();
/// ```
pub fn create_default_secure_storage(
) -> Result<Arc<dyn SecureStoragePort>, SecureStorageFactoryError> {
    let capability = detect_storage_capability();
    debug!(capability = ?capability, "Detected secure storage capability");

    match capability {
        SecureStorageCapability::SystemKeyring => {
            info!("Using system secure storage");
            secure_storage_from_capability(capability)
        }
        SecureStorageCapability::FileBasedKeystore => {
            warn!(
                "File-based secure storage requires app data root; use create_default_secure_storage_in_app_data_root"
            );
            Err(SecureStorageFactoryError::FileBasedInit(
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "File-based secure storage requires app data root",
                ),
            ))
        }
        SecureStorageCapability::Unsupported => {
            error!(capability = ?capability, "Secure storage unsupported");
            Err(SecureStorageFactoryError::Unsupported { capability })
        }
    }
}

/// Create a default secure storage using `app_data_root` when a file-based keystore is required.
///
/// Detects the platform's secure storage capability and returns an appropriate `SecureStoragePort`:
/// - If system secure storage is available, returns the system-backed implementation.
/// - If a file-based keystore is detected, initializes a file-backed implementation rooted at
///   `app_data_root`.
/// - If secure storage is unsupported, returns `SecureStorageFactoryError::Unsupported`.
///
/// # Parameters
///
/// - `app_data_root`: Path to the application's data root used to initialize file-based storage.
///
/// # Errors
///
/// Returns `SecureStorageFactoryError::Unsupported` when secure storage is not available.
/// Returns `SecureStorageFactoryError::FileBasedInit` if initialization of file-based storage fails.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use uc_platform::secure_storage::{
///     create_default_secure_storage_in_app_data_root, SecureStorageFactoryError,
/// };
/// let app_data_root = std::env::temp_dir().join("my_app_storage");
/// let res = create_default_secure_storage_in_app_data_root(app_data_root);
/// // On platforms with system secure storage support this may still return Ok.
/// assert!(matches!(res, Ok(_)) || matches!(res, Err(SecureStorageFactoryError::Unsupported { .. })));
/// ```
pub fn create_default_secure_storage_in_app_data_root(
    app_data_root: PathBuf,
) -> Result<Arc<dyn SecureStoragePort>, SecureStorageFactoryError> {
    let capability = detect_storage_capability();
    debug!(capability = ?capability, "Detected secure storage capability");

    match capability {
        SecureStorageCapability::SystemKeyring => {
            info!("Using system secure storage");
            secure_storage_from_capability(capability)
        }
        SecureStorageCapability::FileBasedKeystore => {
            warn!("Using file-based secure storage (insecure dev fallback for WSL/headless environments)");
            Ok(
                Arc::new(FileSecureStorage::new_in_app_data_root(app_data_root)?)
                    as Arc<dyn SecureStoragePort>,
            )
        }
        SecureStorageCapability::Unsupported => {
            error!(capability = ?capability, "Secure storage unsupported");
            Err(SecureStorageFactoryError::Unsupported { capability })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::capability::SecureStorageCapability;

    use super::*;

    #[test]
    fn secure_storage_from_capability_unsupported_returns_error() {
        let result = secure_storage_from_capability_with_base_dir(
            SecureStorageCapability::Unsupported,
            None,
        );
        assert!(matches!(
            result,
            Err(SecureStorageFactoryError::Unsupported { .. })
        ));
    }

    #[test]
    fn secure_storage_from_capability_system_storage_returns_ok() {
        let result = secure_storage_from_capability_with_base_dir(
            SecureStorageCapability::SystemKeyring,
            None,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn secure_storage_from_capability_file_based_keystore_returns_ok() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let result = secure_storage_from_capability_with_base_dir(
            SecureStorageCapability::FileBasedKeystore,
            Some(temp_dir.path().to_path_buf()),
        );
        assert!(result.is_ok());
    }
}
