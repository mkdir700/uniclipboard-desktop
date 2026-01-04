//! Clipboard domain models.

pub mod content_type;
pub mod metadata;
pub mod payload;

pub use content_type::ContentType;
pub use metadata::{TextMetadata, ImageMetadata, FileMetadata};
pub use payload::{Payload, TextPayload, ImagePayload, FilePayload, FileInfo};
