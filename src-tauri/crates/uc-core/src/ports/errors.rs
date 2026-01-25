use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeviceRepositoryError {
    #[error("device not found")]
    NotFound,

    #[error("storage error: {0}")]
    Storage(String),
}

#[derive(Debug, Error)]
pub enum AppDirsError {
    #[error("system data-local directory unavailable")]
    DataLocalDirUnavailable,

    #[error("system cache directory unavailable")]
    CacheDirUnavailable,

    #[error("platform error: {0}")]
    Platform(String),
}

#[derive(Debug, Error)]
pub enum PairedDeviceRepositoryError {
    #[error("paired device not found")]
    NotFound,

    #[error("storage error: {0}")]
    Storage(String),
}
