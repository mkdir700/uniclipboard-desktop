use crate::{
    ids::{EntryId, EventId},
    BlobId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardEntry {
    pub entry_id: EntryId,
    pub event_id: EventId,
    pub created_at_ms: i64,
    pub title: Option<String>,
    pub total_size: i64,
}

#[derive(Debug, Clone)]
pub enum SelectionState {
    /// 小内容直接 inline（例如小文本、rtf 小片段等）
    Inline {
        mime: Option<String>,
        bytes: Vec<u8>,
    },

    /// 需要 blob，但尚未 materialize（Capture 阶段生成）
    PendingBlob {
        mime: Option<String>,
        representation_id: String, // 指向 snapshot.representations[i].id
    },

    /// 已 materialize 的 blob
    MaterializedBlob {
        mime: Option<String>,
        blob_id: BlobId,
    },
}

#[derive(Debug, Clone)]
pub struct EntrySelection {
    pub entry_id: EntryId,
    pub state: SelectionState,
}
