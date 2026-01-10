use crate::ids::{FormatId, SnapshotId};
use crate::BlobId;

pub struct NewSnapshotRepresentation {
    pub id: SnapshotId,
    pub format_id: FormatId,
    pub mime_type: Option<String>,
    pub size_bytes: i64,
    pub inline_data: Option<Vec<u8>>,
    pub blob_id: Option<BlobId>,
}

impl NewSnapshotRepresentation {
    pub fn new(
        id: SnapshotId,
        format_id: FormatId,
        mime_type: Option<String>,
        size_bytes: i64,
        inline_data: Option<Vec<u8>>,
        blob_id: Option<BlobId>,
    ) -> Self {
        Self {
            id,
            format_id,
            mime_type,
            size_bytes,
            inline_data,
            blob_id,
        }
    }
}
