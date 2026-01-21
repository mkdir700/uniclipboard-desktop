use crate::clipboard::{MimeType, TimestampMs};
use crate::ids::{BlobId, RepresentationId};

/// Thumbnail metadata associated with a clipboard representation.
///
/// 与剪贴板表示关联的缩略图元数据。
pub struct ThumbnailMetadata {
    /// Representation identifier this thumbnail belongs to.
    ///
    /// 此缩略图所属的表示标识符。
    pub representation_id: RepresentationId,
    /// Blob identifier where the thumbnail bytes are stored.
    ///
    /// 缩略图字节存储位置的 Blob 标识符。
    pub thumbnail_blob_id: BlobId,
    /// MIME type of the thumbnail bytes (e.g. image/webp).
    ///
    /// 缩略图字节的 MIME 类型（例如 image/webp）。
    pub thumbnail_mime_type: MimeType,
    /// Thumbnail width in pixels.
    ///
    /// 缩略图宽度（像素）。
    pub width: i32,
    /// Thumbnail height in pixels.
    ///
    /// 缩略图高度（像素）。
    pub height: i32,
    /// Logical size in bytes of the thumbnail payload.
    ///
    /// 缩略图负载的逻辑大小（字节）。
    pub size_bytes: i64,
    /// Optional creation timestamp (epoch millis).
    ///
    /// 可选的创建时间戳（毫秒）。
    pub created_at_ms: Option<TimestampMs>,
}

impl ThumbnailMetadata {
    pub fn new(
        representation_id: RepresentationId,
        thumbnail_blob_id: BlobId,
        thumbnail_mime_type: MimeType,
        width: i32,
        height: i32,
        size_bytes: i64,
        created_at_ms: Option<TimestampMs>,
    ) -> Self {
        Self {
            representation_id,
            thumbnail_blob_id,
            thumbnail_mime_type,
            width,
            height,
            size_bytes,
            created_at_ms,
        }
    }
}

#[cfg(test)]
#[test]
fn test_thumbnail_metadata_builds() {
    let meta = ThumbnailMetadata::new(
        RepresentationId::new(),
        BlobId::new(),
        MimeType("image/webp".to_string()),
        640,
        480,
        1234,
        Some(TimestampMs::from_epoch_millis(1)),
    );
    assert_eq!(meta.thumbnail_mime_type.as_str(), "image/webp");
}
