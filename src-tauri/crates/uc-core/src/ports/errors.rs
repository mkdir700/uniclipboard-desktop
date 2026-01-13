use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeviceRepositoryError {
    #[error("device not found")]
    NotFound,

    #[error("storage error: {0}")]
    Storage(String),
}
