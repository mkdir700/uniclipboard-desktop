use crate::ids::{FormatId, RepresentationId};
use crate::{BlobId, MimeType};

pub struct PersistedClipboardRepresentation {
    pub id: RepresentationId,

    /// Clipboard format identifier (e.g. public.utf8-plain-text)
    pub format_id: FormatId,

    pub mime_type: Option<MimeType>,

    /// Logical size in bytes of the original clipboard representation payload.
    /// This value represents the real size observed from the system clipboard,
    /// independent of storage strategy (inline / blob / lazy materialization).
    pub size_bytes: i64,

    /// Inline stored payload, only present when size is below inline threshold.
    pub inline_data: Option<Vec<u8>>,

    /// Blob identifier if the payload has been materialized into blob storage.
    pub blob_id: Option<BlobId>,
}

impl PersistedClipboardRepresentation {
    pub fn new(
        id: RepresentationId,
        format_id: FormatId,
        mime_type: Option<MimeType>,
        size_bytes: i64,
        inline_data: Option<Vec<u8>>,
        blob_id: Option<BlobId>,
    ) -> Self {
        debug_assert!(
            !(inline_data.is_some() && blob_id.is_some()),
            "inline_data and blob_id should not both be set in normal flow"
        );

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
