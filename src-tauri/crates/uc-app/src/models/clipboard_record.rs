use uc_core::clipboard::ClipboardContent;

pub struct ClipboardRecord {
    pub id: String, // content_hash
    pub content: ClipboardContentRef,
    pub state: ClipboardState,
    pub created_at: i64,
}

pub enum ClipboardContentRef {
    Inline(ClipboardContent),
    External {
        meta: BTreeMap<String, String>,
        encrypted_path: String,
    },
}

pub struct ClipboardState {
    pub favorited: bool,
    pub deleted: bool,
}
